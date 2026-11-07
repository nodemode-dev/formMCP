import "../../../demo/src/shim";
import { events, remember, settings, type LogEvent } from "../../../demo/src/gateway";
import { cloneInstalls, commit, evaluate, installOf, type Ctx, type Spec } from "./engine";

type Args = Record<string, unknown>;
type Core = { invoke: (cmd: string, args?: Args) => Promise<unknown> };

const core = (window as unknown as { __TAURI__: { core: Core } }).__TAURI__.core;
const original = core.invoke;

const MINUTE = 60000;

const KEPT_TOKEN = ["fm", "kept", "demo", "4471", "wxyz"].join("-");
const KEPT_TOKEN_2 = ["enc", "kept", "demo", "9082", "mnop"].join("-");
const FMD_TOKEN = ["fmd", "demo", "0123456789abcdefghij"].join("_");
const ANT_KEY = ["sk", "ant", "api03", "abcdefghijklmnopqrstuvwxyz0123"].join("-");
const BEARER = ["Bearer", "abcdefghijklmnopqrstuvwxyz012345"].join(" ");

export type Recorded = { at: number; spec: Spec; story: string };

export const RECORDING: Recorded[] = [
  { at: -50, story: "Claude Code asks which Form apps exist", spec: { client: "claude", name: "family_apps", args: { mcp_only: true } } },
  { at: -49, story: "Claude Code asks which app fits a task", spec: { client: "claude", name: "suggest_app", args: { task: "cut the silences out of this interview video", limit: 3 } } },
  { at: -48, story: "Find scenes in a long clip", spec: { client: "claude", name: "call_app", app: "formcut", tool: "formcut_find_scenes", args: { path: "interview.mov" }, result: { simulated: true, scenes: 14, longest_s: 41.2 } } },
  { at: -46, story: "Search pages; the app echoes a stored sync token", spec: { client: "claude", name: "call_app", app: "formmd", tool: "search_pages", args: { query: "release notes" }, result: { simulated: true, pages: ["Release notes 0.31", "Release checklist"], note: `sync token ${KEPT_TOKEN}` } } },
  { at: -44, story: "Codex tries to delete a page", spec: { client: "codex", name: "call_app", app: "formmd", tool: "delete_page", args: { page: "Old drafts" } } },
  { at: -43, story: "Codex opens a PDF from a private key folder", spec: { client: "codex", name: "call_app", app: "formpdf", tool: "open_pdf", args: { path: "/home/sam/.ssh/id_rsa" } } },
  { at: -40, story: "Cursor over HTTP asks FormVFX for its status", spec: { client: "cursor", transport: "http", name: "call_app", app: "formvfx", tool: "get_status", args: {} } },
  { at: -37, story: "Queue an encode; the app's log line holds an API key", spec: { client: "claude", name: "call_app", app: "formencoder", tool: "encode", args: { input: "interview.mov", preset: "web-1080p" }, result: { simulated: true, queued: 1, log: `uploader auth ${ANT_KEY}` } } },
  { at: -34, story: "Cursor applies noise reduction; the reply carries a header", spec: { client: "cursor", transport: "http", name: "call_app", app: "formwav", tool: "formwav_apply_effect", args: { effect: "noise reduction", amount: 0.4 }, result: { simulated: true, applied: "noise reduction", debug: `Authorization: ${BEARER}` } } },
  { at: -30, story: "Export the open picture", spec: { client: "claude", name: "call_app", app: "formvis", tool: "export_image", args: { format: "png" }, result: { simulated: true, saved: "poster.png", bytes: 1840233 } } },
  { at: -26, story: "Codex reads FormLit's status", spec: { client: "codex", name: "call_app", app: "formlit", tool: "get_status", args: {} } },
  { at: -22, story: "Redact a name from a PDF", spec: { client: "claude", name: "call_app", app: "formpdf", tool: "redact", args: { pattern: "Alex Morgan" }, result: { simulated: true, matches: 6, pages: [2, 3, 7] } } },
];

export const SAMPLE_TOKENS = { bearer: BEARER, fmd: FMD_TOKEN, ant: ANT_KEY, kept: KEPT_TOKEN };

export const ctx: Ctx = { installs: cloneInstalls(), running: new Set<string>() };

const CLIENT_NAME: Record<string, string> = { claude: "claude-code", codex: "codex-mcp-client", cursor: "cursor-vscode" };

export const recordedLabel: string[] = [];

function emitChange(): void {
  queueMicrotask(() => document.dispatchEvent(new Event("safety-change")));
}

function checked(): { app: string; version: string; files: number; at: number }[] {
  const rows = ctx.installs.map((i) => ({ app: i.id, version: i.version, files: i.files, at: i.recorded, bad: i.changed.length > 0 }));
  rows.sort((a, b) => Number(b.bad) - Number(a.bad));
  return rows.map(({ app, version, files, at }) => ({ app, version, files, at }));
}

const MUTATING = new Set(["safety_switch", "secret_set", "secret_remove", "hook_set", "telemetry_set", "verify_trust"]);

const patched = async (cmd: string, args: Args = {}): Promise<unknown> => {
  if (cmd === "verify_trust") {
    const inst = installOf(ctx.installs, String(args.app));
    if (inst) inst.changed = [];
  }
  const out = await original(cmd, args);
  let result = out;
  if (cmd === "snapshot") {
    const snap = out as { apps: Record<string, unknown>[]; formportal: boolean };
    snap.apps = snap.apps.map((a) => {
      const inst = installOf(ctx.installs, String(a.id));
      return inst ? { ...a, installed: true, reach: "formportal", version: inst.version } : a;
    });
    snap.formportal = true;
  }
  if (cmd === "safety" || MUTATING.has(cmd)) result = { ...(out as Record<string, unknown>), checked: checked() };
  if (MUTATING.has(cmd)) emitChange();
  return result;
};

core.invoke = patched;

settings.redact = true;
settings.verify = true;
settings.hooks.before = "python3 hooks/guard.py";
settings.hooks.after = "python3 hooks/audit.py";
settings.telemetry = "http://127.0.0.1:4318";
settings.start.formcut = "session";
settings.secrets = [
  { app: "formmd", name: "FORMMD_MCP_TOKEN" },
  { app: "formencoder", name: "FORMENCODER_UPLOAD_KEY" },
];
remember(KEPT_TOKEN);
remember(KEPT_TOKEN_2);

function seed(): void {
  for (const c of ["claude", "codex", "cursor"]) void original("connect", { client: c, on: true });
  void original("http_switch", { on: true });
  events.length = 0;
  const now = Date.now();
  const seen = new Set<string>();
  const side = (minutes: number, row: { event: string; [key: string]: unknown }): void => {
    const e: LogEvent = { ...row, t: now + minutes * MINUTE };
    events.push(e);
  };
  RECORDING.forEach((r, i) => {
    const key = `${r.spec.client}:${r.spec.transport ?? "stdio"}`;
    if (!seen.has(key)) {
      seen.add(key);
      side(r.at - 0.2, { event: "session", client: CLIENT_NAME[r.spec.client], as: r.spec.client, transport: r.spec.transport ?? "stdio" });
    }
    if (i === 5) side(r.at + 1, { event: "http-refused", reason: "origin is not this computer", status: 403 });
    if (i === 8) side(r.at + 1, { event: "http-refused", reason: "missing or wrong access token", status: 401 });
    const trace = evaluate(r.spec, ctx);
    recordedLabel.push(trace.label);
    commit(trace, now + r.at * MINUTE, `rec${i}`);
  });
  events.sort((a, b) => a.t - b.t);
}

seed();
