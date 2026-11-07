import "../../../demo/src/shim";
import { events, family, loaded, record, settings, type LogEvent } from "../../../demo/src/gateway";
import { REACHABLE } from "./agent";
import { PORT, address, newToken, server } from "./httpd";
import { changed, connect, guessPlatform, reset, stateOf, type ClientKey } from "./world";

type Args = Record<string, unknown>;
type Core = { invoke: (cmd: string, args?: Args) => Promise<unknown> };

const core = (window as unknown as { __TAURI__: { core: Core } }).__TAURI__.core;
const original = core.invoke;

const MINUTE = 60000;

function seedProfiles(): void {
  settings.profiles["Video crew"] = {
    apps_off: ["formdesign", "formmd", "formpath", "formpdf", "formpromo", "formranker", "formref", "formsculpt", "formslide", "formsurface", "formweb"],
    hidden_tools: { formencoder: ["family"], formwav: ["formwav_export_media"] },
    start: {},
  };
  settings.profiles["Read only review"] = {
    apps_off: [],
    hidden_tools: {
      formcut: ["formcut_import_media", "formcut_add_clip", "formcut_music", "formcut_render"],
      formwav: ["formwav_apply_effect", "formwav_export_media"],
      formencoder: ["encode", "family"],
      formvis: ["add_layer", "set_layer", "apply_filter", "export_image"],
    },
    start: { formai: "session" },
  };
  settings.use_profile.claude = "Video crew";
  settings.use_profile.codex = "Read only review";
  settings.off.cursor = ["formsculpt", "formsurface"];
}

function seedLog(): void {
  const now = Date.now();
  const at = (minutes: number, rest: { event: string; [key: string]: unknown }): void => {
    const row: LogEvent = { ...rest, t: now - minutes * MINUTE };
    events.push(row);
  };
  at(41, { event: "session", client: "claude-code", client_version: "2.1.0", as: "claude", transport: "stdio", profile: "Video crew" });
  at(41, { event: "call", name: "suggest_app", app: null, tool: null, ok: true, ms: 3, as: "claude", transport: "stdio" });
  at(40, { event: "app-start", app: "formcut", via: "formportal", ok: true, tools: 7, ms: 412, policy: "demand", verified: "match" });
  at(40, { event: "call", name: "app_tools", app: "formcut", tool: null, ok: true, ms: 418, as: "claude", transport: "stdio" });
  at(39, { event: "call", name: "call_app", app: "formcut", tool: "formcut_find_scenes", ok: true, ms: 1380, as: "claude", transport: "stdio" });
  at(33, { event: "session", client: "codex-mcp-client", as: "codex", transport: "stdio", profile: "Read only review" });
  at(33, { event: "call", name: "family_apps", app: null, tool: null, ok: true, ms: 2, as: "codex", transport: "stdio" });
  at(32, { event: "call", name: "call_app", app: "formvis", tool: "add_layer", ok: false, ms: 1, as: "codex", transport: "stdio" });
  at(18, { event: "http-start", port: PORT });
  at(17, { event: "http-refused", reason: "origin is not this computer", status: 403 });
  at(16, { event: "http-refused", reason: "missing or wrong access token", status: 401 });
  at(15, { event: "session", client: "cursor-vscode", as: "cursor", transport: "http" });
  at(15, { event: "call", name: "family_apps", app: null, tool: null, ok: true, ms: 2, as: "cursor", transport: "http" });
  at(14, { event: "call", name: "suggest_app", app: null, tool: null, ok: true, ms: 3, as: "cursor", transport: "http" });
}

function clientRow(client: ClientKey): unknown {
  const s = stateOf(client);
  return { client, state: s.state, note: s.note };
}

const patched = async (cmd: string, args: Args = {}): Promise<unknown> => {
  let out: unknown;
  if (cmd === "connect") {
    const client = String(args.client) as ClientKey;
    const s = connect(client, !!args.on);
    out = { client, state: s.state, note: s.note };
  } else if (cmd === "http_switch") {
    server.running = !!args.on;
    if (!server.token) server.token = newToken();
    if (!server.running) server.sessions.clear();
    record(server.running ? "http-start" : "http-stop", { port: PORT });
    out = { running: server.running, port: PORT, address: address() };
  } else if (cmd === "http_token") {
    if (!server.token) server.token = newToken();
    out = server.token;
  } else if (cmd === "http_rotate") {
    server.token = newToken();
    server.sessions.clear();
    out = server.token;
  } else {
    out = await original(cmd, args);
    if (cmd === "snapshot") {
      const snap = out as { apps: Record<string, unknown>[]; clients: unknown[]; http: unknown; formportal: boolean; source: string };
      snap.apps = snap.apps.map((a) => (REACHABLE.includes(String(a.id)) ? { ...a, installed: true, reach: "formportal", version: "simulated" } : a));
      snap.clients = (["claude", "codex", "cursor"] as ClientKey[]).map(clientRow);
      snap.http = { running: server.running, port: PORT, address: address() };
      snap.formportal = true;
    }
  }
  if (cmd !== "snapshot" && cmd !== "log_tail" && cmd !== "http_token") queueMicrotask(changed);
  return out;
};

core.invoke = patched;

const platform = guessPlatform();
reset(platform);
server.token = newToken();
server.running = true;
seedProfiles();
for (const id of REACHABLE) if (family.apps.some((a) => a.id === id)) loaded.add(id);
seedLog();
