import family from "../../core/data/family.json";

type App = (typeof family.apps)[number];
type Filter = "all" | "mcp";

declare const __APP_VERSION__: string;

const STATES: Record<string, string> = { live: "Live", public: "Public", prelaunch: "Coming" };
const AGENTS: Record<string, string> = { claude: "Claude Code", codex: "Codex", cursor: "Cursor", other: "other agents" };
const START_LABELS: [string, string][] = [["demand", "Start on demand"], ["session", "Start with the session"], ["keep", "Keep running"]];

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls = "", text = ""): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text) node.textContent = text;
  return node;
}

function badge(label: string, kind: string, on: boolean): HTMLElement {
  const b = el("span", `badge badge-${kind}${on ? " is-on" : ""}`, label);
  if (!on) b.setAttribute("aria-label", `${label}: not yet`);
  return b;
}

function row(app: App): HTMLLIElement {
  const li = el("li", "app");
  li.dataset.mcp = String(app.mcp);
  const head = el("div", "app-head");
  const name = el("span", "app-name", app.name);
  name.translate = false;
  head.append(name, el("span", "app-kind", app.kind));
  const badges = el("div", "app-badges");
  badges.append(badge("MCP", "mcp", app.mcp), el("span", `state state-${app.state}`, STATES[app.state] ?? app.state));
  const local = el("div", "app-local");
  const where = el("span", "app-where");
  where.dataset.where = "";
  local.append(where);
  const switches = el("div", "app-switches");
  const onFor = makeSwitch(`On for ${AGENTS.claude}`, "Offer this app to the chosen agent: in family_apps, suggest_app and its tools");
  onFor.dataset.for = app.id;
  onFor.setAttribute("aria-checked", "true");
  switches.append(onFor);
  li.append(head, el("p", "app-summary", app.summary), badges, local, switches);
  if (app.mcp) {
    const sw = makeSwitch("Always show tools", "Always show this app's own tools in formMCP's tool list");
    sw.dataset.expose = app.id;
    const start = el("select", "start-pick");
    start.dataset.start = app.id;
    start.disabled = true;
    start.title = "When this app's server starts. On demand stops it after 10 idle minutes";
    start.setAttribute("aria-label", `When ${app.name}'s server starts`);
    for (const [value, label] of START_LABELS) {
      const o = el("option", "", label);
      o.value = value;
      start.append(o);
    }
    switches.append(start);
    const toolsButton = el("button", "btn btn-small app-tools-btn", "Tools");
    toolsButton.type = "button";
    toolsButton.dataset.tools = app.id;
    toolsButton.setAttribute("aria-expanded", "false");
    toolsButton.disabled = true;
    local.append(toolsButton);
    switches.append(sw);
    const panel = el("div", "tool-panel");
    panel.dataset.panel = app.id;
    panel.hidden = true;
    li.append(panel);
  }
  return li;
}

function makeSwitch(label: string, title: string): HTMLButtonElement {
  const sw = el("button", "switch");
  sw.type = "button";
  sw.setAttribute("role", "switch");
  sw.setAttribute("aria-checked", "false");
  sw.disabled = true;
  sw.title = title;
  sw.append(el("span", "switch-track"), el("span", "switch-label", label));
  return sw;
}

const list = document.getElementById("apps") as HTMLUListElement;
const count = document.getElementById("count") as HTMLParagraphElement;
const rows = family.apps.map((app) => ({ app, node: row(app) }));
list.append(...rows.map((r) => r.node));

(document.getElementById("version") as HTMLElement).textContent = `v${__APP_VERSION__}`;
const commit = family.source.commit.slice(0, 7);
(document.getElementById("source") as HTMLElement).textContent = `List from ${family.source.repository} ${family.source.version} (${commit})`;

const chips = Array.from(document.querySelectorAll<HTMLButtonElement>(".chip"));
const withMcp = family.apps.filter((a) => a.mcp).length;

function apply(filter: Filter): void {
  let shown = 0;
  for (const { app, node } of rows) {
    const visible = filter === "all" || app.mcp;
    if (node.hidden === visible) node.hidden = !visible;
    if (visible) shown += 1;
  }
  for (const chip of chips) chip.setAttribute("aria-pressed", String(chip.dataset.filter === filter));
  count.textContent = filter === "all" ? `${family.apps.length} apps · ${withMcp} with an MCP server` : `${shown} of ${family.apps.length} apps`;
}

for (const chip of chips) chip.addEventListener("click", () => apply((chip.dataset.filter as Filter) ?? "all"));
apply("all");

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
type LiveApp = { id: string; installed: boolean; reach: string | null; exposed: boolean; mcp: boolean; version: string; off_for: string[]; profile_off_for: string[]; start: string; hidden_tools: number };
type ProfileRow = { name: string; apps_off: number; tools_off: number; starts: number; used_by: string[] };
type Profiles = { list: ProfileRow[]; in_use: Record<string, string> };
type ClientState = { client: string; state: "connected" | "not-connected" | "taken" | "no-client"; note: string | null };
type HttpState = { running: boolean; port: number; address: string };
type Snapshot = { source: string; apps: LiveApp[]; clients: ClientState[]; formportal: boolean; env_expose: boolean; http: HttpState; profiles: Profiles };
type ToolRow = { name: string; description: string; read_only: boolean; on: boolean };
type ToolList = { app: string; loaded: number | null; tools: ToolRow[] };
type LogEvent = { t: number; event: string; client?: string; name?: string; app?: string | null; tool?: string | null; ok?: boolean; ms?: number; via?: string; error?: string; tools?: number; as?: string; transport?: string; port?: number; status?: number; reason?: string | null; minutes?: number; policy?: string; profile?: string | null; stage?: string; hook?: string; redacted?: number | null; verified?: string | null };
type SecretRow = { app: string; name: string; stored: boolean; error: string | null };
type Safety = { redact: boolean; verify: boolean; hooks: { before?: string; after?: string }; telemetry: string | null; secrets: SecretRow[]; checked: { app: string; version: string; files: number; at: number }[] };

const invoke = (window as unknown as { __TAURI__?: { core?: { invoke?: Invoke } } }).__TAURI__?.core?.invoke;
const status = document.getElementById("status") as HTMLParagraphElement;
const source = document.getElementById("source") as HTMLElement;
const logList = document.getElementById("log") as HTMLOListElement;
const LINK: Record<ClientState["state"], string> = { connected: "Connected", "not-connected": "Not connected", taken: "Another server uses the name", "no-client": "Not installed" };
const LOG_ROWS = 300;
let logNext = 0;
let lastSnapshot = 0;

function setText(node: Element | null, text: string): void {
  if (node && node.textContent !== text) node.textContent = text;
}

function whereText(a: LiveApp): string {
  const installed = a.installed ? `Installed${a.version ? ` · ${a.version}` : ""}` : "Not installed";
  if (!a.mcp) return installed;
  return a.reach ? `${installed} · reachable (${a.reach})` : `${installed} · not reachable here`;
}

function showClients(list: ClientState[]): void {
  for (const c of list) {
    const card = document.querySelector<HTMLElement>(`.client[data-client="${c.client}"]`);
    if (!card) continue;
    card.dataset.link = c.state;
    setText(card.querySelector("[data-state]"), LINK[c.state]);
    const btn = card.querySelector<HTMLButtonElement>("[data-connect]");
    if (!btn) continue;
    setText(btn, c.state === "connected" ? "Disconnect" : "Connect");
    btn.disabled = c.state === "taken" || c.state === "no-client";
    btn.title = c.note ?? "";
  }
}

const agentPick = document.getElementById("agent") as HTMLSelectElement;
const httpState = document.getElementById("http-state") as HTMLElement;
const httpToggle = document.getElementById("http-toggle") as HTMLButtonElement;
const httpCopy = document.getElementById("http-copy") as HTMLButtonElement;
const httpRotate = document.getElementById("http-rotate") as HTMLButtonElement;
const offFor = new Map<string, Set<string>>();
let httpNow: HttpState | null = null;

const profilePick = document.getElementById("profile-pick") as HTMLSelectElement;
const profilesToggle = document.getElementById("profiles-toggle") as HTMLButtonElement;
const profilesBox = document.getElementById("profiles") as HTMLElement;
const profileForm = document.getElementById("profile-form") as HTMLFormElement;
const profileName = document.getElementById("profile-name") as HTMLInputElement;
const profileSave = document.getElementById("profile-save") as HTMLButtonElement;
const profileImport = document.getElementById("profile-import") as HTMLButtonElement;
const profileFile = document.getElementById("profile-file") as HTMLInputElement;
const profileNote = document.getElementById("profile-note") as HTMLElement;
const profileList = document.getElementById("profile-list") as HTMLUListElement;
const profileOff = new Map<string, Set<string>>();
let profilesNow: Profiles = { list: [], in_use: {} };

const safetyToggle = document.getElementById("safety-toggle") as HTMLButtonElement;
const safetyBox = document.getElementById("safety") as HTMLElement;
const safetyNote = document.getElementById("safety-note") as HTMLElement;
const secretForm = document.getElementById("secret-form") as HTMLFormElement;
const secretApp = document.getElementById("secret-app") as HTMLSelectElement;
const secretName = document.getElementById("secret-name") as HTMLInputElement;
const secretValue = document.getElementById("secret-value") as HTMLInputElement;
const secretSave = document.getElementById("secret-save") as HTMLButtonElement;
const safetyList = document.getElementById("safety-list") as HTMLUListElement;
const redactSwitch = document.getElementById("redact-switch") as HTMLButtonElement;
const verifySwitch = document.getElementById("verify-switch") as HTMLButtonElement;
const hookForm = document.getElementById("hook-form") as HTMLFormElement;
const hookBefore = document.getElementById("hook-before") as HTMLInputElement;
const hookAfter = document.getElementById("hook-after") as HTMLInputElement;
const hookSave = document.getElementById("hook-save") as HTMLButtonElement;
const traceForm = document.getElementById("trace-form") as HTMLFormElement;
const traceAddress = document.getElementById("trace-address") as HTMLInputElement;
const traceSave = document.getElementById("trace-save") as HTMLButtonElement;
let safetyNow: Safety | null = null;

for (const app of family.apps.filter((a) => a.mcp)) {
  const o = el("option", "", app.name);
  o.value = app.id;
  secretApp.append(o);
}

function appName(id: string): string {
  return family.apps.find((a) => a.id === id)?.name ?? id;
}

function closeSafety(): void {
  safetyBox.hidden = true;
  safetyToggle.setAttribute("aria-expanded", "false");
}

function twoClick(button: HTMLButtonElement, ask: string, run: () => void): void {
  let armed = 0;
  button.addEventListener("click", () => {
    if (Date.now() > armed) {
      armed = Date.now() + 4000;
      setText(safetyNote, ask);
      return;
    }
    button.disabled = true;
    run();
  });
}

function showSafety(s: Safety): void {
  safetyNow = s;
  redactSwitch.setAttribute("aria-checked", String(s.redact));
  verifySwitch.setAttribute("aria-checked", String(s.verify));
  redactSwitch.disabled = false;
  verifySwitch.disabled = false;
  if (document.activeElement !== hookBefore) hookBefore.value = s.hooks.before ?? "";
  if (document.activeElement !== hookAfter) hookAfter.value = s.hooks.after ?? "";
  if (document.activeElement !== traceAddress) traceAddress.value = s.telemetry ?? "";
  hookSave.disabled = hookBefore.value.trim() === (s.hooks.before ?? "") && hookAfter.value.trim() === (s.hooks.after ?? "");
  traceSave.disabled = traceAddress.value.trim().replace(/\/+$/, "") === (s.telemetry ?? "");
  const secretRows = s.secrets.map((r) => {
    const li = el("li", "profile-row");
    const what = el("span", `profile-what${r.stored ? "" : " is-fail"}`, `Secret ${r.name} for ${appName(r.app)} · ${r.stored ? "in the keychain" : r.error ?? "missing"}`);
    what.title = r.error ?? "";
    const remove = el("button", "btn btn-small btn-danger", "Remove");
    remove.type = "button";
    twoClick(remove, `Click Remove again to delete ${r.name} for ${appName(r.app)} from the keychain.`, () => void safetyCall("secret_remove", { app: r.app, name: r.name }, `Removed ${r.name} for ${appName(r.app)}.`));
    li.append(what, remove);
    return li;
  });
  const checkedRows = s.checked.map((c) => {
    const li = el("li", "profile-row");
    const when = new Date(c.at * 1000).toLocaleDateString();
    li.append(el("span", "profile-what", `Files of ${appName(c.app)} ${c.version} · ${c.files} recorded ${when}`));
    const again = el("button", "btn btn-small", "Record again");
    again.type = "button";
    again.title = "Accept the files as they are now, after you changed them yourself";
    twoClick(again, `Click Record again to accept ${appName(c.app)}'s files as they are now. Only do this for a change you made.`, () => void safetyCall("verify_trust", { app: c.app }, `Recorded ${appName(c.app)}'s files as they are now.`));
    li.append(again);
    return li;
  });
  safetyList.replaceChildren(...checkedRows, ...secretRows);
}

async function safetyCall(cmd: string, args: Record<string, unknown>, done: string): Promise<boolean> {
  if (!invoke) return false;
  try {
    showSafety((await invoke(cmd, args)) as Safety);
    if (done) setText(safetyNote, done);
    return true;
  } catch (e) {
    setText(safetyNote, `Could not save: ${String(e)}`);
    if (safetyNow) showSafety(safetyNow);
    return false;
  }
}

async function loadSafety(): Promise<void> {
  if (!invoke) return;
  try {
    showSafety((await invoke("safety")) as Safety);
  } catch (e) {
    setText(safetyNote, `Could not read the safety settings: ${String(e)}`);
  }
}

function profileNameOk(name: string): boolean {
  return /^[A-Za-z0-9_-]([A-Za-z0-9 _-]{0,38}[A-Za-z0-9_-])?$/.test(name);
}

function showProfiles(p: Profiles): void {
  profilesNow = p;
  const names = p.list.map((r) => r.name);
  if (profilePick.dataset.key !== names.join("|")) {
    profilePick.dataset.key = names.join("|");
    const none = el("option", "", "None");
    none.value = "";
    profilePick.replaceChildren(none, ...names.map((n) => {
      const o = el("option", "", n);
      o.value = n;
      return o;
    }));
  }
  profilePick.value = p.in_use[agentPick.value] ?? "";
  profilePick.disabled = !invoke || !names.length;
  profilesToggle.disabled = !invoke;
  safetyToggle.disabled = !invoke;
  const items = p.list.map((r) => {
    const li = el("li", "profile-row");
    const name = el("span", "profile-name", r.name);
    name.translate = false;
    const what = [r.apps_off ? `${r.apps_off} app${r.apps_off === 1 ? "" : "s"} off` : "", r.tools_off ? `${r.tools_off} tool${r.tools_off === 1 ? "" : "s"} off` : "", r.starts ? `${r.starts} start polic${r.starts === 1 ? "y" : "ies"}` : ""].filter(Boolean).join(" · ") || "nothing off";
    const used = r.used_by.length ? ` · used by ${r.used_by.map((c) => AGENTS[c] ?? c).join(", ")}` : "";
    const exportButton = el("button", "btn btn-small", "Export");
    exportButton.type = "button";
    exportButton.addEventListener("click", () => void profileAction("profile_export", r.name, exportButton));
    const deleteButton = el("button", "btn btn-small btn-danger", "Delete");
    deleteButton.type = "button";
    let armed = 0;
    deleteButton.addEventListener("click", () => {
      if (Date.now() > armed) {
        armed = Date.now() + 4000;
        setText(profileNote, `Click Delete again to delete ${r.name}${r.used_by.length ? "; agents using it go back to no profile" : ""}.`);
        return;
      }
      void profileAction("profile_delete", r.name, deleteButton);
    });
    li.append(name, el("span", "profile-what", what + used), exportButton, deleteButton);
    return li;
  });
  if (!items.length) items.push(el("li", "profile-empty", "No profiles yet. Name one above to save the chosen agent's settings, or import a profile file."));
  profileList.replaceChildren(...items);
}

async function profileAction(cmd: "profile_export" | "profile_delete", name: string, button: HTMLButtonElement): Promise<void> {
  if (!invoke) return;
  button.disabled = true;
  try {
    if (cmd === "profile_export") {
      const path = (await invoke(cmd, { name })) as string;
      setText(profileNote, `Exported ${name} to ${path}`);
      profileNote.title = path;
    } else {
      showProfiles((await invoke(cmd, { name })) as Profiles);
      setText(profileNote, `Deleted ${name}.`);
      void refresh();
    }
  } catch (e) {
    setText(profileNote, `Could not ${cmd === "profile_export" ? "export" : "delete"} ${name}: ${String(e)}`);
  } finally {
    button.disabled = false;
  }
}

function showAgent(): void {
  const agent = agentPick.value;
  for (const { app, node } of rows) {
    const sw = node.querySelector<HTMLButtonElement>("[data-for]");
    if (!sw) continue;
    const byProfile = !!profileOff.get(app.id)?.has(agent);
    sw.setAttribute("aria-checked", String(!byProfile && !offFor.get(app.id)?.has(agent)));
    setText(sw.querySelector(".switch-label"), `On for ${AGENTS[agent] ?? agent}`);
    sw.disabled = !invoke || byProfile;
    sw.title = byProfile ? `Turned off for ${AGENTS[agent]} by the profile ${profilesNow.in_use[agent] ?? ""}` : "Offer this app to the chosen agent: in family_apps, suggest_app and its tools";
  }
  profilePick.value = profilesNow.in_use[agent] ?? "";
  setText(profileSave, `Save ${AGENTS[agent] === "other agents" ? "other agents'" : `${AGENTS[agent]}'s`} settings`);
}

function showHttp(h: HttpState): void {
  httpNow = h;
  setText(httpState, h.running ? `Serving ${h.address} while this window is open · 127.0.0.1 only, access token required` : `Off · starts at ${h.address} on this computer only`);
  httpState.classList.toggle("is-on", h.running);
  setText(httpToggle, h.running ? "Stop" : "Start");
  httpToggle.disabled = !invoke;
  httpCopy.disabled = !invoke;
  httpRotate.disabled = !invoke;
}

function showSnapshot(s: Snapshot): void {
  const byId = new Map(s.apps.map((a) => [a.id, a]));
  for (const { app, node } of rows) {
    const live = byId.get(app.id);
    if (!live) continue;
    setText(node.querySelector("[data-where]"), whereText(live));
    offFor.set(app.id, new Set(live.off_for));
    profileOff.set(app.id, new Set(live.profile_off_for));
    const startPick = node.querySelector<HTMLSelectElement>("[data-start]");
    if (startPick) {
      startPick.value = live.start;
      startPick.disabled = false;
    }
    const sw = node.querySelector<HTMLButtonElement>("[data-expose]");
    if (sw) {
      sw.setAttribute("aria-checked", String(live.exposed));
      sw.disabled = s.env_expose;
    }
    const tb = node.querySelector<HTMLButtonElement>("[data-tools]");
    if (tb) {
      tb.disabled = false;
      tb.title = live.hidden_tools ? `${live.hidden_tools} of its tools turned off` : "Turn single tools on or off";
      tb.classList.toggle("has-off", live.hidden_tools > 0);
    }
  }
  showProfiles(s.profiles);
  showAgent();
  showHttp(s.http);
  showClients(s.clients);
  setText(source, s.source);
  const installed = s.apps.filter((a) => a.installed).length;
  const reachable = s.apps.filter((a) => a.reach).length;
  setText(status, `${installed} installed · ${reachable} reachable here · FormPortal ${s.formportal ? "found" : "not found on this computer"}${s.env_expose ? " · FORMMCP_EXPOSE is set, so the switches follow it" : ""}`);
}

async function refresh(): Promise<void> {
  if (!invoke) return;
  lastSnapshot = Date.now();
  try {
    showSnapshot((await invoke("snapshot")) as Snapshot);
  } catch (e) {
    setText(status, `Could not read this computer's state: ${String(e)}`);
  }
}

function clock(t: number): string {
  const d = new Date(t);
  return [d.getHours(), d.getMinutes(), d.getSeconds()].map((n) => String(n).padStart(2, "0")).join(":");
}

function describe(e: LogEvent): string {
  if (e.event === "session") return `Session started by ${e.client ?? "a client"}${e.as ? ` as ${AGENTS[e.as] ?? e.as}` : ""}${e.transport === "http" ? " over HTTP" : ""}`;
  if (e.event === "app-stop") return `Stopped ${e.app} after ${e.minutes ?? ""} idle minutes`;
  if (e.event === "http-start") return `HTTP started on 127.0.0.1:${e.port ?? ""}`;
  if (e.event === "http-stop") return `HTTP stopped on 127.0.0.1:${e.port ?? ""}`;
  if (e.event === "http-refused") return `HTTP request refused (${e.status ?? ""}): ${e.reason ?? ""}`;
  if (e.event === "app-start") return e.ok ? `Started ${e.app} through ${e.via}${e.policy === "session" ? " with the session" : e.policy === "keep" ? " to keep running" : ""} (${e.tools ?? 0} tools, ${e.ms} ms${e.verified === "formportal" ? ", files as FormPortal installed them" : e.verified === "match" ? ", files as recorded" : e.verified === "baseline" ? ", files recorded" : ""})` : `${e.stage === "verify" ? "Refused to start" : "Could not start"} ${e.app} through ${e.via}: ${e.error ?? ""}`;
  if (e.event === "hook") return e.hook === "before" && e.ok ? `Before-call hook allowed ${e.app} · ${e.tool}` : `${e.hook === "after" ? "After" : "Before"}-call hook ${e.ok ? "ran" : e.error ? `failed: ${e.error}` : `refused ${e.app} · ${e.tool}${e.reason ? `: ${e.reason}` : ""}`}`;
  if (e.event === "telemetry") return `Tracing could not send: ${e.error ?? ""}`;
  const parts = [e.name ?? "call", e.app, e.tool].filter(Boolean).join(" · ");
  return `${parts} · ${e.ok ? "ok" : "failed"} · ${e.ms ?? 0} ms${e.redacted ? ` · ${e.redacted} hidden` : ""}`;
}

async function pollLog(): Promise<void> {
  if (!invoke) return;
  try {
    const { events, next } = (await invoke("log_tail", { from: logNext })) as { events: LogEvent[]; next: number };
    logNext = next;
    if (events.length) {
      const atEnd = logList.scrollTop + logList.clientHeight >= logList.scrollHeight - 4;
      const items = events.map((e) => {
        const li = el("li", `log-row${e.ok === false || e.event === "http-refused" || e.event === "telemetry" ? " is-fail" : ""}`);
        li.append(el("time", "log-time", clock(e.t)), el("span", "log-text", describe(e)));
        return li;
      });
      logList.append(...items);
      while (logList.childElementCount > LOG_ROWS) logList.firstElementChild?.remove();
      if (atEnd) logList.scrollTop = logList.scrollHeight;
    }
  } catch {
    return;
  }
}

if (invoke) {
  const call = invoke;
  for (const btn of document.querySelectorAll<HTMLButtonElement>("[data-connect]")) {
    btn.addEventListener("click", async () => {
      const card = btn.closest<HTMLElement>(".client");
      const client = card?.dataset.client ?? "";
      const on = card?.dataset.link !== "connected";
      btn.disabled = true;
      try {
        const state = (await call("connect", { client, on })) as ClientState;
        showClients([state]);
        setText(status, on ? `${LINK[state.state]}: ${client} starts formMCP from now on.` : `${client} no longer starts formMCP.`);
      } catch (e) {
        btn.disabled = false;
        setText(status, `Could not change ${client}: ${String(e)}`);
      }
    });
  }
  for (const sw of document.querySelectorAll<HTMLButtonElement>("[data-expose]")) {
    sw.addEventListener("click", async () => {
      const on = sw.getAttribute("aria-checked") !== "true";
      sw.disabled = true;
      try {
        const list = (await call("set_expose", { id: sw.dataset.expose, on })) as string[];
        sw.setAttribute("aria-checked", String(list.includes(sw.dataset.expose ?? "")));
        setText(status, "Clients see this the next time they start formMCP.");
      } catch (e) {
        setText(status, `Could not save: ${String(e)}`);
      } finally {
        sw.disabled = false;
      }
    });
  }
  agentPick.addEventListener("change", showAgent);
  profilePick.addEventListener("change", async () => {
    const agent = agentPick.value;
    const name = profilePick.value || null;
    profilePick.disabled = true;
    try {
      showProfiles((await call("profile_use", { client: agent, name })) as Profiles);
      setText(status, name ? `${AGENTS[agent]} uses the profile ${name} from its next request.` : `${AGENTS[agent]} uses no profile from its next request.`);
      await refresh();
    } catch (e) {
      setText(status, `Could not change the profile: ${String(e)}`);
      showProfiles(profilesNow);
    }
  });
  profilesToggle.addEventListener("click", () => {
    const open = profilesBox.hidden;
    if (open) closeSafety();
    profilesBox.hidden = !open;
    profilesToggle.setAttribute("aria-expanded", String(open));
  });
  safetyToggle.addEventListener("click", () => {
    const open = safetyBox.hidden;
    if (open) {
      profilesBox.hidden = true;
      profilesToggle.setAttribute("aria-expanded", "false");
      void loadSafety();
    }
    safetyBox.hidden = !open;
    safetyToggle.setAttribute("aria-expanded", String(open));
  });
  const secretReady = (): void => {
    secretSave.disabled = !/^[A-Z_][A-Z0-9_]{0,63}$/.test(secretName.value.trim()) || !secretValue.value;
  };
  secretName.addEventListener("input", secretReady);
  secretValue.addEventListener("input", secretReady);
  secretForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const app = secretApp.value;
    const name = secretName.value.trim();
    const value = secretValue.value;
    secretSave.disabled = true;
    void safetyCall("secret_set", { app, name, value }, `Kept ${name} for ${appName(app)} in the keychain; formMCP gives it to ${appName(app)} the next time it starts the app.`).then((ok) => {
      if (ok) {
        secretValue.value = "";
        secretName.value = "";
      }
      secretReady();
    });
  });
  for (const [sw, key] of [[redactSwitch, "redact"], [verifySwitch, "verify"]] as const) {
    sw.addEventListener("click", () => {
      const on = sw.getAttribute("aria-checked") !== "true";
      sw.disabled = true;
      void safetyCall("safety_switch", { [key]: on }, key === "redact" ? (on ? "Secrets and token-shaped text are hidden in results from the next request." : "Results go to agents as apps send them; the log is still cleaned.") : on ? "Installed apps' files are checked before each start." : "Installed apps start without the file check.");
    });
  }
  const hookReady = (): void => {
    hookSave.disabled = hookBefore.value.trim() === (safetyNow?.hooks.before ?? "") && hookAfter.value.trim() === (safetyNow?.hooks.after ?? "");
  };
  hookBefore.addEventListener("input", hookReady);
  hookAfter.addEventListener("input", hookReady);
  hookForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    hookSave.disabled = true;
    const before = hookBefore.value.trim();
    const after = hookAfter.value.trim();
    const ok = (before === (safetyNow?.hooks.before ?? "") || (await safetyCall("hook_set", { which: "before", line: before }, ""))) && (await safetyCall("hook_set", { which: "after", line: after }, ""));
    if (ok) setText(safetyNote, before || after ? `Hooks saved: ${before ? "before each call" : ""}${before && after ? " and " : ""}${after ? "after each call" : ""}. A before hook that fails or times out refuses the call.` : "No hooks: calls run without them.");
    hookReady();
  });
  traceAddress.addEventListener("input", () => {
    traceSave.disabled = traceAddress.value.trim().replace(/\/+$/, "") === (safetyNow?.telemetry ?? "");
  });
  traceForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const address = traceAddress.value.trim();
    traceSave.disabled = true;
    void safetyCall("telemetry_set", { address: address || null }, address ? `Each call and app start is sent to ${address} as a trace and metrics: names and outcomes only, never arguments or results.` : "Tracing is off.");
  });
  profileName.addEventListener("input", () => {
    profileSave.disabled = !profileNameOk(profileName.value.trim());
  });
  profileForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    const name = profileName.value.trim();
    if (!profileNameOk(name)) return;
    const replacing = profilesNow.list.some((r) => r.name === name);
    profileSave.disabled = true;
    try {
      showProfiles((await call("profile_save", { name, from: agentPick.value })) as Profiles);
      setText(profileNote, `${replacing ? "Replaced" : "Saved"} ${name} from ${AGENTS[agentPick.value]}'s settings: its apps off, every tool off and the start policies.`);
      profileName.value = "";
    } catch (err) {
      setText(profileNote, `Could not save: ${String(err)}`);
      profileSave.disabled = false;
    }
  });
  profileImport.addEventListener("click", () => profileFile.click());
  profileFile.addEventListener("change", async () => {
    const file = profileFile.files?.[0];
    profileFile.value = "";
    if (!file) return;
    if (file.size > 256 * 1024) {
      setText(profileNote, `${file.name} is larger than a profile can be (256 KB).`);
      return;
    }
    try {
      const done = (await call("profile_import", { text: await file.text() })) as { name: string; replaced: boolean; dropped: string[]; profiles: Profiles };
      showProfiles(done.profiles);
      setText(profileNote, `${done.replaced ? "Replaced" : "Imported"} ${done.name}${done.dropped.length ? `; left out what this formMCP does not know: ${done.dropped.join(", ")}` : ""}.`);
    } catch (err) {
      setText(profileNote, `Could not import ${file.name}: ${String(err)}`);
    }
  });
  for (const pick of document.querySelectorAll<HTMLSelectElement>("[data-start]")) {
    pick.addEventListener("change", async () => {
      const id = pick.dataset.start ?? "";
      pick.disabled = true;
      try {
        pick.value = (await call("set_start", { id, policy: pick.value })) as string;
        const name = family.apps.find((a) => a.id === id)?.name ?? id;
        setText(status, pick.value === "demand" ? `${name} starts when an agent needs it and stops after 10 idle minutes.` : pick.value === "session" ? `${name} starts when an agent's session starts.` : `${name} starts when first needed and keeps running while formMCP runs.`);
      } catch (e) {
        setText(status, `Could not save: ${String(e)}`);
      } finally {
        pick.disabled = false;
      }
    });
  }
  for (const sw of document.querySelectorAll<HTMLButtonElement>("[data-for]")) {
    sw.addEventListener("click", async () => {
      const id = sw.dataset.for ?? "";
      const agent = agentPick.value;
      const on = sw.getAttribute("aria-checked") !== "true";
      sw.disabled = true;
      try {
        const now = (await call("set_for", { client: agent, id, on })) as boolean;
        const set = offFor.get(id) ?? new Set<string>();
        if (now) set.delete(agent);
        else set.add(agent);
        offFor.set(id, set);
        if (agentPick.value === agent) sw.setAttribute("aria-checked", String(now));
        setText(status, `${family.apps.find((a) => a.id === id)?.name ?? id} is ${now ? "offered to" : "turned off for"} ${AGENTS[agent]} from its next request.`);
      } catch (e) {
        setText(status, `Could not save: ${String(e)}`);
      } finally {
        sw.disabled = false;
      }
    });
  }
  const renderTools = (panel: HTMLElement, id: string, list: ToolList, note = ""): void => {
    const name = family.apps.find((a) => a.id === id)?.name ?? id;
    if (!list.tools.length) {
      const load = el("button", "btn btn-small", "Load tools");
      load.type = "button";
      load.addEventListener("click", async () => {
        load.disabled = true;
        setText(load, "Starting the app");
        try {
          renderTools(panel, id, (await call("tool_load", { id })) as ToolList);
        } catch (e) {
          renderTools(panel, id, list, `Could not start ${name}: ${String(e)}`);
        }
      });
      const p = el("p", "tool-note", note || `formMCP has not started ${name} yet. Load tools starts it once to list them; agents see the same list.`);
      panel.replaceChildren(p, load);
      return;
    }
    const off = list.tools.filter((t) => !t.on).length;
    const head = el("p", "tool-note", `${list.tools.length} tools${off ? `, ${off} off` : ""}. Off tools are left out of app_tools and refused in call_app, for every agent.`);
    const ul = el("ul", "tool-list");
    for (const t of list.tools) {
      const li = el("li", "tool-row");
      const sw = makeSwitch(t.name, t.description || t.name);
      sw.classList.add("tool-switch");
      sw.setAttribute("aria-checked", String(t.on));
      sw.disabled = false;
      (sw.querySelector(".switch-label") as HTMLElement).translate = false;
      sw.addEventListener("click", async () => {
        const on = sw.getAttribute("aria-checked") !== "true";
        sw.disabled = true;
        try {
          const now = (await call("set_tool", { id, tool: t.name, on })) as boolean;
          sw.setAttribute("aria-checked", String(now));
          t.on = now;
          const offNow = list.tools.filter((x) => !x.on).length;
          setText(head, `${list.tools.length} tools${offNow ? `, ${offNow} off` : ""}. Off tools are left out of app_tools and refused in call_app, for every agent.`);
        } catch (e) {
          setText(status, `Could not save: ${String(e)}`);
        } finally {
          sw.disabled = false;
        }
      });
      li.append(sw);
      if (t.read_only) li.append(el("span", "tool-ro", "reads only"));
      ul.append(li);
    }
    panel.replaceChildren(head, ul);
  };
  for (const tb of document.querySelectorAll<HTMLButtonElement>("[data-tools]")) {
    tb.addEventListener("click", async () => {
      const id = tb.dataset.tools ?? "";
      const panel = document.querySelector<HTMLElement>(`[data-panel="${id}"]`);
      if (!panel) return;
      const open = panel.hidden;
      tb.setAttribute("aria-expanded", String(open));
      panel.hidden = !open;
      if (!open) return;
      panel.replaceChildren(el("p", "tool-note", "Reading its tools"));
      try {
        renderTools(panel, id, (await call("tool_list", { id })) as ToolList);
      } catch (e) {
        panel.replaceChildren(el("p", "tool-note", `Could not read its tools: ${String(e)}`));
      }
    });
  }
  const copy = async (text: string): Promise<boolean> => {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      const area = el("textarea", "copy-area");
      area.value = text;
      document.body.append(area);
      area.select();
      const done = document.execCommand("copy");
      area.remove();
      return done;
    }
  };
  httpToggle.addEventListener("click", async () => {
    const on = !(httpNow?.running ?? false);
    httpToggle.disabled = true;
    try {
      showHttp((await call("http_switch", { on })) as HttpState);
      setText(status, on ? "formMCP answers over HTTP on this computer while this window is open." : "HTTP stopped.");
    } catch (e) {
      httpToggle.disabled = false;
      setText(status, `Could not ${on ? "start" : "stop"} HTTP: ${String(e)}`);
    }
  });
  httpCopy.addEventListener("click", async () => {
    try {
      const token = (await call("http_token")) as string;
      const address = httpNow?.address ?? "http://127.0.0.1:5625/mcp";
      const done = await copy(`${address}\nAuthorization: Bearer ${token}\n\nclaude mcp add --scope user --transport http formmcp ${address} --header "Authorization: Bearer ${token}"`);
      setText(status, done ? "Copied the address, the header and a Claude Code command. Keep the token private." : "Could not copy here; run formmcp-cli http token.");
    } catch (e) {
      setText(status, `Could not read the token: ${String(e)}`);
    }
  });
  let rotateArmed = 0;
  httpRotate.addEventListener("click", async () => {
    if (Date.now() > rotateArmed) {
      rotateArmed = Date.now() + 4000;
      setText(status, "Click New token again within 4 s: agents set up with the old token stop connecting until they get the new one.");
      return;
    }
    rotateArmed = 0;
    try {
      await call("http_rotate");
      setText(status, "New token made; the old one no longer works. Copy setup gives the new one.");
    } catch (e) {
      setText(status, `Could not make a new token: ${String(e)}`);
    }
  });
  window.addEventListener("focus", () => {
    if (Date.now() - lastSnapshot > 10000) void refresh();
  });
  void refresh();
  void pollLog();
  window.setInterval(() => void pollLog(), 2000);
} else {
  setText(status, "Preview in a browser: open the formMCP app to see this computer's installs, clients and activity.");
  for (const card of document.querySelectorAll(".client")) setText(card.querySelector("[data-state]"), "Not known here");
  setText(httpState, "Not known here");
  agentPick.disabled = true;
}

document.documentElement.classList.remove("booting");
