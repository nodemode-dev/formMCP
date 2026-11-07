import { events, family, redactValue, settings, type LogEvent } from "../../../demo/src/gateway";
import { CLIENT_NAMES, collector, commit, endpointProblem, evaluate, fileProblem, installOf, toolsOf, STEP_TITLES, type Ctx, type Spec, type Trace } from "./engine";
import { DENY_EXIT, SCRIPTS, TIMEOUT_S, judge, runLine, split, type HookEvent } from "./hooks";
import { RECORDING, SAMPLE_TOKENS, ctx, recordedLabel } from "./scenario";

const ICONS = import.meta.glob("../../../demo/assets/media/icons/*.png", { eager: true, query: "?url", import: "default" }) as Record<string, string>;
const MINUTE = 60000;
const AGENTS = ["claude", "codex", "cursor", "other"];
const CALLS = ["call_app", "app_tools", "family_apps", "suggest_app"];
const pending: Promise<unknown>[] = [];

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls = "", text = ""): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text) node.textContent = text;
  return node;
}

function setText(node: Element, text: string): void {
  if (node.textContent !== text) node.textContent = text;
}

function button(label: string, cls = "btn btn-small"): HTMLButtonElement {
  const b = el("button", cls, label);
  b.type = "button";
  return b;
}

function chip(label: string): HTMLButtonElement {
  const b = el("button", "chip", label);
  b.type = "button";
  b.setAttribute("aria-pressed", "false");
  return b;
}

function textInput(value: string, label: string): HTMLInputElement {
  const i = el("input");
  i.type = "text";
  i.value = value;
  i.spellcheck = false;
  i.setAttribute("aria-label", label);
  return i;
}

function select(options: [string, string][], value: string, label: string): HTMLSelectElement {
  const s = el("select");
  for (const [v, text] of options) {
    const o = el("option", "", text);
    o.value = v;
    s.append(o);
  }
  s.value = value;
  s.setAttribute("aria-label", label);
  return s;
}

function area(value: string, label: string): HTMLTextAreaElement {
  const t = el("textarea", "wb-area");
  t.value = value;
  t.spellcheck = false;
  t.setAttribute("aria-label", label);
  return t;
}

function row(label: string, input: HTMLElement): DocumentFragment {
  const f = document.createDocumentFragment();
  f.append(el("span", "", label), input);
  return f;
}

function download(name: string, text: string, type: string): void {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const a = el("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function clock(t: number): string {
  const d = new Date(t);
  return [d.getHours(), d.getMinutes(), d.getSeconds()].map((n) => String(n).padStart(2, "0")).join(":");
}

function appName(id: string): string {
  return family.apps.find((a) => a.id === id)?.name ?? id;
}

function iconNode(id: string): HTMLElement {
  const url = ICONS[`../../../demo/assets/media/icons/${id}.png`];
  if (url) {
    const img = el("img", "app-icon");
    img.src = url;
    img.width = 18;
    img.height = 18;
    img.alt = "";
    img.decoding = "async";
    pending.push(img.decode().catch(() => undefined));
    return img;
  }
  const mono = el("span", "app-icon app-mono", appName(id).replace(/^Form/i, "").slice(0, 2).toUpperCase());
  mono.setAttribute("aria-hidden", "true");
  return mono;
}

function pretty(v: unknown): string {
  return v === null || v === undefined ? "" : JSON.stringify(v, null, 2);
}

function describe(spec: Spec): string {
  const parts = [CLIENT_NAMES[spec.client] ?? spec.client, spec.name, spec.app, spec.tool].filter(Boolean);
  return parts.join(" · ");
}

function notify(name: string): void {
  document.dispatchEvent(new Event(name));
}

function windowSwitch(id: string, on: boolean): void {
  const sw = document.getElementById(id) as HTMLButtonElement | null;
  if (sw && !sw.disabled && (sw.getAttribute("aria-checked") === "true") !== on) sw.click();
}

function windowHooks(before: string, after: string): void {
  const b = document.getElementById("hook-before") as HTMLInputElement;
  const a = document.getElementById("hook-after") as HTMLInputElement;
  const form = document.getElementById("hook-form") as HTMLFormElement;
  b.value = before;
  a.value = after;
  b.dispatchEvent(new Event("input"));
  a.dispatchEvent(new Event("input"));
  form.requestSubmit();
}

function windowTrace(address: string): void {
  const a = document.getElementById("trace-address") as HTMLInputElement;
  const form = document.getElementById("trace-form") as HTMLFormElement;
  a.value = address;
  a.dispatchEvent(new Event("input"));
  form.requestSubmit();
}

function commandFor(file: string): string {
  return file.endsWith(".py") ? `python3 hooks/${file}` : `sh hooks/${file}`;
}

type Detail = { root: HTMLElement; show: (spec: Spec | null, trace: Trace | null, note?: string) => void };

function buildDetail(sentTitle = "What the app sent", seenTitle = "What the agent sees"): Detail {
  const root = el("div", "wb-col");
  const title = el("div", "wb-title", "Pick a call");
  const verdict = el("p", "wb-verdict", "Nothing selected.");
  const list = el("ol", "wb-steps");
  const stepNodes = STEP_TITLES.map((t) => {
    const li = el("li", "wb-step");
    li.dataset.state = "waiting";
    const dot = el("i");
    const name = el("span", "", t);
    const note = el("span", "wb-step-note", "not reached");
    li.append(dot, name, note);
    list.append(li);
    return { li, note };
  });
  const pair = el("div", "wb-pair");
  const sentCol = el("div", "wb-col");
  const seenCol = el("div", "wb-col");
  const sentTitleNode = el("div", "wb-title", sentTitle);
  const seenTitleNode = el("div", "wb-title", seenTitle);
  const sent = el("pre", "wb-code");
  const seen = el("pre", "wb-code");
  sent.tabIndex = 0;
  seen.tabIndex = 0;
  sentCol.append(sentTitleNode, sent);
  seenCol.append(seenTitleNode, seen);
  pair.append(sentCol, seenCol);
  root.append(title, verdict, list, pair);

  const show = (spec: Spec | null, trace: Trace | null, note = ""): void => {
    if (!spec || !trace) {
      setText(title, "Pick a call");
      setText(verdict, note || "Nothing selected.");
      verdict.dataset.kind = "";
      stepNodes.forEach((s, i) => {
        s.li.dataset.state = "waiting";
        setText(s.note, "not reached");
        setText(s.li.children[1], STEP_TITLES[i]);
      });
      setText(sent, "");
      setText(seen, "");
      return;
    }
    setText(title, describe(spec));
    const said = trace.hookSays ? ` Hook said: ${trace.hookSays}` : "";
    setText(verdict, `${trace.label}${trace.reason ? `. ${trace.reason}` : ""}${said}`);
    verdict.dataset.kind = trace.kind;
    trace.steps.forEach((s, i) => {
      stepNodes[i].li.dataset.state = s.state;
      setText(stepNodes[i].note, s.note);
      stepNodes[i].note.title = s.note;
    });
    const has = trace.sent !== null && trace.sent !== undefined;
    setText(sent, has ? pretty(trace.sent) : "No result: the call stopped before the app answered.");
    setText(seen, has ? pretty(trace.seen) : "");
    setText(seenTitleNode, settings.redact ? seenTitle : `${seenTitle} (hiding is off)`);
  };
  return { root, show };
}

type Row = { spec: Spec; at: number; recorded: string; story: string };

function buildReplay(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  let rows: Row[] = RECORDING.map((r, i) => ({ spec: r.spec, at: Date.now() + r.at * MINUTE, recorded: recordedLabel[i] ?? "", story: r.story }));
  let source = "the demo session";
  let selected = 0;
  let traces: Trace[] = [];

  const head = el("div", "wb-head");
  const redactChip = chip("Hide secrets");
  const verifyChip = chip("File check");
  const hookPick = select([["", "no hook"], ...SCRIPTS.filter((s) => s.file !== "audit.py").map((s): [string, string] => [commandFor(s.file), s.file])], settings.hooks.before, "Before-call hook");
  const loadBtn = button("Open a log");
  const fileIn = el("input");
  fileIn.type = "file";
  fileIn.accept = ".jsonl,.json,.txt,text/plain,application/json";
  fileIn.hidden = true;
  const demoBtn = button("Demo session");
  const logBtn = button("Add replay to the Activity log");
  const exportBtn = button("Export log");
  head.append(redactChip, verifyChip, el("span", "wb-field", "Before hook"), hookPick, el("span", "wb-spacer"), loadBtn, demoBtn, exportBtn, fileIn);

  const cols = el("div", "wb-cols wb-cols-wide");
  const left = el("div", "wb-col");
  const ltitle = el("div", "wb-title");
  const list = el("ul", "wb-list");
  list.setAttribute("aria-label", "Recorded calls");
  left.append(ltitle, list);
  const detail = buildDetail();
  cols.append(left, detail.root);

  const foot = el("div", "wb-foot");
  const summary = el("span", "wb-note");
  foot.append(summary, logBtn);
  pane.append(head, cols, foot);

  let nodes: { btn: HTMLButtonElement; recorded: HTMLElement; now: HTMLElement }[] = [];

  function buildRows(): void {
    const header = el("li", "wb-row wb-row-head");
    for (const t of ["Time", "Agent", "Call", "Recorded", "Now"]) header.append(el("span", "", t));
    const items = rows.map((r, i) => {
      const li = el("li");
      const btn = el("button", "wb-row");
      btn.type = "button";
      btn.setAttribute("aria-pressed", String(i === selected));
      const recorded = el("span", "", r.recorded || "unknown");
      recorded.dataset.kind = r.recorded.startsWith("ok") ? "ok" : r.recorded.startsWith("refused") ? "refused" : "failed";
      const now = el("span");
      now.dataset.now = "";
      const call = el("span", "", [r.spec.name, r.spec.app, r.spec.tool].filter(Boolean).join(" · "));
      call.title = r.story;
      btn.append(el("span", "", clock(r.at)), el("span", "", CLIENT_NAMES[r.spec.client] ?? r.spec.client), call, recorded, now);
      btn.addEventListener("click", () => {
        selected = i;
        paint();
      });
      li.append(btn);
      return { li, btn, recorded, now };
    });
    nodes = items;
    list.replaceChildren(header, ...items.map((n) => n.li));
  }

  function paint(): void {
    redactChip.setAttribute("aria-pressed", String(settings.redact));
    verifyChip.setAttribute("aria-pressed", String(settings.verify));
    if (hookPick.value !== settings.hooks.before) {
      if (![...hookPick.options].some((o) => o.value === settings.hooks.before)) {
        const o = el("option", "", settings.hooks.before);
        o.value = settings.hooks.before;
        hookPick.append(o);
      }
      hookPick.value = settings.hooks.before;
    }
    const run: Ctx = { installs: ctx.installs, running: new Set() };
    traces = rows.map((r) => evaluate(r.spec, run));
    let differs = 0;
    nodes.forEach((n, i) => {
      const t = traces[i];
      setText(n.now, t.label);
      n.now.dataset.kind = t.kind;
      const d = t.label !== rows[i].recorded;
      if (d) differs += 1;
      n.btn.dataset.differs = String(d);
      n.btn.setAttribute("aria-pressed", String(i === selected));
    });
    setText(ltitle, `Replaying ${rows.length} calls from ${source} against the settings in the window`);
    setText(summary, differs ? `${differs} of ${rows.length} calls would go differently with today's settings.` : `Every call goes as recorded. Change a switch, the hook or a file and watch the Now column.`);
    const t = traces[selected];
    detail.show(rows[selected]?.spec ?? null, t ?? null);
  }

  redactChip.addEventListener("click", () => windowSwitch("redact-switch", !settings.redact));
  verifyChip.addEventListener("click", () => windowSwitch("verify-switch", !settings.verify));
  hookPick.addEventListener("change", () => windowHooks(hookPick.value, settings.hooks.after));
  logBtn.addEventListener("click", () => {
    const base = Date.now();
    traces.forEach((t, i) => commit(t, base + i * 5, `rep${base}${i}`));
    notify("activity-change");
    setText(summary, `Added ${traces.length} replayed calls to the Activity log with today's outcomes.`);
  });
  demoBtn.addEventListener("click", () => {
    rows = RECORDING.map((r, i) => ({ spec: r.spec, at: Date.now() + r.at * MINUTE, recorded: recordedLabel[i] ?? "", story: r.story }));
    source = "the demo session";
    selected = 0;
    buildRows();
    paint();
  });
  exportBtn.addEventListener("click", () => {
    const lines = events.map((e) => {
      const copy = structuredClone(e);
      redactValue(copy);
      return JSON.stringify(copy);
    });
    download("formmcp-activity.jsonl", `${lines.join("\n")}\n`, "application/x-ndjson");
  });
  loadBtn.addEventListener("click", () => fileIn.click());
  const open = async (file: File | undefined): Promise<void> => {
    if (!file) return;
    if (file.size > 512 * 1024) {
      setText(summary, `${file.name} is too large for the demo.`);
      return;
    }
    const parsed: LogEvent[] = [];
    for (const line of (await file.text()).split(/\r?\n/)) {
      if (!line.trim()) continue;
      try {
        const v = JSON.parse(line) as unknown;
        if (v && typeof v === "object" && typeof (v as LogEvent).event === "string") {
          redactValue(v);
          parsed.push(v as LogEvent);
        }
      } catch {
        continue;
      }
    }
    const calls = parsed.filter((e) => e.event === "call" && CALLS.includes(String(e.name)));
    if (!calls.length) {
      setText(summary, `${file.name} holds ${parsed.length} log lines but no tool calls to replay.`);
      return;
    }
    rows = calls.slice(0, 200).map((e) => {
      const who = AGENTS.includes(String(e.as)) ? String(e.as) : "other";
      const spec: Spec = { client: who, name: String(e.name), app: e.app ? String(e.app) : undefined, tool: e.tool ? String(e.tool) : undefined, args: {}, transport: e.transport === "http" ? "http" : "stdio" };
      const ok = e.ok !== false;
      return { spec, at: Number(e.t) || Date.now(), recorded: ok ? "ok" : "failed", story: `From ${file.name}` };
    });
    source = file.name;
    selected = 0;
    buildRows();
    paint();
  };
  fileIn.addEventListener("change", () => {
    const f = fileIn.files?.[0];
    fileIn.value = "";
    void open(f);
  });
  for (const t of [left, list]) {
    t.addEventListener("dragover", (e) => e.preventDefault());
    t.addEventListener("drop", (e) => {
      e.preventDefault();
      void open((e as DragEvent).dataTransfer?.files?.[0]);
    });
  }
  buildRows();
  return { pane, paint };
}

type Preset = { label: string; agent: string; name: string; app: string; tool: string; args: unknown; result?: unknown };

const PRESETS: Preset[] = [
  { label: "A fine call", agent: "claude", name: "call_app", app: "formvis", tool: "export_image", args: { format: "png" } },
  { label: "Result with a key", agent: "claude", name: "call_app", app: "formencoder", tool: "encode", args: { input: "talk.mov", preset: "web-1080p" }, result: { simulated: true, queued: 1, log: `uploader auth ${SAMPLE_TOKENS.ant}` } },
  { label: "Result with a header", agent: "cursor", name: "call_app", app: "formwav", tool: "formwav_apply_effect", args: { effect: "de-hum" }, result: { simulated: true, debug: `Authorization: ${SAMPLE_TOKENS.bearer}` } },
  { label: "A delete", agent: "codex", name: "call_app", app: "formmd", tool: "delete_page", args: { page: "Meeting notes" } },
  { label: "A key folder", agent: "codex", name: "call_app", app: "formpdf", tool: "open_pdf", args: { path: "/home/sam/.ssh/id_ed25519" } },
  { label: "A changed app", agent: "cursor", name: "call_app", app: "formvfx", tool: "get_status", args: {} },
  { label: "An unknown tool", agent: "claude", name: "call_app", app: "formcut", tool: "formcut_levitate", args: {} },
];

function buildTry(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  const cols = el("div", "wb-cols");
  const left = el("div", "wb-col");
  const presets = el("div", "wb-presets");
  const form = el("div", "wb-form");
  const agent = select(AGENTS.map((a): [string, string] => [a, CLIENT_NAMES[a]]), "claude", "Agent");
  const call = select(CALLS.map((c): [string, string] => [c, c]), "call_app", "Call");
  const mcpApps = family.apps.filter((a) => a.mcp);
  const app = select(mcpApps.map((a): [string, string] => [a.id, a.name]), "formvis", "App");
  const tool = select([], "", "Tool");
  const args = area('{ "format": "png" }', "Arguments");
  const result = area("", "What the app would send back, as JSON; empty for an example answer");
  result.placeholder = "What the app would send back, as JSON. Empty: an example answer.";
  form.append(row("Agent", agent), row("Call", call), row("App", app), row("Tool", tool), row("Arguments", args), row("App answers", result));
  const ask = el("div", "wb-ask");
  const send = button("Send the call", "btn btn-small btn-primary");
  const stop = button("Stop all apps");
  const running = el("span", "wb-note");
  ask.append(send, stop, running);
  left.append(presets, form, ask);
  const detail = buildDetail();
  cols.append(left, detail.root);
  pane.append(cols);

  const fillTools = (keep = ""): void => {
    const a = family.apps.find((x) => x.id === app.value);
    const names = a ? toolsOf(a).map((t) => t.name) : [];
    tool.replaceChildren(...names.map((n) => {
      const o = el("option", "", n);
      o.value = n;
      return o;
    }));
    if (keep && !names.includes(keep)) {
      const o = el("option", "", keep);
      o.value = keep;
      tool.append(o);
    }
    tool.value = keep || names[0] || "";
  };
  const gate = (): void => {
    const c = call.value;
    app.disabled = c === "family_apps" || c === "suggest_app";
    tool.disabled = c !== "call_app";
    result.disabled = c !== "call_app";
    if (c === "suggest_app" && !args.value.includes("task")) args.value = '{ "task": "make a 30 second trailer from these clips" }';
    if (c === "family_apps" && args.value.includes("task")) args.value = '{ "mcp_only": true }';
  };
  app.addEventListener("change", () => fillTools());
  call.addEventListener("change", gate);

  for (const p of PRESETS) {
    const c = chip(p.label);
    c.addEventListener("click", () => {
      agent.value = p.agent;
      call.value = p.name;
      app.value = p.app;
      fillTools(p.tool);
      args.value = JSON.stringify(p.args);
      result.value = p.result ? JSON.stringify(p.result) : "";
      gate();
      go();
    });
    presets.append(c);
  }

  const go = (): void => {
    let parsedArgs: Record<string, unknown> = {};
    let parsedResult: unknown;
    try {
      const v: unknown = args.value.trim() ? JSON.parse(args.value) : {};
      if (!v || typeof v !== "object" || Array.isArray(v)) throw new Error("not an object");
      parsedArgs = v as Record<string, unknown>;
    } catch (e) {
      detail.show(null, null, `The arguments are not a JSON object: ${String(e)}`);
      return;
    }
    if (result.value.trim() && !result.disabled) {
      try {
        parsedResult = JSON.parse(result.value);
      } catch (e) {
        detail.show(null, null, `The app's answer is not valid JSON: ${String(e)}`);
        return;
      }
    }
    const spec: Spec = { client: agent.value, name: call.value, app: call.value === "call_app" || call.value === "app_tools" ? app.value : undefined, tool: call.value === "call_app" ? tool.value : undefined, args: parsedArgs, result: parsedResult };
    const trace = evaluate(spec, ctx);
    commit(trace, Date.now(), `try${Date.now()}`);
    detail.show(spec, trace);
    notify("activity-change");
    paint();
  };
  send.addEventListener("click", go);
  stop.addEventListener("click", () => {
    ctx.running.clear();
    paint();
  });
  fillTools("export_image");
  gate();
  const paint = (): void => {
    setText(running, ctx.running.size ? `Running now: ${[...ctx.running].map(appName).join(", ")}. A started app is not checked again until it stops.` : "No app is running, so the next call starts one and checks its files.");
  };
  return { pane, paint };
}

function buildHooks(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  const cols = el("div", "wb-cols");
  const left = el("div", "wb-col");
  const right = el("div", "wb-col");
  const tabs = el("div", "wb-presets");
  const about = el("p", "wb-text");
  const source = el("pre", "wb-code");
  source.tabIndex = 0;
  const acts = el("div", "wb-ask");
  const useBefore = button("Use as the before hook");
  const useAfter = button("Use as the after hook");
  const none = button("No hooks");
  acts.append(useBefore, useAfter, none);
  left.append(tabs, about, source, acts);

  const form = el("div", "wb-form");
  const line = textInput(settings.hooks.before, "Hook command");
  const which = select([["before", "before the call"], ["after", "after the call"]], "before", "Hook kind");
  const app = textInput("formmd", "App");
  const tool = textInput("delete_page", "Tool");
  const args = area('{ "page": "Old drafts" }', "Arguments");
  args.style.height = "40px";
  form.append(row("Command", line), row("Runs", which), row("App", app), row("Tool", tool), row("Arguments", args));
  const ask = el("div", "wb-ask");
  const run = button("Run the hook", "btn btn-small btn-primary");
  ask.append(run, el("span", "wb-note", `Reads the event as JSON on stdin; a hook that does not finish in ${TIMEOUT_S} s refuses the call.`));
  const stdin = el("pre", "wb-code");
  const out = el("pre", "wb-code");
  stdin.tabIndex = 0;
  out.tabIndex = 0;
  const pair = el("div", "wb-pair");
  const a = el("div", "wb-col");
  const b = el("div", "wb-col");
  a.append(el("div", "wb-title", "What the hook is given on stdin"), stdin);
  b.append(el("div", "wb-title", "What happens"), out);
  pair.append(a, b);
  right.append(form, ask, pair);
  cols.append(left, right);
  pane.append(cols);

  let current = SCRIPTS[0];
  const buttons = SCRIPTS.map((s) => {
    const c = chip(s.file);
    c.title = s.title;
    c.addEventListener("click", () => {
      current = s;
      paintScript();
    });
    tabs.append(c);
    return c;
  });
  function paintScript(): void {
    buttons.forEach((c, i) => c.setAttribute("aria-pressed", String(SCRIPTS[i] === current)));
    setText(about, `${current.title}. ${current.about}`);
    setText(source, current.source);
    useBefore.disabled = current.file === "audit.py";
    useAfter.disabled = current.file !== "audit.py";
  }
  useBefore.addEventListener("click", () => {
    windowHooks(commandFor(current.file), settings.hooks.after);
    line.value = commandFor(current.file);
  });
  useAfter.addEventListener("click", () => windowHooks(settings.hooks.before, commandFor(current.file)));
  none.addEventListener("click", () => windowHooks("", ""));

  run.addEventListener("click", () => {
    let parsed: unknown = {};
    try {
      parsed = args.value.trim() ? JSON.parse(args.value) : {};
    } catch (e) {
      setText(out, `The arguments are not valid JSON: ${String(e)}`);
      out.className = "wb-code is-fail";
      return;
    }
    const base: HookEvent = { event: which.value, app: app.value.trim(), tool: tool.value.trim(), arguments: parsed, client: "claude", transport: "stdio", formmcp: "0.9.1" };
    const event: HookEvent = which.value === "after" ? { ...base, ok: true, ms: 41, result: { simulated: true } } : base;
    setText(stdin, pretty(event));
    const lines: string[] = [];
    try {
      lines.push(`command split into ${JSON.stringify(split(line.value))}`);
    } catch (e) {
      lines.push(String(e));
    }
    const ran = runLine(line.value, event);
    let tone = "is-ok";
    if (ran.error) {
      lines.push(`The hook could not run: ${ran.error}`, which.value === "before" ? "formMCP refuses the call." : "The result is still returned; the failure is noted in the log.");
      tone = "is-fail";
    } else if (ran.ran) {
      lines.push(`exit code ${ran.ran.code}  ·  ${ran.ran.ms} ms`, `stdout: ${ran.ran.stdout || "(empty)"}`, `stderr: ${ran.ran.stderr || "(empty)"}`);
      if (which.value === "before") {
        const v = judge(ran.ran);
        lines.push(v.ok ? "Verdict: allowed. The call goes on." : `Verdict: refused${v.reason ? `: ${v.reason}` : ""}. The agent is told why.`);
        if (!v.ok) tone = "is-fail";
      } else lines.push(ran.ran.code === 0 ? "Output is ignored." : `Non-zero exit (${ran.ran.code}) is noted in the log; the result is still returned.`);
      lines.push(`Exit code ${DENY_EXIT} also refuses; any other non-zero exit refuses too.`);
    }
    setText(out, lines.join("\n"));
    out.className = `wb-code ${tone}`;
  });
  paintScript();
  const paint = (): void => {
    if (document.activeElement !== line && which.value === "before" && line.value !== settings.hooks.before && !line.dataset.touched) line.value = settings.hooks.before;
  };
  line.addEventListener("input", () => {
    line.dataset.touched = "1";
  });
  return { pane, paint };
}

function buildTracing(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  const cols = el("div", "wb-cols");
  const left = el("div", "wb-col");
  const presets = el("div", "wb-presets");
  const state = el("p", "wb-text");
  const list = el("ul", "wb-list");
  list.setAttribute("aria-label", "Spans the collector received");
  left.append(presets, state, list);
  const right = el("div", "wb-col");
  const tTitle = el("div", "wb-title", "POST /v1/traces");
  const traces = el("pre", "wb-code");
  const mTitle = el("div", "wb-title", "POST /v1/metrics");
  const metrics = el("pre", "wb-code");
  traces.tabIndex = 0;
  metrics.tabIndex = 0;
  const acts = el("div", "wb-ask");
  const save = button("Save the OTLP bodies");
  const clear = button("Clear the collector");
  acts.append(save, clear);
  right.append(tTitle, traces, mTitle, metrics, acts);
  cols.append(left, right);
  pane.append(cols);

  const choices: [string, string][] = [
    ["Collector 127.0.0.1:4318", "http://127.0.0.1:4318"],
    ["Nothing listens on :9", "http://127.0.0.1:9"],
    ["Not an address", "collector"],
    ["Off", ""],
  ];
  const chipNodes = choices.map(([label, address]) => {
    const c = chip(label);
    c.addEventListener("click", () => windowTrace(address));
    presets.append(c);
    return c;
  });
  let picked = -1;
  let rendered = -1;

  function paint(): void {
    const address = settings.telemetry ?? "";
    chipNodes.forEach((c, i) => c.setAttribute("aria-pressed", String(choices[i][1] === address)));
    const problem = address ? endpointProblem(address) : null;
    if (!address) setText(state, "Tracing is off. Nothing leaves this computer, and nothing is recorded for a collector.");
    else if (problem) setText(state, `${problem}.`);
    else if (/:4318\/?$/.test(address)) setText(state, `Each call and app start is sent to ${address}/v1/traces and /v1/metrics as OTLP JSON. The demo's collector accepts it and keeps ${collector.received.length} spans below. Names and outcomes only: no arguments, results or secrets.`);
    else setText(state, `${address} refuses the connection. formMCP warns once per run in the Activity log and goes on; calls are never slowed or refused by tracing.`);
    if (rendered !== collector.received.length) {
      rendered = collector.received.length;
      const rows = collector.received.map((r, i) => {
        const li = el("li");
        const btn = el("button", "wb-row");
        btn.type = "button";
        btn.style.gridTemplateColumns = "5.4em minmax(0, 1.6fr) 3.6em 4.4em";
        btn.setAttribute("aria-pressed", String(i === picked));
        const outcome = el("span", "", r.span.ok ? "ok" : "error");
        outcome.dataset.kind = r.span.ok ? "ok" : "failed";
        btn.append(el("span", "", clock(r.span.at)), el("span", "", r.span.name), outcome, el("span", "", `${r.span.ms} ms`));
        btn.addEventListener("click", () => {
          picked = i;
          rendered = -1;
          paint();
        });
        li.append(btn);
        return li;
      });
      list.replaceChildren(...rows);
      if (picked < 0 || picked >= collector.received.length) picked = collector.received.length - 1;
      list.scrollTop = list.scrollHeight;
    }
    const r = collector.received[picked];
    setText(traces, r ? pretty(r.traces) : "No span yet. Send a call from Try a call or add the replay to the log.");
    setText(metrics, r ? pretty(r.metrics) : "");
  }
  save.addEventListener("click", () => {
    const r = collector.received[picked];
    if (r) download("formmcp-otlp.json", pretty({ traces: r.traces, metrics: r.metrics }), "application/json");
  });
  clear.addEventListener("click", () => {
    collector.received.length = 0;
    picked = -1;
    rendered = -2;
    paint();
  });
  return { pane, paint };
}

function buildFiles(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  const cols = el("div", "wb-cols");
  const left = el("div", "wb-col");
  const list = el("div", "wb-list");
  left.append(el("div", "wb-title", "Installed apps and what formMCP compared when it last started each"), list);
  const right = el("div", "wb-col");
  const title = el("div", "wb-title");
  const text = el("pre", "wb-code");
  text.tabIndex = 0;
  right.append(title, text);
  cols.append(left, right);
  pane.append(cols);
  let picked = "formvfx";

  const lines = ctx.installs.map((i) => {
    const r = el("div", "wb-fileline");
    const who = el("span", "wb-who");
    who.append(iconNode(i.id), el("span", "", appName(i.id)));
    const version = el("span", "", i.version);
    const state = el("span", "wb-fstate");
    const acts = el("span", "wb-actions");
    const change = button("Change a file");
    const restore = button("Restore");
    acts.append(change, restore);
    r.append(who, version, state, acts);
    r.addEventListener("click", () => {
      picked = i.id;
      paint();
    });
    change.addEventListener("click", () => {
      const next = i.candidates.find((c) => !i.changed.includes(c));
      if (next) i.changed.push(next);
      notify("safety-change");
    });
    restore.addEventListener("click", () => {
      i.changed = [];
      notify("safety-change");
    });
    list.append(r);
    return { i, r, state, change, restore };
  });

  function paint(): void {
    for (const l of lines) {
      const bad = l.i.changed.length > 0;
      l.r.dataset.bad = String(bad);
      l.r.setAttribute("aria-current", String(l.i.id === picked));
      setText(l.state, !settings.verify ? `${l.i.files} files, check is off` : bad ? `${l.i.changed.length} of ${l.i.files} files differ` : `${l.i.files} files match`);
      l.change.disabled = l.i.changed.length >= l.i.candidates.length;
      l.restore.disabled = !bad;
    }
    const inst = installOf(ctx.installs, picked);
    if (!inst) return;
    setText(title, `${appName(inst.id)} ${inst.version}`);
    const problem = fileProblem(inst);
    const how = inst.via === "formportal" ? "FormPortal's files record (0.46.0 and later) lists every file with its digest; formMCP compares what is on disk with it." : "This install has no FormPortal record, so formMCP compares with the file list it recorded itself the first time it started this version.";
    const body = [how, "", settings.verify ? (problem ? `A call to ${appName(inst.id)} gets:\n${appName(inst.id)} was not started: ${problem}` : `Nothing differs: ${appName(inst.id)} starts.`) : "The file check is off in the Safety panel: this app starts as it is.", "", inst.changed.length ? `Files that differ:\n${[...inst.changed].sort().map((c) => `  ${c}`).join("\n")}` : "No file differs.", "", "In the window's Safety panel, Record again accepts the files as they are now (two clicks); only do that for a change you made yourself. FormPortal's Reinstall or Repair restores them."];
    setText(text, body.join("\n"));
    text.className = `wb-code ${settings.verify && problem ? "is-fail" : ""}`;
  }
  return { pane, paint };
}

function buildAsking(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  const scroll = el("div", "wb-scroll");
  const intro = el("p", "wb-text", "formMCP has no consent dialog of its own: it does not interrupt an agent with a prompt. A person stays in charge through these real controls, and an agent is expected to ask before changing files.");
  const list = el("ul", "wb-consent");
  const items: [string, string][] = [
    ["Two-click confirms", "Remove a secret, Record again and New token each need a second click within 4 s, and say what the click will do. Try one in the window's Safety panel or HTTP bar."],
    ["Before-call hook", "A command of the person's choosing sees each call as JSON and allows or refuses it, with a reason the agent can pass on. It is the place for \"ask me first\" rules; see the Hooks tab."],
    ["Switches", "An app turned off for one agent, a profile that hides apps or tools, or a single tool switch refuses at once, and the refusal names who can change it."],
    ["File check", "An installed app whose files differ from FormPortal's record is never started, however the agent asks. Only the person can accept the change."],
    ["Secrets", "Kept in the keychain and given to an app only when formMCP starts it. Results are cleaned before the agent reads them; the Activity log is always cleaned."],
    ["The agent's rule", "The family's harness skill tells an agent to name what a tool will change and to wait for a yes before a tool that edits or deletes. That is an agent's habit, not something formMCP enforces, so the demo only quotes it."],
  ];
  for (const [name, what] of items) {
    const li = el("li");
    li.append(el("strong", "", name), el("span", "", what));
    list.append(li);
  }
  scroll.append(intro, list);
  pane.append(scroll);
  return { pane, paint: () => undefined };
}

function build(): void {
  const wb = document.getElementById("wb") as HTMLElement;
  const tabs = el("div", "chips wb-tabs");
  tabs.setAttribute("role", "group");
  const names = ["Replay", "Try a call", "Hooks", "Tracing", "Files", "Asking the person"];
  const parts = [buildReplay(), buildTry(), buildHooks(), buildTracing(), buildFiles(), buildAsking()];
  let current = 0;
  const buttons = names.map((n, i) => {
    const b = el("button", "chip", n);
    b.type = "button";
    b.setAttribute("aria-pressed", String(i === 0));
    b.addEventListener("click", () => {
      current = i;
      buttons.forEach((x, j) => x.setAttribute("aria-pressed", String(i === j)));
      parts.forEach((p, j) => (p.pane.hidden = i !== j));
      parts[i].paint();
    });
    tabs.append(b);
    return b;
  });
  const body = el("div", "wb-body");
  parts.forEach((p, i) => {
    p.pane.hidden = i !== 0;
    body.append(p.pane);
  });
  wb.append(tabs, body);
  const all = (): void => {
    for (const p of parts) p.paint();
  };
  document.addEventListener("safety-change", all);
  document.addEventListener("activity-change", () => parts[current].paint());
  all();
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

async function open(): Promise<void> {
  const toggle = document.getElementById("safety-toggle") as HTMLButtonElement;
  const before = document.getElementById("hook-before") as HTMLInputElement;
  for (let i = 0; i < 40 && toggle.disabled; i += 1) await wait(25);
  if (toggle.getAttribute("aria-expanded") !== "true") toggle.click();
  for (let i = 0; i < 40 && !before.value; i += 1) await wait(25);
}

build();
const root = document.documentElement;
root.classList.add("booting");
const failsafe = wait(1500);
void Promise.race([Promise.all([open(), ...pending]), failsafe]).then(() => root.classList.remove("booting"));
