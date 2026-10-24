import family from "../../core/data/family.json";

type App = (typeof family.apps)[number];
type Filter = "all" | "mcp" | "skills";

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
  li.dataset.skills = String(app.skills);
  const head = el("div", "app-head");
  const name = el("span", "app-name", app.name);
  name.translate = false;
  head.append(name, el("span", "app-kind", app.kind));
  const badges = el("div", "app-badges");
  badges.append(badge("MCP", "mcp", app.mcp), badge("Skills", "skills", app.skills), el("span", `state state-${app.state}`, STATES[app.state] ?? app.state));
  li.append(head, el("p", "app-summary", app.summary), badges);
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
const withSkills = family.apps.filter((a) => a.skills).length;

function apply(filter: Filter): void {
  let shown = 0;
  for (const { app, node } of rows) {
    const visible = filter === "all" || (filter === "mcp" ? app.mcp : app.skills);
    if (node.hidden === visible) node.hidden = !visible;
    if (visible) shown += 1;
  }
  for (const chip of chips) chip.setAttribute("aria-pressed", String(chip.dataset.filter === filter));
  count.textContent = filter === "all" ? `${family.apps.length} apps · ${withMcp} with an MCP server · ${withSkills} with skills` : `${shown} of ${family.apps.length} apps`;
}

for (const chip of chips) chip.addEventListener("click", () => apply((chip.dataset.filter as Filter) ?? "all"));
apply("all");
document.documentElement.classList.remove("booting");
