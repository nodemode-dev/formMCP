import { appOf, events, familyApps, family, isOff, profileOf, redactValue, sampleTools, settings, suggest, toolHidden, type App, type LogEvent } from "../../../demo/src/gateway";
import { judge, runLine, shorten, type HookEvent } from "./hooks";

export const CORE_VERSION = "0.8.3";
export const CLIENT_NAMES: Record<string, string> = { claude: "Claude Code", codex: "Codex", cursor: "Cursor", other: "Other agents" };
export const COLLECTOR = "http://127.0.0.1:4318";

export type Install = { id: string; version: string; via: "formportal" | "baseline"; files: number; candidates: string[]; changed: string[]; recorded: number };
export type Spec = { client: string; name: string; app?: string; tool?: string; args: Record<string, unknown>; result?: unknown; transport?: "stdio" | "http" };
export type StepState = "done" | "failed" | "skipped" | "waiting";
export type Step = { title: string; state: StepState; note: string };
export type Kind = "ok" | "refused" | "failed";
export type Span = { name: string; ms: number; ok: boolean; error?: string; attributes: [string, string][]; at: number };
export type Trace = {
  steps: Step[];
  kind: Kind;
  label: string;
  reason: string;
  hidden: number;
  exposed: number;
  hookEvent: HookEvent | null;
  hookSays: string;
  sent: unknown;
  seen: unknown;
  lines: Omit<LogEvent, "t">[];
  span: Span;
  startSpan: Span | null;
  ms: number;
};
export type Ctx = { installs: Install[]; running: Set<string> };

export const STEP_TITLES = ["Agent and app", "Tool switch", "Before-call hook", "File check and start", "The app runs the tool", "After-call hook", "Hide secrets", "Log and trace"];

const day = 24 * 3600;
const earlier = Math.floor(Date.now() / 1000) - 3 * day;

export const installs: Install[] = [
  { id: "formai", version: "0.46.2", via: "formportal", files: 412, candidates: ["resources/portal/app.js"], changed: [], recorded: earlier },
  { id: "formcut", version: "0.58.1", via: "formportal", files: 1894, candidates: ["backend/app/main.py", "frontend/dist/assets/index.js", "resources/engine/render.py"], changed: [], recorded: earlier },
  { id: "formvis", version: "2.0.0", via: "formportal", files: 2310, candidates: ["backend/app/main.py", "frontend/dist/assets/index.js"], changed: [], recorded: earlier },
  { id: "formwav", version: "2.23.1", via: "formportal", files: 1572, candidates: ["backend/app/main.py", "frontend/dist/assets/index.js"], changed: [], recorded: earlier },
  { id: "formmd", version: "0.31.0", via: "formportal", files: 988, candidates: ["web/dist/assets/index.js", "server/main.py"], changed: [], recorded: earlier },
  { id: "formpdf", version: "0.20.0", via: "formportal", files: 1203, candidates: ["backend/app/main.py", "frontend/dist/assets/index.js"], changed: [], recorded: earlier },
  { id: "formvfx", version: "0.53.1", via: "formportal", files: 1746, candidates: ["backend/app/main.py", "frontend/dist/assets/index.js", "native/fvx-core.so"], changed: ["backend/app/main.py (changed)", "frontend/dist/assets/index.js (changed)"], recorded: earlier },
  { id: "formencoder", version: "0.40.2", via: "baseline", files: 233, candidates: ["backend/app/engine.py", "frontend/dist/assets/index.js"], changed: [], recorded: earlier + day },
  { id: "formlit", version: "0.90.0", via: "baseline", files: 1411, candidates: ["backend/app/develop.py", "frontend/dist/assets/index.js"], changed: [], recorded: earlier + day },
];

export function cloneInstalls(): Install[] {
  return installs.map((i) => ({ ...i, changed: [...i.changed] }));
}

export function installOf(list: Install[], id: string): Install | undefined {
  return list.find((i) => i.id === id);
}

const EXTRA: Record<string, [string, string, boolean][]> = {
  formmd: [
    ["search_pages", "searches pages by text", true],
    ["read_page", "reads one page", true],
    ["delete_page", "moves a page to the bin", false],
    ["export_markdown", "writes pages to a folder", false],
  ],
  formpdf: [
    ["get_status", "what FormPDF has open", true],
    ["open_pdf", "opens a PDF", false],
    ["extract_text", "text of a page range", true],
    ["redact", "blacks out matched text", false],
    ["save", "saves the open PDF", false],
  ],
};

export function toolsOf(app: App): { name: string; description: string; read_only: boolean }[] {
  const extra = EXTRA[app.id];
  if (extra) return extra.map(([name, what, read_only]) => ({ name, description: `Simulated sample: ${what}.`, read_only }));
  return sampleTools(app);
}

export function installedApps(): App[] {
  return family.apps.filter((a) => a.mcp && installs.some((i) => i.id === a.id));
}

export function keptSecrets(app: string): number {
  return settings.secrets.filter((s) => s.app === app).length;
}

function hash(s: string): number {
  let h = 11;
  for (const c of s) h = (h * 33 + c.charCodeAt(0)) % 100003;
  return h;
}

const SHOWN = 5;

export function fileProblem(i: Install): string | null {
  if (!i.changed.length) return null;
  const sorted = [...i.changed].sort();
  const more = sorted.length > SHOWN ? ` and ${sorted.length - SHOWN} more` : "";
  const list = sorted.slice(0, SHOWN).join(", ");
  if (i.via === "formportal") return `${i.id} ${i.version} differs from what FormPortal recorded when it installed it: ${list}${more}. formMCP will not start it. Reinstall or repair it with FormPortal`;
  return `${i.id} ${i.version} changed on disk since formMCP first checked it: ${list}${more}. formMCP will not start it. Reinstall it with FormPortal, or if you made the change yourself, run formmcp-cli verify --trust ${i.id}`;
}

function clientLabel(c: string): string {
  return CLIENT_NAMES[c] ?? c;
}

function exampleResult(spec: Spec, tool: string): unknown {
  return { simulated: true, app: spec.app, tool, echoed_arguments: spec.args };
}

function cleanArgs(args: Record<string, unknown>): unknown {
  const copy = structuredClone(args);
  redactValue(copy);
  return shorten(copy);
}

function span(name: string, ms: number, ok: boolean, attributes: [string, string][], at: number, error?: string): Span {
  return { name, ms, ok, error, attributes, at };
}

function blank(): Step[] {
  return STEP_TITLES.map((title) => ({ title, state: "waiting" as StepState, note: "not reached" }));
}

function labelOf(kind: Kind, why: string, hidden: number, exposed: number): string {
  if (kind === "refused") return `refused: ${why}`;
  if (kind === "failed") return `failed: ${why}`;
  if (hidden) return `ok · ${hidden} hidden`;
  if (exposed) return `ok · ${exposed} shown`;
  return "ok";
}

export function evaluate(spec: Spec, ctx: Ctx): Trace {
  const steps = blank();
  const lines: Trace["lines"] = [];
  const transport = spec.transport ?? "stdio";
  const trace: Trace = { steps, kind: "ok", label: "ok", reason: "", hidden: 0, exposed: 0, hookEvent: null, hookSays: "", sent: null, seen: null, lines, span: span("", 0, true, [], 0), startSpan: null, ms: 0 };
  const done = (i: number, note: string): void => {
    steps[i] = { ...steps[i], state: "done", note };
  };
  const skip = (i: number, note: string): void => {
    steps[i] = { ...steps[i], state: "skipped", note };
  };
  const fail = (i: number, note: string): void => {
    steps[i] = { ...steps[i], state: "failed", note };
  };
  const finish = (kind: Kind, why: string, error: string, app: string | null, tool: string | null, hiddenCount: number): Trace => {
    trace.kind = kind;
    trace.reason = error;
    trace.hidden = hiddenCount;
    trace.label = labelOf(kind, why, kind === "ok" && settings.redact ? hiddenCount : 0, kind === "ok" && !settings.redact ? trace.exposed : 0);
    const ms = trace.ms || 2;
    lines.push({ event: "call", name: spec.name, app, tool, ok: kind === "ok", ms, as: spec.client, transport, ...(hiddenCount > 0 ? { redacted: hiddenCount } : {}) });
    const attrs: [string, string][] = [["formmcp.tool", spec.name], ["formmcp.client", spec.client], ["formmcp.transport", transport]];
    if (app) attrs.push(["formmcp.app", app]);
    if (tool) attrs.push(["formmcp.app_tool", tool]);
    trace.span = span(`tools/call ${spec.name}`, ms, kind === "ok", attrs, 0, kind === "ok" ? undefined : "tool error");
    if (kind === "ok") done(7, `recorded; span ${trace.span.name}`);
    else done(7, `recorded as failed; span ${trace.span.name} with status error`);
    return trace;
  };
  const answer = (value: unknown): number => {
    trace.sent = value;
    const copy = structuredClone(value);
    const found = redactValue({ v: copy });
    const seen = structuredClone(value);
    if (settings.redact) redactValue({ v: seen });
    trace.seen = settings.redact ? seen : value;
    trace.exposed = settings.redact ? 0 : found;
    return settings.redact ? found : 0;
  };

  if (spec.name === "family_apps" || spec.name === "suggest_app") {
    done(0, "no app named in this call");
    for (const i of [1, 2, 3, 4, 5]) skip(i, "only call_app and app_tools reach an app");
    const value = spec.name === "family_apps" ? familyApps(spec.client, spec.args.mcp_only === true) : { task: spec.args.task, apps: suggest(spec.client, String(spec.args.task ?? ""), Number(spec.args.limit ?? 3)) };
    trace.ms = 1 + (hash(spec.name) % 4);
    const n = answer(value);
    done(6, settings.redact ? (n ? `${n} hidden` : "nothing to hide") : "hiding is off");
    return finish("ok", "", "", null, null, n);
  }

  const app = appOf(String(spec.app ?? ""));
  if (!app) {
    fail(0, "no such app");
    return finish("failed", "no such app", `there is no Form app ${JSON.stringify(spec.app ?? "")}; family_apps lists them`, spec.app ?? null, spec.tool ?? null, 0);
  }
  if (isOff(spec.client, app.id)) {
    const profile = profileOf(spec.client);
    const byProfile = !!profile?.apps_off.includes(app.id);
    const name = byProfile ? settings.use_profile[spec.client] : "";
    fail(0, byProfile ? `off by profile ${name}` : "turned off");
    const why = byProfile ? `${app.name} is turned off for ${clientLabel(spec.client)} by the profile ${JSON.stringify(name)} in formMCP; the person can change that in formMCP's window` : `${app.name} is turned off for ${clientLabel(spec.client)} in formMCP; the person can turn it on in formMCP's window`;
    return finish("refused", "app off", why, app.id, spec.tool ?? null, 0);
  }
  done(0, `${app.name} is on for ${clientLabel(spec.client)}`);

  const toolName = spec.name === "call_app" ? String(spec.tool ?? "") : "";
  if (spec.name === "call_app") {
    if (!toolName) {
      fail(1, "no tool named");
      return finish("failed", "no tool", "tool is required: a name from app_tools", app.id, null, 0);
    }
    if (toolHidden(spec.client, app.id, toolName)) {
      fail(1, "turned off");
      return finish("refused", "tool off", `${toolName} in ${app.name} is turned off in formMCP; the person can turn it on in formMCP's window`, app.id, toolName, 0);
    }
    done(1, `${toolName} is on`);
  } else skip(1, "lists tools; the switches filter the list");

  let hookEvent: HookEvent | null = null;
  if (spec.name === "call_app") {
    hookEvent = { event: "before", app: app.id, tool: toolName, arguments: cleanArgs(spec.args), client: spec.client, transport, formmcp: CORE_VERSION };
    trace.hookEvent = hookEvent;
    const line = settings.hooks.before.trim();
    if (!line) skip(2, "no before hook set");
    else {
      const ran = runLine(line, hookEvent);
      if (ran.error) {
        const msg = `formMCP's before-call hook failed, so the call was not made: ${ran.error}`;
        lines.push({ event: "hook", hook: "before", ok: false, error: ran.error });
        trace.hookSays = ran.error;
        trace.ms = ran.ms;
        fail(2, `failed: ${ran.error}`);
        return finish("refused", "hook error", msg, app.id, toolName, 0);
      }
      const verdict = judge(ran.ran!);
      trace.hookSays = ran.ran!.stdout || ran.ran!.stderr || `(exit ${ran.ran!.code}, no output)`;
      if (!verdict.ok) {
        lines.push({ event: "hook", hook: "before", ok: false, app: app.id, tool: toolName, reason: verdict.reason });
        fail(2, verdict.reason ? `refused: ${verdict.reason}` : "refused");
        trace.ms = ran.ms;
        return finish("refused", "hook", `formMCP's before-call hook refused this call${verdict.reason ? `: ${verdict.reason}` : ""}`, app.id, toolName, 0);
      }
      lines.push({ event: "hook", hook: "before", ok: true, app: app.id, tool: toolName });
      done(2, `allowed in ${ran.ms} ms`);
    }
  } else skip(2, "hooks run for call_app only");

  const policy = settings.start[app.id] ?? "demand";
  const inst = installOf(ctx.installs, app.id);
  if (ctx.running.has(app.id)) done(3, "already running; no new check");
  else if (!inst) {
    fail(3, "not installed on this machine");
    const why = `${app.name} is not installed on this simulated machine, so formMCP cannot start it; FormPortal installs the Form apps`;
    lines.push({ event: "app-start", app: app.id, via: "formportal", ok: false, error: why, stage: "start", ms: 3, policy });
    trace.startSpan = span(`app start ${app.id}`, 3, false, [["formmcp.app", app.id], ["formmcp.via", "formportal"], ["formmcp.policy", policy]], 0, why);
    return finish("failed", "not started", why, app.id, toolName || null, 0);
  } else {
    const problem = settings.verify ? fileProblem(inst) : null;
    if (problem) {
      const why = `${app.name} was not started: ${problem}`;
      fail(3, `refused: ${inst.changed.length} file${inst.changed.length === 1 ? "" : "s"} differ`);
      lines.push({ event: "app-start", app: app.id, via: "formportal", ok: false, error: why, stage: "verify", ms: 40 + (hash(app.id) % 60), policy });
      trace.startSpan = span(`app start ${app.id}`, 40, false, [["formmcp.app", app.id], ["formmcp.via", "formportal"], ["formmcp.policy", policy]], 0, why);
      return finish("refused", "files differ", why, app.id, toolName || null, 0);
    }
    const verified = !settings.verify ? null : inst.via === "formportal" ? "formportal" : "match";
    const tools = toolsOf(app).filter((t) => !toolHidden(spec.client, app.id, t.name)).length;
    const ms = 300 + (hash(app.id) % 500);
    lines.push({ event: "app-start", app: app.id, via: "formportal", ok: true, tools, ms, policy, verified, files: settings.verify ? inst.files : null, secrets: keptSecrets(app.id) });
    trace.startSpan = span(`app start ${app.id}`, ms, true, [["formmcp.app", app.id], ["formmcp.via", "formportal"], ["formmcp.policy", policy]], 0);
    ctx.running.add(app.id);
    done(3, settings.verify ? `files ${inst.via === "formportal" ? "match FormPortal's record" : "match what formMCP recorded"} (${inst.files}); started in ${ms} ms` : `file check is off; started in ${ms} ms`);
  }

  if (spec.name === "app_tools") {
    skip(4, "app_tools lists the tools the app offers");
    skip(5, "hooks run for call_app only");
    const list = toolsOf(app).filter((t) => !toolHidden(spec.client, app.id, t.name));
    trace.ms = 20 + (hash(app.id) % 30);
    const n = answer({ app: app.id, name: app.name, simulated: true, tools: list.map((t) => ({ name: t.name, description: t.description })) });
    done(6, settings.redact ? (n ? `${n} hidden` : "nothing to hide") : "hiding is off");
    return finish("ok", "", "", app.id, null, n);
  }

  const known = toolsOf(app).some((t) => t.name === toolName);
  const ms = 30 + (hash(`${app.id}${toolName}`) % 700);
  trace.ms = ms;
  let ok = true;
  let value: unknown;
  if (!known) {
    ok = false;
    value = { error: `${app.name} has no tool ${JSON.stringify(toolName)}; names come from app_tools` };
    fail(4, `${app.name} answered: unknown tool`);
  } else {
    value = spec.result ?? exampleResult(spec, toolName);
    done(4, `${app.name} answered in ${ms} ms`);
  }

  const after = settings.hooks.after.trim();
  if (!after) skip(5, "no after hook set");
  else {
    const copy = structuredClone(value);
    redactValue({ v: copy });
    const event: HookEvent = { event: "after", app: app.id, tool: toolName, arguments: cleanArgs(spec.args), client: spec.client, transport, formmcp: CORE_VERSION, ok, ms, result: shorten(copy) };
    const ran = runLine(after, event);
    const problem = ran.error ?? (ran.ran && ran.ran.code !== 0 ? `it exited with Some(${ran.ran.code})` : "");
    if (problem) {
      lines.push({ event: "hook", hook: "after", ok: false, error: problem });
      fail(5, `failed: ${problem}; the result is still returned`);
    } else done(5, `ran with the redacted result; its output is ignored`);
  }

  const n = answer(value);
  if (!settings.redact) done(6, trace.exposed ? `hiding is off: ${trace.exposed} secret${trace.exposed === 1 ? "" : "s"} go to the agent` : "hiding is off; nothing secret here");
  else done(6, n ? `${n} hidden in the result` : "nothing to hide");
  return ok ? finish("ok", "", "", app.id, toolName, n) : finish("failed", "app error", `${app.name} has no tool ${JSON.stringify(toolName)}`, app.id, toolName, n);
}

export type Received = { span: Span; traces: unknown; metrics: unknown };
export const collector = { received: [] as Received[], warned: false, problems: [] as string[] };

const BOUNDS = [10, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 30000];

export function endpointProblem(endpoint: string): string | null {
  const e = endpoint.trim();
  const schemeOk = e.startsWith("http://") || e.startsWith("https://");
  const host = e.split("://")[1];
  if (!schemeOk || e.length > 2048 || /\s/.test(e) || !host) return `${JSON.stringify(e)} is not an OpenTelemetry HTTP address; give the collector's base address, for example http://127.0.0.1:4318`;
  return null;
}

export function reachable(endpoint: string): boolean {
  return /^https?:\/\/(127\.0\.0\.1|localhost):4318\/?$/.test(endpoint.trim());
}

function hex(len: number, seed: string): string {
  let x = hash(seed) * 2654435761;
  let out = "";
  for (let i = 0; i < len * 2; i += 1) {
    x = (x * 1103515245 + 12345) % 2147483648;
    out += ((x >> 8) & 15).toString(16);
  }
  return out;
}

export function otlp(s: Span, seed: string): { traces: unknown; metrics: unknown } {
  const nanos = (ms: number): string => `${Math.floor(ms)}000000`;
  const attrs = [...s.attributes, ["formmcp.outcome", s.ok ? "ok" : "error"] as [string, string]];
  const list = attrs.map(([key, v]) => ({ key, value: { stringValue: v } }));
  const resource = { attributes: [{ key: "service.name", value: { stringValue: "formmcp" } }, { key: "service.version", value: { stringValue: CORE_VERSION } }] };
  const scope = { name: "formmcp", version: CORE_VERSION };
  const end = s.at + s.ms;
  const traces = {
    resourceSpans: [
      {
        resource,
        scopeSpans: [
          {
            scope,
            spans: [{ traceId: hex(16, seed), spanId: hex(8, seed + "s"), name: s.name, kind: 2, startTimeUnixNano: nanos(s.at), endTimeUnixNano: nanos(end), attributes: list, status: s.ok ? { code: 1 } : { code: 2, message: s.error ?? "failed" } }],
          },
        ],
      },
    ],
  };
  const slot = BOUNDS.findIndex((b) => s.ms <= b);
  const buckets = BOUNDS.map(() => "0").concat("0");
  buckets[slot === -1 ? BOUNDS.length : slot] = "1";
  const metrics = {
    resourceMetrics: [
      {
        resource,
        scopeMetrics: [
          {
            scope,
            metrics: [
              { name: "formmcp.operations", unit: "1", description: "Tool calls and app starts through formMCP", sum: { aggregationTemporality: 1, isMonotonic: true, dataPoints: [{ attributes: list, startTimeUnixNano: nanos(s.at), timeUnixNano: nanos(end), asInt: "1" }] } },
              { name: "formmcp.duration", unit: "ms", description: "How long each one took", histogram: { aggregationTemporality: 1, dataPoints: [{ attributes: list, startTimeUnixNano: nanos(s.at), timeUnixNano: nanos(end), count: "1", sum: s.ms, bucketCounts: buckets, explicitBounds: BOUNDS }] } },
            ],
          },
        ],
      },
    ],
  };
  return { traces, metrics };
}

export type Sent = { to: string; ok: boolean; error?: string };

export function exportSpan(s: Span, seed: string): Sent | null {
  const endpoint = settings.telemetry;
  if (!endpoint || endpointProblem(endpoint)) return null;
  const to = `${endpoint.replace(/\/+$/, "")}/v1/traces`;
  if (reachable(endpoint)) {
    const body = otlp(s, seed);
    collector.received.push({ span: s, ...body });
    return { to, ok: true };
  }
  const error = `${to}: Connection Failed: Connect error: Connection refused (os error 111)`;
  if (!collector.warned) {
    collector.warned = true;
    events.push({ t: s.at, event: "telemetry", ok: false, error });
    collector.problems.push(error);
  }
  return { to, ok: false, error };
}

export function commit(trace: Trace, at: number, seed: string): void {
  const stamped = (s: Span | null): Span | null => (s ? { ...s, at } : null);
  let n = 0;
  for (const line of trace.lines) {
    const fields: Record<string, unknown> = { ...line };
    redactValue(fields);
    events.push({ t: at + n, event: String(line.event), ...fields });
    n += 1;
  }
  const start = stamped(trace.startSpan);
  if (start) exportSpan(start, `${seed}a`);
  const main = stamped(trace.span);
  if (main) exportSpan(main, seed);
}
