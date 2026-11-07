import { family } from "../../../demo/src/gateway";

const ICON_IDS = ["formmcp", "formai", "formcut", "formencoder", "formlit", "formmd", "formpdf", "formskills", "formslide", "formtray", "formvfx", "formvis", "formwav"];

export const pending: Promise<unknown>[] = [];

export function nameOf(id: string): string {
  return family.apps.find((a) => a.id === id)?.name ?? id;
}

export function iconNode(id: string, size: number): HTMLElement {
  if (ICON_IDS.includes(id)) {
    const img = document.createElement("img");
    img.className = "app-icon";
    img.src = `./assets/media/icons/${id}.png`;
    img.width = size;
    img.height = size;
    img.alt = "";
    img.decoding = "async";
    pending.push(img.decode().catch(() => undefined));
    return img;
  }
  const mono = document.createElement("span");
  mono.className = "app-icon app-mono";
  mono.style.width = `${size}px`;
  mono.style.height = `${size}px`;
  mono.textContent = nameOf(id).replace(/^Form/i, "").slice(0, 2).toUpperCase() || "F";
  mono.setAttribute("aria-hidden", "true");
  return mono;
}

export function decorateCards(): void {
  const cards = Array.from(document.querySelectorAll<HTMLElement>(".apps .app"));
  family.apps.forEach((app, i) => {
    const head = cards[i]?.querySelector<HTMLElement>(".app-head");
    if (!head) return;
    const left = document.createElement("span");
    left.className = "app-title";
    const nameNode = head.querySelector(".app-name");
    left.append(iconNode(app.id, 28));
    if (nameNode) left.append(nameNode);
    head.prepend(left);
  });
}
