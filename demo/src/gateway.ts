import family from "../../core/data/family.json";

export type App = (typeof family.apps)[number];
export type Profile = { apps_off: string[]; hidden_tools: Record<string, string[]>; start: Record<string, string> };
export type LogEvent = { t: number; event: string; [key: string]: unknown };

export const CLIENT_KEYS = ["claude", "codex", "cursor", "other"] as const;
export const START_POLICIES = ["demand", "session", "keep"];
export const PROFILE_FORMAT = "formmcp-profile";
export const PROFILE_LIMIT = 256 * 1024;
export const MARK = "[redacted by formMCP]";
export const PORTAL_ID = "formai";
export const PORTAL_PAGE = "https://github.com/nodemode-dev/formportal";

type Stored = {
  expose: string[];
  off: Record<string, string[]>;
  hidden_tools: Record<string, string[]>;
  start: Record<string, string>;
  profiles: Record<string, Profile>;
  use_profile: Record<string, string>;
  secrets: { app: string; name: string }[];
  redact: boolean;
  verify: boolean;
  hooks: { before: string; after: string };
  telemetry: string | null;
};

const KEY = "formmcp-demo-settings";

function fresh(): Stored {
  return { expose: [], off: {}, hidden_tools: {}, start: {}, profiles: {}, use_profile: {}, secrets: [], redact: true, verify: true, hooks: { before: "", after: "" }, telemetry: null };
}

function load(): Stored {
  try {
    const raw = window.localStorage.getItem(KEY);
    if (raw) return { ...fresh(), ...(JSON.parse(raw) as Partial<Stored>) };
  } catch {
    return fresh();
  }
  return fresh();
}

export const settings: Stored = load();
export const known: string[] = [];
export const events: LogEvent[] = [];
export const live = { answerWithSamples: true };

export function save(): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(settings));
  } catch {
    return;
  }
}

export function record(event: string, fields: Record<string, unknown> = {}): void {
  events.push({ t: Date.now(), event, ...fields });
}

export function appOf(name: string): App | undefined {
  const wanted = name.trim().toLowerCase();
  return family.apps.find((a) => a.id === wanted || a.app.toLowerCase() === wanted || a.name.toLowerCase() === wanted);
}

export function profileOf(client: string): Profile | undefined {
  const name = settings.use_profile[client];
  return name ? settings.profiles[name] : undefined;
}

export function profileHidesApp(client: string, id: string): boolean {
  return !!profileOf(client)?.apps_off.includes(id);
}

export function isOff(client: string, id: string): boolean {
  return !!settings.off[client]?.includes(id) || profileHidesApp(client, id);
}

export function toolHidden(client: string, id: string, tool: string): boolean {
  return !!settings.hidden_tools[id]?.includes(tool) || !!profileOf(client)?.hidden_tools[id]?.includes(tool);
}

export function profileNameOk(name: string): boolean {
  return /^[A-Za-z0-9_-]([A-Za-z0-9 _-]{0,38}[A-Za-z0-9_-])?$/.test(name);
}

export function profileFrom(client: string): Profile {
  return { apps_off: [...(settings.off[client] ?? [])], hidden_tools: structuredClone(settings.hidden_tools), start: { ...settings.start } };
}

export function readProfile(text: string): { name: string; profile: Profile; dropped: string[] } {
  if (text.length > PROFILE_LIMIT) throw `a profile file is at most ${PROFILE_LIMIT} bytes`;
  let file: { format?: unknown; format_version?: unknown; name?: unknown; apps_off?: unknown; hidden_tools?: unknown; start?: unknown };
  try {
    file = JSON.parse(text);
  } catch (e) {
    throw `not a formMCP profile (${String(e)})`;
  }
  if (!file || typeof file !== "object" || file.format !== PROFILE_FORMAT || file.format_version !== 1) throw `not a formMCP profile (format ${JSON.stringify(file?.format)} version ${JSON.stringify(file?.format_version)})`;
  const name = typeof file.name === "string" ? file.name : "";
  if (!profileNameOk(name)) throw `the profile name ${JSON.stringify(name)} is not allowed: 1 to 40 letters, digits, spaces, hyphens or underscores`;
  const dropped: string[] = [];
  const knownApp = (id: string): boolean => family.apps.some((a) => a.id === id);
  const appsOff = (Array.isArray(file.apps_off) ? file.apps_off : []).filter((a): a is string => typeof a === "string");
  const profile: Profile = { apps_off: [], hidden_tools: {}, start: {} };
  for (const a of appsOff) {
    if (knownApp(a)) profile.apps_off.push(a);
    else dropped.push(a);
  }
  const hidden = file.hidden_tools && typeof file.hidden_tools === "object" ? (file.hidden_tools as Record<string, unknown>) : {};
  for (const [app, tools] of Object.entries(hidden)) {
    const list = (Array.isArray(tools) ? tools : []).filter((t): t is string => typeof t === "string" && t.length > 0 && t.length <= 128);
    if (knownApp(app) && list.length) profile.hidden_tools[app] = list;
    else dropped.push(app);
  }
  const start = file.start && typeof file.start === "object" ? (file.start as Record<string, unknown>) : {};
  for (const [app, policy] of Object.entries(start)) {
    if (knownApp(app) && typeof policy === "string" && START_POLICIES.includes(policy)) profile.start[app] = policy;
    else dropped.push(`${app} (${String(policy)})`);
  }
  profile.apps_off = [...new Set(profile.apps_off)].sort();
  return { name, profile, dropped };
}

export function writeProfile(name: string, profile: Profile): string {
  return `${JSON.stringify({ format: PROFILE_FORMAT, format_version: 1, name, ...profile }, null, 2)}\n`;
}

const STOP = new Set(["a", "an", "the", "and", "or", "to", "of", "in", "on", "for", "with", "my", "me", "i", "it", "this", "that", "is", "are", "be", "can", "you", "your", "some", "from", "into", "by", "at", "as", "do", "make", "want", "need", "please", "help", "new", "using", "use", "app", "apps", "file", "files", "form", "out", "own", "machine", "one", "all", "every", "more", "up", "get", "like", "something", "thing", "things", "work", "way"]);

const KIND_WORDS: Record<string, string[]> = {
  image: ["photo", "picture", "image", "edit", "retouch", "layer", "mask", "background", "remove", "cutout", "crop", "resize", "thumbnail", "poster", "blur", "png", "jpeg", "psd"],
  photo: ["photo", "raw", "develop", "library", "grade", "exposure", "lightroom", "camera"],
  video: ["video", "cut", "clip", "trim", "footage", "caption", "subtitle", "reel", "timeline", "film", "edit", "mp4"],
  audio: ["audio", "sound", "mix", "podcast", "music", "voice", "record", "wav", "mp3", "noise", "master"],
  motion: ["motion", "animate", "animation", "effects", "vfx", "composite", "compositing", "title", "keyframe"],
  encoder: ["convert", "compress", "transcode", "export", "batch", "encode", "format"],
  layout: ["layout", "page", "magazine", "flyer", "poster", "print", "brochure", "book", "publish", "pdf"],
  vector: ["vector", "illustration", "logo", "svg", "path", "icon", "draw", "artboard", "shape"],
  pdf: ["pdf", "sign", "form", "annotate", "fill", "comment", "redact"],
  notes: ["note", "notes", "doc", "docs", "wiki", "database", "markdown", "write", "journal", "kanban"],
  document: ["document", "word", "letter", "report", "text", "essay", "docx", "write"],
  slides: ["slides", "slide", "presentation", "deck", "pitch", "keynote", "powerpoint", "talk"],
  spreadsheet: ["spreadsheet", "excel", "table", "formula", "csv", "budget", "sheet", "xlsx"],
  library: ["asset", "assets", "library", "organise", "organize", "catalogue", "catalog", "tag", "collection", "dam"],
  reference: ["reference", "moodboard", "board", "inspiration", "collect", "clipping"],
  sculpt: ["sculpt", "3d", "model", "mesh", "clay", "character"],
  texture: ["texture", "material", "pbr", "3d", "paint", "surface"],
  web: ["web", "website", "site", "html", "landing", "page", "blog"],
  chat: ["chat", "agent", "claude", "conversation", "assistant", "artifact"],
  portal: ["install", "update", "uninstall", "download", "rollback", "version"],
  launcher: ["launcher", "dock", "switch", "switcher", "launch"],
};

function words(text: string): string[] {
  return text.toLowerCase().split(/[^\p{L}\p{N}]+/u).filter((w) => w && !STOP.has(w));
}

function close(a: string, b: string): boolean {
  if (a === b) return true;
  const stem = (w: string): string => Array.from(w).slice(0, 5).join("");
  return a.length >= 4 && b.length >= 4 && stem(a) === stem(b);
}

function score(task: string[], app: App): number {
  const hay = words(`${app.name} ${app.id} ${app.kind} ${app.summary}`);
  const extra = KIND_WORDS[app.kind];
  if (extra) hay.push(...extra);
  let total = 0;
  for (const word of task) {
    if (word === app.id || word === app.name.toLowerCase() || word === app.app.toLowerCase()) total += 5;
    else if (word === app.kind) total += 2;
    else if (hay.some((h) => close(word, h))) total += 1;
  }
  return total;
}

export function nextStep(app: App): string {
  if (!app.mcp) return `${app.name} has no MCP server yet, so an agent cannot drive it. Install ${app.name} with FormPortal.`;
  if (!app.released) return `${app.name} is not released yet (${app.state}).`;
  return `Install FormPortal first (${PORTAL_PAGE}), then install ${app.name} with it. formMCP never installs.`;
}

export type Suggestion = { id: string; name: string; kind: string; summary: string; score: number; mcp: boolean; installed: boolean; reach: null; state: string; next: string };

export function suggest(client: string, task: string, limit: number): Suggestion[] {
  const taskWords = words(task);
  const aboutInstalling = taskWords.some((w) => ["install", "update", "uninstall", "rollback"].includes(w));
  return family.apps
    .filter((a) => a.id !== PORTAL_ID || aboutInstalling)
    .map((a) => ({ a, s: score(taskWords, a) }))
    .filter((x) => x.s > 0)
    .sort((x, y) => y.s - x.s || Number(y.a.released) - Number(x.a.released) || (x.a.id < y.a.id ? -1 : x.a.id > y.a.id ? 1 : 0))
    .slice(0, 5)
    .map(({ a, s }) => ({ id: a.id, name: a.name, kind: a.kind, summary: a.summary, score: s, mcp: a.mcp, installed: false, reach: null, state: a.state, next: nextStep(a) }))
    .filter((p) => !isOff(client, p.id))
    .slice(0, Math.min(5, Math.max(1, limit)));
}

const SHAPES: RegExp[] = [
  /-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----/g,
  /\bfmd_[A-Za-z0-9_-]{16,}/g,
  /\bsk-ant-[A-Za-z0-9_-]{20,}/g,
  /\bsk-(?:proj-)?[A-Za-z0-9_-]{32,}/g,
  /\bgh[pousr]_[A-Za-z0-9]{30,}/g,
  /\bgithub_pat_[A-Za-z0-9_]{40,}/g,
  /\bxox[abposr]-[A-Za-z0-9-]{10,}/g,
  /\bAKIA[0-9A-Z]{16}\b/g,
  /\bAIza[0-9A-Za-z_-]{35}/g,
  /\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}/g,
  /\b(bearer|token|api[_-]?key|secret|password)(["']?\s*[:=]\s*["']?|\s+)[A-Za-z0-9._~+/-]{20,}=*/gi,
];

export function remember(value: string): void {
  const v = value.trim();
  if (Array.from(v).length < 8 || known.includes(v)) return;
  known.push(v);
  known.sort((a, b) => b.length - a.length);
}

export function forget(): void {
  known.length = 0;
}

export function redact(input: string): { text: string; found: number } {
  let out = input;
  let found = 0;
  for (const k of known) {
    const n = out.split(k).length - 1;
    if (n > 0) {
      found += n;
      out = out.split(k).join(MARK);
    }
  }
  SHAPES.forEach((shape, i) => {
    const n = (out.match(shape) ?? []).length;
    if (!n) return;
    found += n;
    out = i === SHAPES.length - 1 ? out.replace(shape, (_m, a: string, b: string) => `${a}${b}${MARK}`) : out.replace(shape, MARK);
  });
  return { text: out, found };
}

export function redactValue(v: unknown): number {
  if (typeof v === "string" || v === null || typeof v !== "object") return 0;
  let n = 0;
  const holder = v as Record<string, unknown>;
  for (const key of Object.keys(holder)) {
    if (key === "blob" || key === "data") continue;
    const item = holder[key];
    if (typeof item === "string") {
      const r = redact(item);
      if (r.found) {
        holder[key] = r.text;
        n += r.found;
      }
    } else n += redactValue(item);
  }
  return n;
}

export type SampleTool = { name: string; description: string; read_only: boolean };

export function sampleTools(app: App): SampleTool[] {
  return [
    { name: "get_status", description: `Simulated sample: what ${app.name} has open right now.`, read_only: true },
    { name: "list_items", description: `Simulated sample: the ${app.kind} items in the open project.`, read_only: true },
    { name: "open_file", description: `Simulated sample: opens a file in ${app.name}.`, read_only: false },
    { name: "export", description: `Simulated sample: exports the open ${app.kind} work to a file.`, read_only: false },
  ];
}

export const loaded = new Set<string>();

export function familyApps(client: string, mcpOnly: boolean): Record<string, unknown> {
  const shown = family.apps.filter((a) => (!mcpOnly || a.mcp) && !isOff(client, a.id));
  return {
    count: shown.length,
    apps: shown.map((a) => ({ id: a.id, name: a.name, kind: a.kind, summary: a.summary, mcp: a.mcp, state: a.state, installed: false, reach: null })),
    source: `bundled list from ${family.source.repository} ${family.source.version} (${family.source.commit.slice(0, 7)})`,
    turned_off: family.apps.filter((a) => isOff(client, a.id)).length,
    install: "formMCP never installs. FormPortal (id formai) installs, updates and removes the Form apps.",
  };
}

export { family };
