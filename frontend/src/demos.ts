type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

export const DEMOS: { id: string; page: string; title: string }[] = [
  { id: "window", page: "demo/index.html", title: "The window and the Lab" },
  { id: "connect", page: "demos/connect/index.html", title: "Connecting agents" },
  { id: "safety", page: "demos/safety/index.html", title: "Safety and activity" },
];

let openWith: ((id: string) => Promise<string>) | null = null;

export async function demoCommand(command: string, args: Record<string, unknown>): Promise<unknown> {
  if (command !== "demo.open") throw new Error(`FormMCP has no window command ${command}`);
  const id = String(args.id ?? "");
  if (!openWith) throw new Error("this window has no demos");
  return { id, title: await openWith(id) };
}

export function mountDemos(invoke: Invoke | undefined, say: (text: string) => void): void {
  const toggle = document.querySelector<HTMLButtonElement>("[data-demos-toggle]");
  const box = document.querySelector<HTMLElement>("[data-demos]");
  if (!toggle || !box) return;

  const open = async (id: string): Promise<string> => {
    const demo = DEMOS.find((d) => d.id === id);
    if (!demo) throw new Error(`there is no demo ${JSON.stringify(id)}; the demos are ${DEMOS.map((d) => d.id).join(", ")}`);
    if (invoke) await invoke("open_demo", { id });
    else window.open(`/${demo.page}`, `formmcp-demo-${demo.id}`);
    say(`Demo opened in its own window: ${demo.title}. It is a sandboxed copy; nothing you do there changes FormMCP.`);
    return demo.title;
  };
  openWith = open;

  const others = ["profiles", "safety", "ext-panel"];
  const close = (): void => {
    box.hidden = true;
    toggle.setAttribute("aria-expanded", "false");
  };
  toggle.addEventListener("click", () => {
    const show = box.hidden;
    if (show) {
      for (const id of others) {
        const panel = document.getElementById(id);
        if (panel) panel.hidden = true;
      }
      for (const id of ["profiles-toggle", "safety-toggle"]) document.getElementById(id)?.setAttribute("aria-expanded", "false");
    }
    box.hidden = !show;
    toggle.setAttribute("aria-expanded", String(show));
  });
  for (const id of ["profiles-toggle", "safety-toggle"]) document.getElementById(id)?.addEventListener("click", close);
  const extPanel = document.getElementById("ext-panel");
  if (extPanel) new MutationObserver(() => !extPanel.hidden && close()).observe(extPanel, { attributes: true, attributeFilter: ["hidden"] });
  for (const button of box.querySelectorAll<HTMLButtonElement>("[data-demo-open]")) {
    button.addEventListener("click", () => void open(button.dataset.demoOpen ?? "").catch((e) => say(`Could not open the demo: ${String((e as Error).message ?? e)}`)));
  }
}
