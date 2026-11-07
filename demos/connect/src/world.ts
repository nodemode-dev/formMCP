export type ClientKey = "claude" | "codex" | "cursor";
export type Link = "connected" | "not-connected" | "taken" | "no-client";
export type Platform = "macos" | "windows" | "linux";
export type ShellVar = { name: string; value: string; on: boolean };
export type Backup = { path: string; at: number; text: string };

export const SERVER_NAME = "formmcp";
export const PASSED_ENV = ["FORMMCP_DEV_ROOTS", "FORMPORTAL_ROOT", "FORMPORTAL_BIN", "FORMMCP_CATALOG", "FORMAI_CATALOG_PUBKEY"];
export const LABELS: Record<ClientKey, string> = { claude: "Claude Code", codex: "Codex", cursor: "Cursor" };

type ClaudeEntry = { command: string; args: string[]; env: [string, string][] };

export const PLATFORMS: Record<Platform, { label: string; home: string; sep: string; launcher: string }> = {
  macos: { label: "macOS", home: "/Users/sam", sep: "/", launcher: "/Applications/FormMCP.app/Contents/MacOS/formmcp" },
  windows: { label: "Windows", home: "C:\\Users\\sam", sep: "\\", launcher: "C:\\Program Files\\FormMCP\\formmcp.exe" },
  linux: { label: "Linux", home: "/home/sam", sep: "/", launcher: "/usr/bin/formmcp" },
};

export const world = {
  platform: "macos" as Platform,
  launcher: PLATFORMS.macos.launcher,
  shell: [] as ShellVar[],
  claudeInstalled: true,
  claude: null as ClaudeEntry | null,
  codex: null as string | null,
  cursor: null as string | null,
  backups: [] as Backup[],
  commands: [] as string[],
};

export function guessPlatform(): Platform {
  const ua = navigator.userAgent;
  if (/Windows/i.test(ua)) return "windows";
  if (/Android/i.test(ua)) return "macos";
  if (/Linux|X11|CrOS/i.test(ua)) return "linux";
  return "macos";
}

export function home(): string {
  return PLATFORMS[world.platform].home;
}

export function join(...parts: string[]): string {
  return parts.join(PLATFORMS[world.platform].sep);
}

export function codexPath(): string {
  return join(home(), ".codex", "config.toml");
}

export function cursorPath(): string {
  return join(home(), ".cursor", "mcp.json");
}

export function pathOf(client: ClientKey): string | null {
  return client === "codex" ? codexPath() : client === "cursor" ? cursorPath() : null;
}

export function quote(text: string): string {
  return `"${text.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

export function ours(text: string): boolean {
  return text.includes("formmcp") || text.includes("formMCP");
}

export function tomlSection(text: string, name: string): [number, number] | null {
  const header = `[mcp_servers.${name}]`;
  const sub = `[mcp_servers.${name}.`;
  let offset = 0;
  let start: number | null = null;
  for (const line of text.match(/[^\n]*\n|[^\n]+$/g) ?? []) {
    const trimmed = line.trim();
    if (start !== null) {
      if (trimmed.startsWith("[") && !trimmed.startsWith(sub)) return [start, offset];
    } else if (trimmed === header) start = offset;
    offset += line.length;
  }
  return start === null ? null : [start, text.length];
}

export function passedEnv(): [string, string][] {
  return world.shell.filter((v) => v.on && v.value.trim()).map((v) => [v.name, v.value]);
}

export function sortKeys(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value && typeof value === "object") {
    const out: Record<string, unknown> = {};
    for (const key of Object.keys(value as object).sort()) out[key] = sortKeys((value as Record<string, unknown>)[key]);
    return out;
  }
  return value;
}

export function pretty(value: unknown): string {
  return `${JSON.stringify(sortKeys(value), null, 2)}\n`;
}

function parseJson(text: string | null): { ok: true; value: unknown } | { ok: false; error: string } {
  if (text === null || !text.trim()) return { ok: true, value: {} };
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch (e) {
    return { ok: false, error: String((e as Error).message ?? e) };
  }
}

export function stateOf(client: ClientKey): { state: Link; note: string | null } {
  if (client === "claude") {
    if (!world.claudeInstalled) return { state: "no-client", note: "the claude command is not installed on this computer" };
    return { state: world.claude ? "connected" : "not-connected", note: null };
  }
  const taken = "another server named formmcp is set up; left alone";
  if (client === "codex") {
    const text = world.codex ?? "";
    const sec = tomlSection(text, SERVER_NAME);
    if (!sec) return { state: "not-connected", note: null };
    return ours(text.slice(sec[0], sec[1])) ? { state: "connected", note: null } : { state: "taken", note: taken };
  }
  const parsed = parseJson(world.cursor);
  const data = parsed.ok ? (parsed.value as Record<string, unknown> | null) : null;
  const servers = data && typeof data === "object" ? (data.mcpServers as Record<string, unknown> | undefined) : undefined;
  const entry = servers && typeof servers === "object" ? servers[SERVER_NAME] : undefined;
  if (entry === undefined) return { state: "not-connected", note: null };
  return ours(JSON.stringify(entry)) ? { state: "connected", note: null } : { state: "taken", note: taken };
}

export type Plan = { text: string | null; commands: string[]; error: string | null; backup: boolean };

function claudeAdd(): string {
  const env = passedEnv();
  const parts = ["claude", "mcp", "add", "--scope", "user", SERVER_NAME];
  for (const [k, v] of env) parts.push("-e", `${k}=${v}`);
  parts.push("--", world.launcher, "mcp");
  return parts.map((p) => (/[\s"']/.test(p) ? `"${p.replace(/"/g, '\\"')}"` : p)).join(" ");
}

export function planConnect(client: ClientKey): Plan {
  const exe = world.launcher;
  const env = passedEnv();
  if (client === "claude") {
    if (!world.claudeInstalled) return { text: null, commands: [], error: "the claude command is not installed", backup: false };
    return { text: null, commands: [`claude mcp remove ${SERVER_NAME} --scope user`, claudeAdd()], error: null, backup: false };
  }
  if (client === "codex") {
    const text = world.codex ?? "";
    const envLine = env.length ? `env = { ${env.map(([k, v]) => `${k} = ${quote(v)}`).join(", ")} }\n` : "";
    const block = `[mcp_servers.${SERVER_NAME}]\ncommand = ${quote(exe)}\nargs = [${quote("mcp")}]\n${envLine}startup_timeout_sec = 600\ntool_timeout_sec = 900\n`;
    const sec = tomlSection(text, SERVER_NAME);
    const next = sec ? `${text.slice(0, sec[0])}${block}\n${text.slice(sec[1])}` : !text.trim() ? block : `${text.replace(/\s+$/, "")}\n\n${block}`;
    return { text: next, commands: [], error: null, backup: world.codex !== null };
  }
  const path = cursorPath();
  const parsed = parseJson(world.cursor);
  if (!parsed.ok) return { text: null, commands: [], error: `${path} is not valid JSON (${parsed.error}); not changed`, backup: false };
  const data = parsed.value;
  if (!data || typeof data !== "object" || Array.isArray(data)) return { text: null, commands: [], error: `${path} is not a JSON object; not changed`, backup: false };
  const obj = data as Record<string, unknown>;
  if (obj.mcpServers === undefined) obj.mcpServers = {};
  const servers = obj.mcpServers;
  if (!servers || typeof servers !== "object" || Array.isArray(servers)) return { text: null, commands: [], error: `mcpServers in ${path} is not an object; not changed`, backup: false };
  const entry: Record<string, unknown> = { command: exe, args: ["mcp"] };
  if (env.length) entry.env = Object.fromEntries(env);
  (servers as Record<string, unknown>)[SERVER_NAME] = entry;
  return { text: pretty(obj), commands: [], error: null, backup: world.cursor !== null };
}

export function planRemove(client: ClientKey): Plan {
  if (client === "claude") return { text: null, commands: world.claudeInstalled ? [`claude mcp remove ${SERVER_NAME} --scope user`] : [], error: null, backup: false };
  if (client === "codex") {
    const text = world.codex ?? "";
    const sec = tomlSection(text, SERVER_NAME);
    return sec ? { text: `${text.slice(0, sec[0])}${text.slice(sec[1])}`, commands: [], error: null, backup: true } : { text: null, commands: [], error: null, backup: false };
  }
  const parsed = parseJson(world.cursor);
  if (!parsed.ok || !parsed.value || typeof parsed.value !== "object") return { text: null, commands: [], error: null, backup: false };
  const obj = parsed.value as Record<string, unknown>;
  const servers = obj.mcpServers;
  if (servers && typeof servers === "object" && !Array.isArray(servers) && SERVER_NAME in (servers as object)) {
    delete (servers as Record<string, unknown>)[SERVER_NAME];
    return { text: pretty(obj), commands: [], error: null, backup: true };
  }
  return { text: null, commands: [], error: null, backup: false };
}

export function nextPlan(client: ClientKey): { on: boolean; plan: Plan } {
  const s = stateOf(client).state;
  if (s === "connected") return { on: false, plan: planRemove(client) };
  return { on: true, plan: planConnect(client) };
}

function backup(path: string, text: string | null): void {
  if (text !== null) world.backups.push({ path: `${path}.bak-${Math.floor(Date.now() / 1000)}`, at: Date.now(), text });
}

function apply(client: ClientKey, plan: Plan): void {
  if (plan.error) throw plan.error;
  if (client === "claude") {
    for (const c of plan.commands) world.commands.push(c);
    const connecting = plan.commands.length > 1;
    world.claude = connecting ? { command: world.launcher, args: ["mcp"], env: passedEnv() } : null;
    return;
  }
  if (plan.text === null) return;
  const path = pathOf(client) as string;
  if (plan.backup) backup(path, client === "codex" ? world.codex : world.cursor);
  if (client === "codex") world.codex = plan.text;
  else world.cursor = plan.text;
}

export function connect(client: ClientKey, on: boolean): { state: Link; note: string | null } {
  const before = stateOf(client).state;
  if (on && (before === "not-connected" || before === "connected")) apply(client, planConnect(client));
  else if (!on && before === "connected") apply(client, planRemove(client));
  return stateOf(client);
}

export type Row = { kind: "eq" | "add" | "del"; text: string };

export function diffLines(a: string, b: string): Row[] {
  const x = a.length ? a.replace(/\n$/, "").split("\n") : [];
  const y = b.length ? b.replace(/\n$/, "").split("\n") : [];
  const n = x.length;
  const m = y.length;
  const t: number[][] = Array.from({ length: n + 1 }, () => new Array<number>(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) for (let j = m - 1; j >= 0; j--) t[i][j] = x[i] === y[j] ? t[i + 1][j + 1] + 1 : Math.max(t[i + 1][j], t[i][j + 1]);
  const out: Row[] = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (x[i] === y[j]) {
      out.push({ kind: "eq", text: x[i] });
      i++;
      j++;
    } else if (t[i + 1][j] >= t[i][j + 1]) out.push({ kind: "del", text: x[i++] });
    else out.push({ kind: "add", text: y[j++] });
  }
  while (i < n) out.push({ kind: "del", text: x[i++] });
  while (j < m) out.push({ kind: "add", text: y[j++] });
  return out;
}

export function shellDefaults(): ShellVar[] {
  const h = home();
  return [
    { name: "FORMMCP_DEV_ROOTS", value: join(h, "code"), on: true },
    { name: "FORMPORTAL_ROOT", value: join(h, "formportal"), on: false },
    { name: "FORMPORTAL_BIN", value: world.platform === "windows" ? "C:\\Program Files\\FormPortal\\formportal-cli.exe" : "/usr/local/bin/formportal-cli", on: false },
    { name: "FORMMCP_CATALOG", value: "https://dl.formai.studio/catalog/apps.json", on: false },
    { name: "FORMAI_CATALOG_PUBKEY", value: "paste a public key", on: false },
  ];
}

export function baseCodex(): string {
  return `approval_policy = "on-request"\nsandbox_mode = "workspace-write"\n\n[mcp_servers.team-docs]\ncommand = ${quote(join(home(), "bin", "team-docs"))}\nargs = ["serve"]\n`;
}

export function baseCursor(): string {
  return `{\n  "mcpServers": {\n    "team-docs": {\n      "command": ${JSON.stringify(join(home(), "bin", "team-docs"))},\n      "args": ["serve"]\n    }\n  }\n}\n`;
}

export function reset(platform: Platform): void {
  world.platform = platform;
  world.launcher = PLATFORMS[platform].launcher;
  world.shell = shellDefaults();
  world.claudeInstalled = true;
  world.claude = null;
  world.codex = baseCodex();
  world.cursor = baseCursor();
  world.backups = [];
  world.commands = [];
  connect("claude", true);
  connect("codex", true);
  world.backups = [];
  world.commands = [];
}

export const changed = (): void => {
  document.dispatchEvent(new CustomEvent("world-change"));
};
