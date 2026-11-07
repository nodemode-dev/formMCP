import {
  CLIENT_KEYS,
  START_POLICIES,
  events,
  family,
  isOff,
  loaded,
  profileFrom,
  profileHidesApp,
  profileNameOk,
  readProfile,
  record,
  remember,
  forget,
  sampleTools,
  save,
  settings,
  appOf,
  writeProfile,
  toolHidden,
} from "./gateway";

type Args = Record<string, unknown>;

const HTTP_PORT = 5625;
const http = { running: false, token: "" };

function newToken(): string {
  const bytes = new Uint8Array(24);
  crypto.getRandomValues(bytes);
  return `fmd_demo_${Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("")}`;
}

function httpState(): { running: boolean; port: number; address: string } {
  return { running: http.running, port: HTTP_PORT, address: `http://127.0.0.1:${HTTP_PORT}/mcp` };
}

function profiles(): { list: unknown[]; in_use: Record<string, string> } {
  const list = Object.entries(settings.profiles)
    .sort(([a], [b]) => (a < b ? -1 : 1))
    .map(([name, p]) => ({
      name,
      apps_off: p.apps_off.length,
      tools_off: Object.values(p.hidden_tools).reduce((n, l) => n + l.length, 0),
      starts: Object.keys(p.start).length,
      used_by: Object.entries(settings.use_profile).filter(([, n]) => n === name).map(([c]) => c),
    }));
  return { list, in_use: { ...settings.use_profile } };
}

const connected = new Set<string>();

function clientState(client: string): { client: string; state: string; note: string | null } {
  return { client, state: connected.has(client) ? "connected" : "not-connected", note: "Demo: this only changes the state shown here; no agent settings file exists in a browser." };
}

function snapshot(): unknown {
  const apps = family.apps.map((a) => ({
    id: a.id,
    name: a.name,
    kind: a.kind,
    summary: a.summary,
    mcp: a.mcp,
    state: a.state,
    version: "",
    installed: false,
    reach: null,
    exposed: settings.expose.includes(a.id),
    off_for: CLIENT_KEYS.filter((c) => settings.off[c]?.includes(a.id)),
    profile_off_for: CLIENT_KEYS.filter((c) => profileHidesApp(c, a.id)),
    start: settings.start[a.id] ?? "demand",
    hidden_tools: settings.hidden_tools[a.id]?.length ?? 0,
  }));
  return {
    source: `bundled list from ${family.source.repository} ${family.source.version} (${family.source.commit.slice(0, 7)})`,
    apps,
    clients: ["claude", "codex", "cursor"].map(clientState),
    formportal: false,
    env_expose: false,
    http: httpState(),
    profiles: profiles(),
  };
}

function safety(): unknown {
  return {
    redact: settings.redact,
    verify: settings.verify,
    hooks: { before: settings.hooks.before || undefined, after: settings.hooks.after || undefined },
    telemetry: settings.telemetry,
    secrets: settings.secrets.map((s) => ({ ...s, stored: true, error: null })),
    checked: [],
  };
}

function toolRows(id: string): unknown {
  const app = appOf(id);
  if (!app || !loaded.has(id)) return { app: id, loaded: null, tools: [] };
  return { app: id, loaded: Date.now(), tools: sampleTools(app).map((t) => ({ ...t, on: !settings.hidden_tools[id]?.includes(t.name) })) };
}

function download(name: string, text: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function str(args: Args, key: string): string {
  return String(args[key] ?? "");
}

function client(args: Args): string {
  const c = str(args, "client");
  if (!CLIENT_KEYS.includes(c as (typeof CLIENT_KEYS)[number])) throw `there is no agent ${JSON.stringify(c)}`;
  return c;
}

const handlers: Record<string, (args: Args) => unknown> = {
  snapshot,
  safety,
  connect: (a) => {
    const c = str(a, "client");
    if (a.on) {
      connected.add(c);
      record("session", { client: c, as: c });
    } else connected.delete(c);
    return clientState(c);
  },
  set_expose: (a) => {
    const id = str(a, "id");
    settings.expose = settings.expose.filter((x) => x !== id);
    if (a.on) settings.expose.push(id);
    settings.expose.sort();
    save();
    return [...settings.expose];
  },
  set_start: (a) => {
    const policy = str(a, "policy");
    if (!START_POLICIES.includes(policy)) throw `${JSON.stringify(policy)}: use demand, session or keep`;
    if (policy === "demand") delete settings.start[str(a, "id")];
    else settings.start[str(a, "id")] = policy;
    save();
    return policy;
  },
  set_for: (a) => {
    const c = client(a);
    const id = str(a, "id");
    const list = (settings.off[c] ?? []).filter((x) => x !== id);
    if (!a.on) list.push(id);
    list.sort();
    if (list.length) settings.off[c] = list;
    else delete settings.off[c];
    save();
    return !isOff(c, id);
  },
  tool_list: (a) => toolRows(str(a, "id")),
  tool_load: (a) => {
    const id = str(a, "id");
    loaded.add(id);
    record("app-start", { app: id, via: "demo sample", ok: true, tools: 4, ms: 0 });
    return toolRows(id);
  },
  set_tool: (a) => {
    const id = str(a, "id");
    const tool = str(a, "tool");
    const list = (settings.hidden_tools[id] ?? []).filter((t) => t !== tool);
    if (!a.on) list.push(tool);
    list.sort();
    if (list.length) settings.hidden_tools[id] = list;
    else delete settings.hidden_tools[id];
    save();
    return !toolHidden("other", id, tool);
  },
  profile_save: (a) => {
    const name = str(a, "name");
    if (!profileNameOk(name)) throw `the profile name ${JSON.stringify(name)} is not allowed: 1 to 40 letters, digits, spaces, hyphens or underscores`;
    settings.profiles[name] = profileFrom(client({ client: a.from }));
    save();
    return profiles();
  },
  profile_use: (a) => {
    const c = client(a);
    const name = a.name === null || a.name === undefined ? null : str(a, "name");
    if (name !== null && !settings.profiles[name]) throw `there is no profile ${JSON.stringify(name)}`;
    if (name === null) delete settings.use_profile[c];
    else settings.use_profile[c] = name;
    save();
    return profiles();
  },
  profile_delete: (a) => {
    const name = str(a, "name");
    if (!settings.profiles[name]) throw `there is no profile ${JSON.stringify(name)}`;
    delete settings.profiles[name];
    for (const [c, n] of Object.entries(settings.use_profile)) if (n === name) delete settings.use_profile[c];
    save();
    return profiles();
  },
  profile_export: (a) => {
    const name = str(a, "name");
    const p = settings.profiles[name];
    if (!p) throw `there is no profile ${JSON.stringify(name)}`;
    const file = `formmcp-profile-${name.replace(/[^A-Za-z0-9_-]+/g, "-")}.json`;
    download(file, writeProfile(name, p));
    return file;
  },
  profile_import: (a) => {
    const { name, profile, dropped } = readProfile(str(a, "text"));
    const replaced = !!settings.profiles[name];
    settings.profiles[name] = profile;
    save();
    return { name, replaced, dropped, profiles: profiles() };
  },
  http_switch: (a) => {
    http.running = !!a.on;
    if (!http.token) http.token = newToken();
    record(http.running ? "http-start" : "http-stop", { port: HTTP_PORT });
    return httpState();
  },
  http_token: () => {
    if (!http.token) http.token = newToken();
    return http.token;
  },
  http_rotate: () => {
    http.token = newToken();
    return http.token;
  },
  safety_switch: (a) => {
    if ("redact" in a) settings.redact = !!a.redact;
    if ("verify" in a) settings.verify = !!a.verify;
    save();
    return safety();
  },
  secret_set: (a) => {
    const app = str(a, "app");
    const name = str(a, "name");
    remember(str(a, "value"));
    settings.secrets = settings.secrets.filter((s) => !(s.app === app && s.name === name));
    settings.secrets.push({ app, name });
    save();
    return safety();
  },
  secret_remove: (a) => {
    settings.secrets = settings.secrets.filter((s) => !(s.app === str(a, "app") && s.name === str(a, "name")));
    if (!settings.secrets.length) forget();
    save();
    return safety();
  },
  hook_set: (a) => {
    settings.hooks[str(a, "which") === "after" ? "after" : "before"] = str(a, "line");
    save();
    return safety();
  },
  telemetry_set: (a) => {
    settings.telemetry = a.address ? str(a, "address").trim().replace(/\/+$/, "") : null;
    save();
    return safety();
  },
  verify_trust: () => safety(),
  log_tail: (a) => {
    const from = Number(a.from ?? 0);
    return { events: events.slice(from), next: events.length };
  },
};

const invoke = async (cmd: string, args: Args = {}): Promise<unknown> => {
  const run = handlers[cmd];
  if (!run) throw `the demo has no command ${cmd}`;
  return run(args);
};

(window as unknown as { __TAURI__: unknown }).__TAURI__ = { core: { invoke } };
