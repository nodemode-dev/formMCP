import family from "../../core/data/family.json";

type App = (typeof family.apps)[number];
type Filter = "all" | "mcp";

declare const __APP_VERSION__: string;

const STATES: Record<string, string> = { live: "Live", public: "Public", prelaunch: "Coming" };
const AGENTS: Record<string, string> = { claude: "Claude Code", codex: "Codex", cursor: "Cursor", other: "other agents" };

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
type LiveApp = { id: string; installed: boolean; reach: string | null; exposed: boolean; mcp: boolean; version: string; off_for: string[]; hidden_tools: number };
type ClientState = { client: string; state: "connected" | "not-connected" | "taken" | "no-client"; note: string | null };
type HttpState = { running: boolean; port: number; address: string };
type Snapshot = { source: string; apps: LiveApp[]; clients: ClientState[]; formportal: boolean; env_expose: boolean; http: HttpState };
type ToolRow = { name: string; description: string; read_only: boolean; on: boolean };
type ToolList = { app: string; loaded: number | null; tools: ToolRow[] };
type LogEvent = { t: number; event: string; client?: string; name?: string; app?: string | null; tool?: string | null; ok?: boolean; ms?: number; via?: string; error?: string; tools?: number; as?: string; transport?: string; port?: number; status?: number; reason?: string };

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

function showAgent(): void {
  const agent = agentPick.value;
  for (const { app, node } of rows) {
    const sw = node.querySelector<HTMLButtonElement>("[data-for]");
    if (!sw) continue;
    sw.setAttribute("aria-checked", String(!offFor.get(app.id)?.has(agent)));
    setText(sw.querySelector(".switch-label"), `On for ${AGENTS[agent] ?? agent}`);
    sw.disabled = !invoke;
  }
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
  if (e.event === "http-start") return `HTTP started on 127.0.0.1:${e.port ?? ""}`;
  if (e.event === "http-stop") return `HTTP stopped on 127.0.0.1:${e.port ?? ""}`;
  if (e.event === "http-refused") return `HTTP request refused (${e.status ?? ""}): ${e.reason ?? ""}`;
  if (e.event === "app-start") return e.ok ? `Started ${e.app} through ${e.via} (${e.tools ?? 0} tools, ${e.ms} ms)` : `Could not start ${e.app} through ${e.via}: ${e.error ?? ""}`;
  const parts = [e.name ?? "call", e.app, e.tool].filter(Boolean).join(" · ");
  return `${parts} · ${e.ok ? "ok" : "failed"} · ${e.ms ?? 0} ms`;
}

async function pollLog(): Promise<void> {
  if (!invoke) return;
  try {
    const { events, next } = (await invoke("log_tail", { from: logNext })) as { events: LogEvent[]; next: number };
    logNext = next;
    if (events.length) {
      const atEnd = logList.scrollTop + logList.clientHeight >= logList.scrollHeight - 4;
      const items = events.map((e) => {
        const li = el("li", `log-row${e.ok === false || e.event === "http-refused" ? " is-fail" : ""}`);
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
