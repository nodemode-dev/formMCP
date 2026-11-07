import { appOf, isOff, record, redactValue, sampleTools, settings, suggest, toolHidden } from "./gateway";
import { iconNode, shotUrl } from "./media";

type Kind = "suggest_app" | "app_tools" | "call_app";
type Step = { kind: Kind; app?: string; tool?: string; args: Record<string, unknown>; result?: unknown; shot: string; why: string };
type Flow = { title: string; task: string; steps: Step[] };

const AGENTS: [string, string][] = [["claude", "Claude Code"], ["codex", "Codex"], ["cursor", "Cursor"], ["other", "Other agents"]];

function bearer(): string {
  return ["Author", "ization: Bea", "rer ", "demoDemoDemo1234567890abcdef"].join("");
}

const FLOWS: Flow[] = [
  {
    title: "Make a 30 s trailer from these clips",
    task: "cut a 30 second trailer from these video clips",
    steps: [
      { kind: "suggest_app", args: { task: "cut a 30 second trailer from these video clips", limit: 3 }, shot: "mcp-agents", why: "formMCP ranks the family for the task. It suggests; it never installs." },
      { kind: "app_tools", app: "formcut", args: { app: "formcut" }, shot: "formcut", why: "Only now is FormCut's own server started and its tools listed." },
      { kind: "call_app", app: "formcut", tool: "formcut_import_media", args: { paths: ["clips/harbour.mp4", "clips/market.mp4", "clips/ridge.mp4"] }, result: { imported: 3, bin: "Trailer", note: "simulated" }, shot: "formcut", why: "The clips land in the project's media bin." },
      { kind: "call_app", app: "formcut", tool: "formcut_find_scenes", args: { clip: "harbour.mp4" }, result: { scenes: [{ in: 0, out: 4.2 }, { in: 4.2, out: 9.8 }, { in: 9.8, out: 14.1 }], note: "simulated" }, shot: "formcut", why: "Scene cuts give the agent real in and out points." },
      { kind: "call_app", app: "formcut", tool: "formcut_render", args: { preset: "web 1080p", name: "trailer" }, result: { job: "r-0001", state: "queued", note: "simulated" }, shot: "formcut", why: "The timeline renders to a file." },
      { kind: "call_app", app: "formencoder", tool: "encode", args: { path: "trailer.mp4", preset: "social square" }, result: { output: "trailer-square.mp4", webhook_reply: bearer(), note: "simulated" }, shot: "formencoder", why: "FormEncoder makes the social version. A token in the reply is hidden before the agent sees it." },
    ],
  },
  {
    title: "Clean up and master a podcast episode",
    task: "clean up podcast audio and master it",
    steps: [
      { kind: "suggest_app", args: { task: "clean up podcast audio and master it", limit: 3 }, shot: "mcp-agents", why: "The task words match audio apps first." },
      { kind: "app_tools", app: "formwav", args: { app: "formwav" }, shot: "formwav", why: "FormWav's tool list, loaded on demand." },
      { kind: "call_app", app: "formwav", tool: "formwav_analyze_media", args: { path: "episode-12.wav" }, result: { loudness_lufs: -21.4, noise_floor_db: -48, note: "simulated" }, shot: "formwav", why: "Measure before touching anything." },
      { kind: "call_app", app: "formwav", tool: "formwav_apply_effect", args: { effect: "denoise", amount: 0.6 }, result: { applied: "denoise", noise_floor_db: -62, note: "simulated" }, shot: "formwav", why: "Repair the recording." },
      { kind: "call_app", app: "formwav", tool: "formwav_export_media", args: { path: "episode-12-clean.wav" }, result: { written: "episode-12-clean.wav", note: "simulated" }, shot: "formwav", why: "Hand the cleaned file on." },
      { kind: "call_app", app: "formencoder", tool: "encode", args: { path: "episode-12-clean.wav", preset: "podcast mp3", loudness: "-16 LUFS" }, result: { output: "episode-12.mp3", loudness_lufs: -16.0, note: "simulated" }, shot: "formencoder", why: "Loudness-normalised delivery file." },
    ],
  },
  {
    title: "Retouch a photo and build a poster",
    task: "retouch a photo and remove the background",
    steps: [
      { kind: "suggest_app", args: { task: "retouch a photo and remove the background", limit: 3 }, shot: "mcp-agents", why: "Image apps rank first." },
      { kind: "app_tools", app: "formvis", args: { app: "formvis" }, shot: "formvis", why: "FormVis starts and lists its tools." },
      { kind: "call_app", app: "formvis", tool: "add_layer", args: { name: "Sky" }, result: { id: "L4", name: "Sky", note: "simulated" }, shot: "formvis", why: "A new layer for the replacement sky." },
      { kind: "call_app", app: "formvis", tool: "apply_filter", args: { kind: "gaussianBlur", radius: 6 }, result: { filter: "gaussianBlur", layer: "L4", note: "simulated" }, shot: "formvis", why: "Soften the plate." },
      { kind: "call_app", app: "formvis", tool: "export_image", args: { path: "poster.png" }, result: { written: "poster.png", note: "simulated" }, shot: "formvis", why: "Save the finished poster." },
    ],
  },
];

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls = "", text = ""): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text) node.textContent = text;
  return node;
}

function setText(node: Element, text: string): void {
  if (node.textContent !== text) node.textContent = text;
}

const wait = (ms: number): Promise<void> => new Promise((r) => window.setTimeout(r, ms));

export function buildFlows(): HTMLElement {
  const pane = el("section", "lab-pane");
  const form = el("form", "lab-form");
  const pick = el("select");
  pick.setAttribute("aria-label", "Task flow");
  FLOWS.forEach((f, i) => {
    const o = el("option", "", f.title);
    o.value = String(i);
    pick.append(o);
  });
  const agent = el("select");
  agent.setAttribute("aria-label", "Agent");
  for (const [v, n] of AGENTS) {
    const o = el("option", "", n);
    o.value = v;
    agent.append(o);
  }
  const go = el("button", "btn btn-small", "Run flow");
  go.type = "submit";
  form.append(pick, agent, go);

  const note = el("p", "profile-note", "");
  const list = el("ol", "flow-steps");
  const slots = Array.from({ length: 6 }, () => {
    const li = el("li", "flow-step");
    li.tabIndex = 0;
    li.dataset.state = "waiting";
    const icon = el("span", "flow-icon");
    const body = el("span", "flow-body");
    const title = el("strong", "flow-title");
    title.translate = false;
    const args = el("span", "flow-args");
    body.append(title, args);
    const state = el("span", "flow-state", "waiting");
    li.append(icon, body, state);
    list.append(li);
    return { li, icon, title, args, state, appId: "" };
  });

  const frame = el("div", "flow-shot");
  const shot = el("img");
  shot.width = 960;
  shot.height = 600;
  shot.alt = "";
  frame.append(shot);
  const why = el("p", "profile-note flow-why", "");
  const out = el("pre", "lab-out flow-out");
  out.tabIndex = 0;
  pane.append(form, note, list, frame, why, out);

  const details: string[] = [];
  let running = false;
  let shown = "";

  function flow(): Flow {
    return FLOWS[Number(pick.value)];
  }

  async function swap(id: string): Promise<void> {
    const url = shotUrl(id);
    if (!url || url === shown) return;
    const next = new Image();
    next.src = url;
    try {
      await next.decode();
    } catch {
      return;
    }
    shown = url;
    shot.src = url;
  }

  function select(i: number): void {
    slots.forEach((s, n) => s.li.setAttribute("aria-current", String(n === i)));
    const step = flow().steps[i];
    if (!step) return;
    setText(why, step.why);
    setText(out, details[i] ?? JSON.stringify(request(step), null, 2));
    void swap(step.shot);
  }

  function request(step: Step): unknown {
    return { jsonrpc: "2.0", method: "tools/call", params: { name: step.kind, arguments: step.kind === "call_app" ? { app: step.app, tool: step.tool, arguments: step.args } : step.args } };
  }

  function reset(): void {
    const f = flow();
    details.length = 0;
    slots.forEach((s, i) => {
      const step = f.steps[i];
      s.li.hidden = !step;
      s.li.dataset.state = "waiting";
      setText(s.state, "waiting");
      if (!step) return;
      const id = step.app ?? "formmcp";
      if (s.appId !== id || !s.icon.firstChild) {
        s.icon.replaceChildren(iconNode(id, 22));
        s.appId = id;
      }
      setText(s.title, step.kind === "call_app" ? `call_app · ${step.app} · ${step.tool}` : step.app ? `app_tools · ${step.app}` : "suggest_app");
      setText(s.args, JSON.stringify(step.args));
    });
    setText(note, `Task given to the agent: "${f.task}". Run the flow to watch formMCP serve each call; select a step to read its request and result.`);
    select(0);
  }

  function execute(step: Step, who: string): { ok: boolean; result: unknown; hidden: number } {
    if (step.kind === "suggest_app") {
      const picks = suggest(who, String(step.args.task), 3);
      return { ok: picks.length > 0, result: { task: step.args.task, apps: picks.map((p) => ({ id: p.id, name: p.name, kind: p.kind, score: p.score, mcp: p.mcp })), note: "Best first. formMCP never installs: FormPortal installs and updates the Form apps." }, hidden: 0 };
    }
    const app = appOf(step.app ?? "");
    if (!app) return { ok: false, result: { isError: true, error: `there is no Form app ${JSON.stringify(step.app)}` }, hidden: 0 };
    if (isOff(who, app.id)) return { ok: false, result: { isError: true, error: `${app.name} is turned off for this agent` }, hidden: 0 };
    const tools = sampleTools(app);
    if (step.kind === "app_tools") {
      return { ok: true, result: { app: app.id, name: app.name, simulated: true, tools: tools.filter((t) => !toolHidden(who, app.id, t.name)).map((t) => ({ name: t.name, description: t.description, read_only: t.read_only })) }, hidden: 0 };
    }
    if (toolHidden(who, app.id, step.tool ?? "")) return { ok: false, result: { isError: true, error: `${step.tool} is turned off for ${app.name}` }, hidden: 0 };
    const value: Record<string, unknown> = { content: [{ type: "text", text: JSON.stringify(step.result) }], isError: false };
    const hidden = settings.redact ? redactValue(value) : 0;
    return { ok: true, result: value, hidden };
  }

  async function run(): Promise<void> {
    if (running) return;
    running = true;
    go.disabled = true;
    pick.disabled = true;
    agent.disabled = true;
    reset();
    const f = flow();
    for (let i = 0; i < f.steps.length; i += 1) {
      const step = f.steps[i];
      slots[i].li.dataset.state = "running";
      setText(slots[i].state, "running");
      select(i);
      await wait(650);
      const started = performance.now();
      const r = execute(step, agent.value);
      record("call", { name: step.kind, app: step.app ?? null, tool: step.tool ?? null, ok: r.ok, ms: Math.round(performance.now() - started), client: agent.value });
      details[i] = JSON.stringify({ request: request(step), response: r.result }, null, 2);
      slots[i].li.dataset.state = r.ok ? "done" : "failed";
      setText(slots[i].state, r.ok ? (r.hidden ? `done · ${r.hidden} hidden` : "done") : "refused");
      select(i);
      if (!r.ok) {
        setText(note, "The flow stopped: formMCP refused the call. Switch the app on again in the window, or choose another agent, and run it again.");
        break;
      }
      if (i === f.steps.length - 1) setText(note, `Done: ${f.steps.length} calls served, each one recorded in the Activity log. Every app result here is simulated; no Form app runs in a browser.`);
    }
    running = false;
    go.disabled = false;
    pick.disabled = false;
    agent.disabled = false;
  }

  slots.forEach((s, i) => {
    s.li.addEventListener("click", () => select(i));
    s.li.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        select(i);
      }
    });
  });
  form.addEventListener("submit", (e) => {
    e.preventDefault();
    void run();
  });
  pick.addEventListener("change", () => {
    if (!running) reset();
  });
  reset();
  return pane;
}
