import { buildFlows } from "./flows";
import { iconNode } from "./media";
import { MARK, PROFILE_FORMAT, appOf, familyApps, family, isOff, live, loaded, record, redact, redactValue, sampleTools, settings, suggest, toolHidden } from "./gateway";

const AGENTS: [string, string][] = [["claude", "Claude Code"], ["codex", "Codex"], ["cursor", "Cursor"], ["other", "Other agents"]];
const TASKS = ["cut the silences out of this interview video", "retouch a photo and remove the background", "sign and annotate a PDF", "clean up podcast audio and master it", "lay out a magazine page for print", "convert a folder of videos to smaller files", "take notes in markdown with a database"];

const TOOL_LIST: [string, string, string][] = [
  ["family_apps", "The Form family", "Every Form app: id, name, kind, one line on what it is for, whether it has an MCP server, its release state, whether it is installed and its reach. No app's tools are listed here."],
  ["suggest_app", "Which Form app fits a task", "Given a task in the person's words, returns up to five apps that fit, best first, with the next step. It never installs anything."],
  ["app_tools", "One app's tools", "Starts one app's own MCP server and returns its tools with their input schemas."],
  ["call_app", "Run one of an app's tools", "Runs a tool from app_tools in that app and returns the app's own result unchanged."],
  ["app_resources", "One app's resources", "Lists the resources one app offers, and its resource templates."],
  ["read_app_resource", "Read one app resource", "Reads one resource by the uri app_resources gave."],
  ["app_prompts", "One app's prompts", "Lists the prompt templates one app offers, with their arguments."],
  ["get_app_prompt", "Fill one app prompt", "Fills one prompt from app_prompts with its arguments."],
];

const SAMPLE_ARGS: Record<string, string> = {
  family_apps: '{ "mcp_only": true }',
  suggest_app: '{ "task": "cut the silences out of this interview video", "limit": 3 }',
  app_tools: '{ "app": "formcut" }',
  call_app: '{ "app": "formcut", "tool": "formcut_find_scenes", "arguments": { "clip": "harbour.mp4" } }',
  app_resources: '{ "app": "formmd" }',
  read_app_resource: '{ "app": "formmd", "uri": "formmd://notes" }',
  app_prompts: '{ "app": "formwav" }',
  get_app_prompt: '{ "app": "formwav", "name": "clean_voice", "arguments": {} }',
};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls = "", text = ""): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text) node.textContent = text;
  return node;
}

function setText(node: Element, text: string): void {
  if (node.textContent !== text) node.textContent = text;
}

function agentSelect(label: string): { wrap: HTMLLabelElement; select: HTMLSelectElement } {
  const wrap = el("label", "agent-pick");
  wrap.append(el("span", "", label));
  const select = el("select");
  for (const [value, name] of AGENTS) {
    const o = el("option", "", name);
    o.value = value;
    select.append(o);
  }
  wrap.append(select);
  return { wrap, select };
}

function tabs(names: string[]): { bar: HTMLElement; buttons: HTMLButtonElement[] } {
  const bar = el("div", "chips lab-tabs");
  bar.setAttribute("role", "group");
  const buttons = names.map((n) => {
    const b = el("button", "chip", n);
    b.type = "button";
    b.setAttribute("aria-pressed", "false");
    bar.append(b);
    return b;
  });
  return { bar, buttons };
}

function sample(): string {
  const bits = {
    fmd: ["fmd", "_", "demo0123456789ABCDEFabcd"].join(""),
    ant: ["sk", "-ant-", "api03-DEMODEMODEMODEMO12345"].join(""),
    gh: ["gh", "p_", "0123456789abcdefghijklmnopqrstuvwxyz"].join(""),
    aws: ["AK", "IA", "0123456789ABCDEF"].join(""),
    bearer: ["Author", "ization: Bea", "rer ", "demoDemoDemo1234567890abcdef"].join(""),
    key: ["-----BEGIN ", "PRIVATE KEY-----\nDEMODEMODEMODEMO\n-----END ", "PRIVATE KEY-----"].join(""),
  };
  return `result from an app tool\nform.md token: ${bits.fmd}\napi key line: ${bits.ant}\nrepo access ${bits.gh}\naws id ${bits.aws}\n${bits.bearer}\n${bits.key}\nnothing secret on this line\n`;
}

function build(): void {
  const strip = document.getElementById("demo-strip") as HTMLElement;
  const open = strip.querySelector<HTMLButtonElement>("[data-lab-open]");
  const lab = el("aside", "lab");
  lab.id = "lab";
  lab.hidden = true;
  lab.setAttribute("aria-label", "Lab");

  const head = el("div", "lab-head");
  head.append(el("span", "lab-title", "Lab"));
  const close = el("button", "btn btn-small", "Close");
  close.type = "button";
  head.append(close);

  const { bar, buttons } = tabs(["Flows", "Suggest", "Console", "Redact"]);
  const flowsPane = buildFlows();

  const suggestPane = el("section", "lab-pane");
  const sForm = el("form", "lab-form");
  const sTask = el("input");
  sTask.type = "text";
  sTask.placeholder = "What the person wants done";
  sTask.setAttribute("aria-label", "Task");
  sTask.value = TASKS[0];
  const sAgent = agentSelect("Agent");
  const sGo = el("button", "btn btn-small", "Suggest");
  sGo.type = "submit";
  sForm.append(sTask, sAgent.wrap, sGo);
  const sChips = el("div", "lab-samples");
  for (const t of TASKS) {
    const c = el("button", "chip", t);
    c.type = "button";
    c.addEventListener("click", () => {
      sTask.value = t;
      run();
    });
    sChips.append(c);
  }
  const sNote = el("p", "profile-note", "suggest_app scores the task against every app's name, kind and summary, the same way formMCP's engine does, and skips apps switched off for the agent. Switch an app off in the window and ask again.");
  const sList = el("ol", "lab-results");
  const slots = Array.from({ length: 5 }, () => {
    const li = el("li", "lab-result");
    li.hidden = true;
    const top = el("div", "lab-result-top");
    const name = el("strong");
    name.translate = false;
    const meta = el("span", "app-kind");
    const who = el("span", "lab-who");
    const icon = el("span", "lab-icon");
    who.append(icon, name);
    top.append(who, meta);
    const sum = el("p", "app-summary");
    const next = el("p", "lab-next");
    li.append(top, sum, next);
    sList.append(li);
    return { li, name, icon, meta, sum, next, shown: "" };
  });
  suggestPane.append(sForm, sChips, sNote, sList);

  function run(): void {
    const task = sTask.value.trim();
    if (!task) {
      setText(sNote, "Type a task first.");
      return;
    }
    const picks = suggest(sAgent.select.value, task, 5);
    record("call", { name: "suggest_app", ok: true, ms: 0, client: sAgent.select.value });
    slots.forEach((s, i) => {
      const p = picks[i];
      s.li.hidden = !p;
      if (!p) return;
      setText(s.name, p.name);
      if (s.shown !== p.id) {
        s.icon.replaceChildren(iconNode(p.id, 22));
        s.shown = p.id;
      }
      setText(s.meta, `${p.kind} · score ${p.score} · ${p.mcp ? "MCP server" : "no MCP server"} · ${p.state}`);
      setText(s.sum, p.summary);
      setText(s.next, p.next);
    });
    setText(sNote, picks.length ? `${picks.length} app${picks.length === 1 ? "" : "s"} for "${task}", best first.` : "No app matched the words of the task; call family_apps and choose by kind and summary.");
  }
  sForm.addEventListener("submit", (e) => {
    e.preventDefault();
    run();
  });

  const consolePane = el("section", "lab-pane");
  const cForm = el("form", "lab-form");
  const cTool = el("select");
  cTool.setAttribute("aria-label", "MCP tool");
  const lists = el("option", "", "tools/list");
  lists.value = "tools/list";
  cTool.append(lists);
  for (const [name] of TOOL_LIST) {
    const o = el("option", "", name);
    o.value = name;
    cTool.append(o);
  }
  cTool.value = "family_apps";
  const cAgent = agentSelect("Agent");
  const cGo = el("button", "btn btn-small", "Run");
  cGo.type = "submit";
  cForm.append(cTool, cAgent.wrap, cGo);
  const cArgs = el("textarea", "lab-args");
  cArgs.spellcheck = false;
  cArgs.setAttribute("aria-label", "Arguments as JSON");
  cArgs.value = SAMPLE_ARGS.family_apps;
  const cSim = el("label", "lab-check");
  const cBox = el("input");
  cBox.type = "checkbox";
  cBox.checked = live.answerWithSamples;
  cSim.append(cBox, el("span", "", "Answer app calls with simulated samples (no Form app runs in a browser)"));
  const cOut = el("pre", "lab-out");
  cOut.tabIndex = 0;
  consolePane.append(cForm, cArgs, cSim, cOut);

  cTool.addEventListener("change", () => {
    cArgs.value = SAMPLE_ARGS[cTool.value] ?? "{}";
  });
  cBox.addEventListener("change", () => {
    live.answerWithSamples = cBox.checked;
  });

  function callTool(name: string, agent: string, args: Record<string, unknown>): unknown {
    const id = String(args.app ?? "");
    if (name === "family_apps") return familyApps(agent, args.mcp_only === true);
    if (name === "suggest_app") {
      const task = String(args.task ?? "").trim();
      if (!task) throw "task is required: what the person wants done, in their words";
      const picks = suggest(agent, task, Number(args.limit ?? 3));
      return { task, apps: picks, note: picks.length ? "Best first. formMCP never installs: FormPortal installs and updates the Form apps." : "No app matched the words of the task; call family_apps and choose by kind and summary." };
    }
    const app = appOf(id);
    if (!app) throw `there is no Form app ${JSON.stringify(id)}; ids come from family_apps`;
    if (isOff(agent, app.id)) throw `${app.name} is turned off for this agent`;
    if (!app.mcp) throw `${app.name} has no MCP server`;
    if (!live.answerWithSamples) throw `${app.name} is not installed here, and formMCP cannot start its server in a browser`;
    if (name === "app_tools") {
      loaded.add(app.id);
      return { app: app.id, name: app.name, simulated: true, tools: sampleTools(app).filter((t) => !toolHidden(agent, app.id, t.name)).map((t) => ({ ...t, inputSchema: { type: "object", properties: {} } })) };
    }
    if (name === "call_app") {
      const tool = String(args.tool ?? "");
      if (!sampleTools(app).some((t) => t.name === tool)) throw `${app.name} has no tool ${JSON.stringify(tool)}; names come from app_tools`;
      if (toolHidden(agent, app.id, tool)) throw `${tool} is turned off for ${app.name}`;
      return { content: [{ type: "text", text: JSON.stringify({ simulated: true, app: app.id, tool, echoed_arguments: args.arguments ?? {} }) }], isError: false };
    }
    if (name === "app_resources") return { app: app.id, count: 0, resources: [], templates: [], note: `${app.name} offers no resources` };
    if (name === "app_prompts") return { app: app.id, count: 0, prompts: [], note: `${app.name} offers no prompts` };
    throw `${app.name} offers no resource or prompt of that name`;
  }

  cForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const name = cTool.value;
    const started = performance.now();
    let result: unknown;
    let ok = true;
    try {
      if (name === "tools/list") {
        result = { tools: TOOL_LIST.map(([n, title, description]) => ({ name: n, title, description })) };
      } else {
        const parsed = cArgs.value.trim() ? JSON.parse(cArgs.value) : {};
        const value = callTool(name, cAgent.select.value, parsed);
        result = { content: [{ type: "text", text: "(structuredContent below)" }], structuredContent: value, isError: false };
        if (settings.redact) {
          const hidden = redactValue(result);
          if (hidden) (result as Record<string, unknown>)._hidden_by_formmcp = hidden;
        }
      }
    } catch (err) {
      ok = false;
      result = { content: [{ type: "text", text: String(err) }], isError: true };
    }
    record("call", { name, ok, ms: Math.round(performance.now() - started), client: cAgent.select.value });
    setText(cOut, JSON.stringify(result, null, 2));
  });

  const redactPane = el("section", "lab-pane");
  const rIn = el("textarea", "lab-args");
  rIn.spellcheck = false;
  rIn.placeholder = "Paste text an app might send back, or drop a text file anywhere on this panel";
  rIn.setAttribute("aria-label", "Text to scan");
  const rRow = el("div", "lab-form");
  const rLoad = el("button", "btn btn-small", "Load sample");
  rLoad.type = "button";
  const rPick = el("button", "btn btn-small", "Open a file");
  rPick.type = "button";
  const rFile = el("input");
  rFile.type = "file";
  rFile.hidden = true;
  const rGo = el("button", "btn btn-small", "Scan");
  rGo.type = "button";
  const rCount = el("span", "count", "");
  rRow.append(rLoad, rPick, rGo, rCount, rFile);
  const rOut = el("pre", "lab-out");
  rOut.tabIndex = 0;
  const rNote = el("p", "profile-note", `Known secret shapes and every value kept in the Safety panel are replaced with ${MARK}. Turn hiding off in Safety to see results unchanged.`);
  redactPane.append(rRow, rIn, rOut, rNote);

  function scan(): void {
    const r = redact(rIn.value);
    setText(rOut, settings.redact ? r.text : rIn.value);
    setText(rCount, settings.redact ? `${r.found} hidden` : `${r.found} found, hiding is off`);
  }
  rLoad.addEventListener("click", () => {
    rIn.value = sample();
    scan();
  });
  rGo.addEventListener("click", scan);
  rIn.addEventListener("input", scan);
  rPick.addEventListener("click", () => rFile.click());
  rFile.addEventListener("change", () => {
    const f = rFile.files?.[0];
    rFile.value = "";
    if (f) void take(f);
  });

  const panes = [flowsPane, suggestPane, consolePane, redactPane];
  const show = (i: number): void => {
    panes.forEach((p, n) => {
      p.hidden = n !== i;
      buttons[n].setAttribute("aria-pressed", String(n === i));
    });
  };
  buttons.forEach((b, i) => b.addEventListener("click", () => show(i)));

  const drop = el("div", "lab-drop", "Drop a file here. A formMCP profile (JSON) is imported into the window; any other text file is scanned for secrets.");
  const dropNote = el("p", "profile-note", "");

  async function take(file: File): Promise<void> {
    if (file.size > 2 * 1024 * 1024) {
      setText(dropNote, `${file.name} is larger than 2 MB.`);
      return;
    }
    const text = await file.text();
    let isProfile = false;
    try {
      isProfile = (JSON.parse(text) as { format?: string }).format === PROFILE_FORMAT;
    } catch {
      isProfile = false;
    }
    if (isProfile) {
      const profilesToggle = document.getElementById("profiles-toggle") as HTMLButtonElement;
      if ((document.getElementById("profiles") as HTMLElement).hidden) profilesToggle.click();
      const input = document.getElementById("profile-file") as HTMLInputElement;
      const dt = new DataTransfer();
      dt.items.add(file);
      input.files = dt.files;
      input.dispatchEvent(new Event("change"));
      setText(dropNote, `${file.name} sent to the Profiles panel.`);
      close.click();
      return;
    }
    rIn.value = text;
    show(3);
    scan();
    setText(dropNote, `${file.name} (${text.length} characters) scanned.`);
  }

  for (const type of ["dragenter", "dragover"]) {
    lab.addEventListener(type, (e) => {
      e.preventDefault();
      drop.classList.add("is-over");
    });
  }
  for (const type of ["dragleave", "drop"]) {
    lab.addEventListener(type, (e) => {
      e.preventDefault();
      drop.classList.remove("is-over");
    });
  }
  lab.addEventListener("drop", (e) => {
    const f = (e as DragEvent).dataTransfer?.files[0];
    if (f) void take(f);
  });

  lab.append(head, bar, flowsPane, suggestPane, consolePane, redactPane, drop, dropNote);
  document.body.append(lab);
  show(0);
  run();

  const setOpen = (on: boolean): void => {
    lab.hidden = !on;
    open?.setAttribute("aria-expanded", String(on));
  };
  open?.addEventListener("click", () => setOpen(lab.hidden === true));
  document.addEventListener("lab-ask", (e) => {
    const task = String((e as CustomEvent<{ task: string }>).detail.task);
    sTask.value = task;
    show(1);
    setOpen(true);
    run();
  });
  close.addEventListener("click", () => setOpen(false));
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !lab.hidden) setOpen(false);
  });
  setText(document.getElementById("demo-count") as HTMLElement, `${family.apps.length} apps in the real family list`);
}

build();
