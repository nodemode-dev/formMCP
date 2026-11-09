import { createReadStream, existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
export const DEMO_FOLDERS = ["demo", "demos/connect", "demos/safety"];
const TYPES: Record<string, string> = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".css": "text/css", ".png": "image/png", ".webp": "image/webp", ".svg": "image/svg+xml", ".json": "application/json" };

function shipped(folder: string): string[] {
  const base = join(ROOT, folder);
  const out: string[] = [];
  if (existsSync(join(base, "index.html"))) out.push("index.html");
  const walk = (dir: string): void => {
    for (const name of readdirSync(dir).sort()) {
      const path = join(dir, name);
      if (statSync(path).isDirectory()) walk(path);
      else out.push(relative(base, path).split(sep).join("/"));
    }
  };
  if (existsSync(join(base, "assets"))) walk(join(base, "assets"));
  return out;
}

function served(url: string): string | null {
  const path = decodeURIComponent(url.split("?")[0] ?? "").replace(/^\/+/, "");
  for (const folder of DEMO_FOLDERS) {
    if (!path.startsWith(`${folder}/`)) continue;
    const rest = path.slice(folder.length + 1);
    if (shipped(folder).includes(rest)) return join(ROOT, folder, rest);
  }
  return null;
}

export function demosBundle(): Plugin {
  return {
    name: "formmcp-demos-bundle",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const file = served(req.url ?? "");
        if (!file) return next();
        res.setHeader("Content-Type", TYPES[extname(file)] ?? "application/octet-stream");
        res.setHeader("Cache-Control", "no-store");
        createReadStream(file).pipe(res);
      });
    },
    generateBundle() {
      for (const folder of DEMO_FOLDERS) {
        for (const rest of shipped(folder)) this.emitFile({ type: "asset", fileName: `${folder}/${rest}`, source: readFileSync(join(ROOT, folder, rest)) });
      }
    },
  };
}
