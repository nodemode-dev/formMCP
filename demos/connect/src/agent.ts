import { appOf, family, isOff, live, loaded, profileOf, record, redactValue, sampleTools, settings, suggest, toolHidden, type App } from "../../../demo/src/gateway";

export const REACHABLE = ["formcut", "formwav", "formencoder", "formvis"];
export const PROTOCOL = "2025-06-18";
export const SUPPORTED = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
export const AGENT_LABEL: Record<string, string> = { claude: "Claude Code", codex: "Codex", cursor: "Cursor", other: "other agents" };

export const INSTRUCTIONS =
  "formMCP is the one connection to the Form family of desktop apps (image, video, audio, layout, vector, notes and more, by nodemode.ai). Call suggest_app with the person's task to get the app that fits and the next step, or family_apps for the whole list (whether each app has an MCP server and whether this computer can reach it). For a reachable app, app_tools starts it and lists its own tools, and call_app runs one of them; load one app at a time, only when the task needs it. An app's own resources and prompts, when it offers them, come through app_resources and read_app_resource, app_prompts and get_app_prompt. The person may have turned some apps or tools off for this agent in formMCP's window: they are left out of the lists and refused with the reason. formMCP never installs anything: when an app is missing, tell the person which one fits and that FormPortal installs and updates the Form apps. formMCP does not handle skills: the Form apps' released skills are viewed and downloaded in formskills.";

const obj = (props: Record<string, unknown>, required: string[] = []): unknown => ({ type: "object", properties: props, ...(required.length ? { required } : {}) });
const appArg = { type: "string", description: "An id from family_apps." };
const ro = { readOnlyHint: true, openWorldHint: false };

export const TOOLS: unknown[] = [
  { name: "family_apps", title: "The Form family", description: "Every Form app: id, name, what kind of work it does, one line on what it is for, whether it has an MCP server agents can use, its release state (live, public or prelaunch), whether FormPortal installed it on this computer, and reach: how formMCP can start its server here (dev for a local checkout, formportal for an install, null when it cannot). Use it to pick the app that fits a task. Short on purpose: no app's tools are listed here.", inputSchema: obj({ mcp_only: { type: "boolean", description: "Only the apps that have an MCP server." } }), annotations: ro },
  { name: "suggest_app", title: "Which Form app fits a task", description: "Given the person's task in their own words, returns up to five Form apps that fit, best first, each with whether it has an MCP server, whether it is installed and reachable here, and the next step: call app_tools, install it with FormPortal, install FormPortal first, or why an agent cannot drive it yet. It never installs anything.", inputSchema: obj({ task: { type: "string", description: "What the person wants done, for example 'cut the silences out of this interview video'." }, limit: { type: "integer", minimum: 1, maximum: 5, description: "How many apps at most (3 when left out)." } }, ["task"]), annotations: ro },
  { name: "app_tools", title: "One app's tools", description: "Starts one Form app's own MCP server (or reuses it once started) and returns its tools with their input schemas, its name and version, and its instructions. Call it for the one app the task needs, after family_apps shows it is reachable. Fails with the reason when the app has no MCP server or cannot be reached.", inputSchema: obj({ app: appArg, refresh: { type: "boolean", description: "Ask the app for its tools again." } }, ["app"]), annotations: ro },
  { name: "call_app", title: "Run one of an app's tools", description: "Runs a tool from app_tools in that Form app and returns the app's own result unchanged. What it changes is what the app's tool changes: read that tool's description and annotations first, and confirm with the person before anything that changes their files.", inputSchema: obj({ app: appArg, tool: { type: "string", description: "A tool name from app_tools." }, arguments: { type: "object", description: "The tool's arguments, as its input schema in app_tools describes." } }, ["app", "tool"]), annotations: { readOnlyHint: false, destructiveHint: true, openWorldHint: true } },
  { name: "app_resources", title: "One app's resources", description: "Starts one Form app's MCP server (or reuses it) and lists the resources it offers (documents, project state, references, each with a uri) and its resource templates. Empty, with a note, when the app offers none.", inputSchema: obj({ app: appArg }, ["app"]), annotations: ro },
  { name: "read_app_resource", title: "Read one app resource", description: "Reads one resource by the uri app_resources gave (or one filled in from its templates) and returns its contents as the app sent them.", inputSchema: obj({ app: appArg, uri: { type: "string", description: "The resource's uri." } }, ["app", "uri"]), annotations: ro },
  { name: "app_prompts", title: "One app's prompts", description: "Lists the prompt templates one Form app offers, with their arguments. Empty, with a note, when the app offers none.", inputSchema: obj({ app: appArg }, ["app"]), annotations: ro },
  { name: "get_app_prompt", title: "Fill one app prompt", description: "Fills one prompt from app_prompts with its arguments and returns the app's messages, ready to follow.", inputSchema: obj({ app: appArg, name: { type: "string", description: "A prompt name from app_prompts." }, arguments: { type: "object", description: "The prompt's arguments as strings.", additionalProperties: { type: "string" } } }, ["app", "name"]), annotations: ro },
];

export function clientKey(name: string): string {
  const lower = name.toLowerCase();
  return ["claude", "codex", "cursor"].find((k) => lower.includes(k)) ?? "other";
}

export function reachOf(id: string): string | null {
  return REACHABLE.includes(id) ? "formportal" : null;
}

function nextStep(app: App, via: string | null): string {
  if (via) return `${app.name} can be reached here (${via}): call app_tools with app ${app.id}.`;
  if (!app.mcp) return `${app.name} has no MCP server yet, so an agent cannot drive it. Install ${app.name} with FormPortal.`;
  if (!app.released) return `${app.name} is not released yet (${app.state}).`;
  return `Install ${app.name} with FormPortal (open FormPortal, or its install tool for ${app.id}). formMCP never installs.`;
}

export function sourceText(): string {
  return `bundled list from ${family.source.repository} ${family.source.version} (${family.source.commit.slice(0, 7)})`;
}

function profileMessage(client: string, app: App): string {
  const p = profileOf(client);
  const name = settings.use_profile[client] ?? "";
  if (p && p.apps_off.includes(app.id)) return `${app.name} is turned off for ${AGENT_LABEL[client]} by the profile ${JSON.stringify(name)} in formMCP; the person can change that in formMCP's window`;
  return `${app.name} is turned off for ${AGENT_LABEL[client]} in formMCP; the person can turn it on in formMCP's window`;
}

function pick(client: string, args: Record<string, unknown>): App {
  const name = String(args.app ?? "");
  if (!name.trim()) throw "app is required: an id from family_apps, for example formvis";
  const app = appOf(name);
  if (!app) throw `there is no Form app ${JSON.stringify(name)}; family_apps lists them`;
  if (isOff(client, app.id)) throw profileMessage(client, app);
  return app;
}

function server(app: App): void {
  if (!app.mcp) throw `${app.name} has no MCP server yet, so there are no tools to reach`;
  if (!reachOf(app.id) || !live.answerWithSamples) {
    throw `${app.name} cannot be reached: it is not installed: FormPortal installs it, or set FORMMCP_DEV_ROOTS to a folder holding its checkout`;
  }
  if (!loaded.has(app.id)) {
    loaded.add(app.id);
    record("app-start", { app: app.id, via: "formportal", ok: true, tools: sampleTools(app).length, ms: 240, policy: settings.start[app.id] ?? "demand" });
  }
}

type Mine = { mine: unknown } | { passed: unknown };

function callApp(client: string, args: Record<string, unknown>): Mine {
  const app = pick(client, args);
  const tool = String(args.tool ?? "");
  if (!tool) throw "tool is required: a name from app_tools";
  if (toolHidden(client, app.id, tool)) throw `${tool} in ${app.name} is turned off in formMCP; the person can turn it on in formMCP's window`;
  server(app);
  const known = sampleTools(app).find((t) => t.name === tool);
  const text = JSON.stringify({ app: app.id, tool, ok: !!known, simulated: true, arguments: args.arguments ?? {}, note: known ? "Simulated sample result: the demo has no running app." : `${app.name} has no tool ${tool}` });
  return { passed: { content: [{ type: "text", text }], isError: !known } };
}

function run(client: string, name: string, args: Record<string, unknown>): Mine {
  if (name === "family_apps") {
    const mcpOnly = !!args.mcp_only;
    const shown = family.apps.filter((a) => (!mcpOnly || a.mcp) && !isOff(client, a.id));
    return {
      mine: {
        count: shown.length,
        apps: shown.map((a) => ({ id: a.id, name: a.name, kind: a.kind, summary: a.summary, mcp: a.mcp, state: a.state, installed: !!reachOf(a.id), reach: reachOf(a.id) })),
        source: sourceText(),
        turned_off: family.apps.filter((a) => isOff(client, a.id)).length,
        install: "formMCP never installs. FormPortal (id formai) installs, updates and removes the Form apps.",
      },
    };
  }
  if (name === "suggest_app") {
    const task = String(args.task ?? "").trim();
    if (!task) throw "task is required: what the person wants done, in their words";
    const limit = Number(args.limit ?? 3);
    const apps = suggest(client, task, 5)
      .slice(0, Math.max(1, Math.min(5, limit)))
      .map((p) => {
        const app = appOf(p.id) as App;
        const via = reachOf(p.id);
        return { ...p, installed: !!via, reach: via, next: nextStep(app, via) };
      });
    return { mine: { task, apps, note: apps.length ? "Best first. formMCP never installs: FormPortal installs and updates the Form apps." : "No app matched the words of the task; call family_apps and choose by kind and summary." } };
  }
  if (name === "app_tools") {
    const app = pick(client, args);
    server(app);
    const all = sampleTools(app);
    const hidden = all.filter((t) => toolHidden(client, app.id, t.name)).map((t) => t.name);
    const shown = all.filter((t) => !hidden.includes(t.name)).map((t) => ({ name: t.name, description: t.description, annotations: { readOnlyHint: t.read_only } }));
    return { mine: { app: app.id, via: "formportal", server: { name: app.id, version: "simulated" }, instructions: "", count: shown.length, tools: shown, hidden, offers: { resources: false, prompts: false } } };
  }
  if (name === "call_app") return callApp(client, args);
  if (name === "app_resources" || name === "app_prompts") {
    const app = pick(client, args);
    server(app);
    const what = name === "app_resources" ? "resources" : "prompts";
    return { mine: { app: app.id, via: "formportal", count: 0, [what]: [], ...(what === "resources" ? { templates: [] } : {}), note: `${app.name} offers no ${what}` } };
  }
  if (name === "read_app_resource" || name === "get_app_prompt") {
    const app = pick(client, args);
    server(app);
    throw `${app.name} offers no ${name === "read_app_resource" ? "resources" : "prompts"}`;
  }
  const exposed = name.split("__");
  if (exposed.length === 2 && settings.expose.includes(exposed[0]) && !isOff(client, exposed[0])) return callApp(client, { app: exposed[0], tool: exposed[1], arguments: args });
  throw `no tool named ${JSON.stringify(name)}; tools/list has the tools`;
}

export type Answer = { content: unknown; structuredContent?: unknown; isError: boolean };

export function toolsCall(client: string, transport: string, name: string, args: Record<string, unknown>): Answer {
  const started = performance.now();
  let answer: Answer;
  try {
    const out = run(client, name, args);
    if ("mine" in out) answer = { content: [{ type: "text", text: JSON.stringify(out.mine) }], structuredContent: out.mine, isError: false };
    else answer = out.passed as Answer;
  } catch (e) {
    answer = { content: [{ type: "text", text: String(e) }], isError: true };
  }
  const hidden = settings.redact ? redactValue(answer) : 0;
  const split = name.split("__");
  const app = split.length === 2 ? split[0] : (args.app as string | undefined) ?? null;
  const tool = split.length === 2 ? split[1] : (args.tool as string | undefined) ?? null;
  record("call", { name, app, tool, ok: !answer.isError, ms: Math.max(1, Math.round(performance.now() - started)), as: client, transport, redacted: hidden > 0 ? hidden : undefined });
  return answer;
}

export function initialize(client: string, transport: string, clientName: string, version: string): unknown {
  record("session", { client: clientName, client_version: version, as: client, transport, expose: settings.expose, profile: settings.use_profile[client] ?? null });
  return { protocolVersion: PROTOCOL, capabilities: { tools: { listChanged: settings.expose.length > 0 && transport === "stdio" } }, serverInfo: { name: "formmcp", title: "formMCP", version: "0.9.1" }, instructions: INSTRUCTIONS };
}

export function answer(client: string, transport: string, msg: Record<string, unknown>): Record<string, unknown> | null {
  if (msg.id === undefined || msg.id === null) return null;
  const method = String(msg.method ?? "");
  const params = (msg.params ?? {}) as Record<string, unknown>;
  if (method === "initialize") {
    const info = (params.clientInfo ?? {}) as Record<string, unknown>;
    return { jsonrpc: "2.0", id: msg.id, result: initialize(client, transport, String(info.name ?? ""), String(info.version ?? "")) };
  }
  if (method === "ping") return { jsonrpc: "2.0", id: msg.id, result: {} };
  if (method === "tools/list") return { jsonrpc: "2.0", id: msg.id, result: { tools: TOOLS } };
  if (method === "tools/call") return { jsonrpc: "2.0", id: msg.id, result: toolsCall(client, transport, String(params.name ?? ""), (params.arguments ?? {}) as Record<string, unknown>) };
  if (method === "resources/list") return { jsonrpc: "2.0", id: msg.id, result: { resources: [] } };
  if (method === "prompts/list") return { jsonrpc: "2.0", id: msg.id, result: { prompts: [] } };
  return { jsonrpc: "2.0", id: msg.id, error: { code: -32601, message: `method ${JSON.stringify(method)} is not served by formMCP` } };
}
