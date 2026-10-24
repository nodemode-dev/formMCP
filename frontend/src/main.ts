import family from "../../core/data/family.json";

type App = (typeof family.apps)[number];
type Filter = "all" | "mcp";

declare const __APP_VERSION__: string;

const STATES: Record<string, string> = { live: "Live", public: "Public", prelaunch: "Coming" };

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
  if (app.mcp) {
    const sw = el("button", "switch");
    sw.type = "button";
    sw.setAttribute("role", "switch");
    sw.setAttribute("aria-checked", "false");
    sw.disabled = true;
    sw.dataset.expose = app.id;
    sw.title = "Always show this app's own tools in formMCP's tool list";
    sw.append(el("span", "switch-track"), el("span", "switch-label", "Always show tools"));
    local.append(sw);
  }
  li.append(head, el("p", "app-summary", app.summary), badges, local);
  return li;
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
type LiveApp = { id: string; installed: boolean; reach: string | null; exposed: boolean; mcp: boolean; version: string };
type ClientState = { client: string; state: "connected" | "not-connected" | "taken" | "no-client"; note: string | null };
type Snapshot = { source: string; apps: LiveApp[]; clients: ClientState[]; formportal: boolean; env_expose: boolean };
type LogEvent = { t: number; event: string; client?: string; name?: string; app?: string | null; tool?: string | null; ok?: boolean; ms?: number; via?: string; error?: string; tools?: number };

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

function showSnapshot(s: Snapshot): void {
  const byId = new Map(s.apps.map((a) => [a.id, a]));
  for (const { app, node } of rows) {
    const live = byId.get(app.id);
    if (!live) continue;
    setText(node.querySelector("[data-where]"), whereText(live));
    const sw = node.querySelector<HTMLButtonElement>("[data-expose]");
    if (sw) {
      sw.setAttribute("aria-checked", String(live.exposed));
      sw.disabled = s.env_expose;
    }
  }
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
  if (e.event === "session") return `Session started by ${e.client ?? "a client"}`;
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
        const li = el("li", `log-row${e.ok === false ? " is-fail" : ""}`);
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
  window.addEventListener("focus", () => {
    if (Date.now() - lastSnapshot > 10000) void refresh();
  });
  void refresh();
  void pollLog();
  window.setInterval(() => void pollLog(), 2000);
} else {
  setText(status, "Preview in a browser: open the formMCP app to see this computer's installs, clients and activity.");
  for (const card of document.querySelectorAll(".client")) setText(card.querySelector("[data-state]"), "Not known here");
}

document.documentElement.classList.remove("booting");
