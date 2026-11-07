import { family } from "./gateway";

const ICON_IDS = ["formmcp", "formai", "formcut", "formencoder", "formlit", "formmd", "formpdf", "formskills", "formslide", "formtray", "formvfx", "formvis", "formwav"];
const SHOT_IDS = [...ICON_IDS, "mcp-agents", "mcp-safety", "mcp-hero"];

export const REEL: [string, string][] = [
  ["formcut", "cut the silences out of this interview video"],
  ["formvis", "retouch a photo and remove the background"],
  ["formwav", "clean up podcast audio and master it"],
  ["formpdf", "sign and annotate a PDF"],
  ["formslide", "build a slide deck for a talk"],
  ["formlit", "cull and develop a day of raw photos"],
  ["formmd", "take notes in markdown with a database"],
  ["formencoder", "convert a folder of videos to smaller files"],
  ["formvfx", "animate a title over footage"],
  ["formai", "install and update the Form apps"],
  ["formskills", "browse the skills the Form apps ship"],
  ["formtray", "switch between open Form app windows"],
];

export function iconUrl(id: string): string | null {
  return ICON_IDS.includes(id) ? `./assets/media/icons/${id}.png` : null;
}

export function shotUrl(id: string): string | null {
  return SHOT_IDS.includes(id) ? `./assets/media/shots/${id}.webp` : null;
}

function name(id: string): string {
  return family.apps.find((a) => a.id === id)?.name ?? id;
}

export function iconNode(id: string, size: number): HTMLElement {
  const url = iconUrl(id);
  if (url) {
    const img = document.createElement("img");
    img.className = "app-icon";
    img.src = url;
    img.width = size;
    img.height = size;
    img.alt = "";
    img.decoding = "async";
    return img;
  }
  const mono = document.createElement("span");
  mono.className = "app-icon app-mono";
  mono.style.width = `${size}px`;
  mono.style.height = `${size}px`;
  mono.textContent = name(id).replace(/^Form/i, "").slice(0, 2).toUpperCase() || "F";
  mono.setAttribute("aria-hidden", "true");
  return mono;
}

const pending: Promise<unknown>[] = [];

function track(node: HTMLElement): HTMLElement {
  if (node instanceof HTMLImageElement) pending.push(node.decode().catch(() => undefined));
  return node;
}

function decorateCards(): void {
  const cards = Array.from(document.querySelectorAll<HTMLElement>(".apps .app"));
  family.apps.forEach((app, i) => {
    const head = cards[i]?.querySelector<HTMLElement>(".app-head");
    if (!head) return;
    const left = document.createElement("span");
    left.className = "app-title";
    const nameNode = head.querySelector(".app-name");
    left.append(track(iconNode(app.id, 28)));
    if (nameNode) left.append(nameNode);
    head.prepend(left);
  });
}

function buildReel(): void {
  const header = document.querySelector<HTMLElement>("header.bar");
  if (!header) return;
  const reel = document.createElement("section");
  reel.className = "reel";
  reel.setAttribute("aria-label", "Apps an agent can reach through formMCP");
  const list = document.createElement("ul");
  list.className = "reel-list";
  for (const [id, task] of REEL) {
    const li = document.createElement("li");
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "reel-tile";
    btn.title = `Ask suggest_app: ${task}`;
    const frame = document.createElement("span");
    frame.className = "reel-frame";
    const shot = document.createElement("img");
    shot.src = shotUrl(id) ?? "";
    shot.alt = `${name(id)} window`;
    shot.width = 960;
    shot.height = 600;
    shot.decoding = "async";
    pending.push(shot.decode().catch(() => undefined));
    frame.append(shot);
    const cap = document.createElement("span");
    cap.className = "reel-cap";
    cap.append(track(iconNode(id, 20)));
    const label = document.createElement("span");
    label.className = "reel-name";
    label.textContent = name(id);
    label.translate = false;
    cap.append(label);
    btn.append(frame, cap);
    btn.addEventListener("click", () => document.dispatchEvent(new CustomEvent("lab-ask", { detail: { task } })));
    li.append(btn);
    list.append(li);
  }
  reel.append(list);
  header.after(reel);
}

const root = document.documentElement;
root.classList.add("booting");
decorateCards();
buildReel();
const failsafe = new Promise((resolve) => window.setTimeout(resolve, 1500));
void Promise.race([Promise.all(pending), failsafe]).then(() => root.classList.remove("booting"));
