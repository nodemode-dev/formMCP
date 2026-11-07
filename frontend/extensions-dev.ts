import { spawn } from "node:child_process";
import { createReadStream, existsSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const SAFE_HEADERS = { "X-Content-Type-Options": "nosniff", "Cache-Control": "no-store", "Cross-Origin-Resource-Policy": "cross-origin", "Access-Control-Allow-Origin": "*" };

function cli(): string | null {
  if (process.env.FORMMCP_CLI) return process.env.FORMMCP_CLI;
  for (const kind of ["release", "debug"]) {
    const path = `${ROOT}target/${kind}/formmcp-cli`;
    if (existsSync(path)) return path;
  }
  return null;
}

function run(args: string[], input = ""): Promise<string> {
  const program = cli();
  if (!program) return Promise.reject(new Error("formmcp-cli is not built; run ./formmcp cli version once"));
  return new Promise((resolve, reject) => {
    const child = spawn(program, args, { stdio: ["pipe", "pipe", "pipe"] });
    let out = "";
    let err = "";
    const timer = setTimeout(() => child.kill(), 30000);
    child.stdout.on("data", (d) => (out += d));
    child.stderr.on("data", (d) => (err += d));
    child.on("error", reject);
    child.on("close", (code) => {
      clearTimeout(timer);
      if (code === 0) resolve(out);
      else reject(new Error(err.trim() || `formmcp-cli exited with ${code}`));
    });
    child.stdin.on("error", () => undefined);
    child.stdin.end(input);
  });
}

function send(res: ServerResponse, status: number, body: string, type = "application/json"): void {
  res.writeHead(status, { ...SAFE_HEADERS, "Content-Type": type });
  res.end(body);
}

function body(req: IncomingMessage): Promise<string> {
  return new Promise((resolve, reject) => {
    let text = "";
    req.on("data", (d) => {
      text += d;
      if (text.length > 64 * 1024 * 1024) req.destroy();
    });
    req.on("end", () => resolve(text));
    req.on("error", reject);
  });
}

async function handle(req: IncomingMessage, res: ServerResponse, next: () => void): Promise<void> {
  const url = req.url ?? "";
  if (!url.startsWith("/api/extensions")) return next();
  const path = url.split("?")[0];
  try {
    if (path.startsWith("/api/extensions/files/")) {
      if (req.method !== "GET" && req.method !== "HEAD") return send(res, 404, "", "text/plain");
      const served = JSON.parse(await run(["extensions", "file", path])) as { status: number; headers: [string, string][]; file: string | null };
      if (served.status !== 200 || !served.file) {
        res.writeHead(404, Object.fromEntries(served.headers));
        return void res.end();
      }
      res.writeHead(200, Object.fromEntries(served.headers));
      if (req.method === "HEAD") return void res.end();
      return void createReadStream(served.file).on("error", () => res.end()).pipe(res);
    }
    const local = req.headers["x-formmcp"] === "1";
    if (path === "/api/extensions" && req.method === "GET") return send(res, 200, await run(["extensions", "window", "/api/extensions/files"]));
    if (path === "/api/extensions/stamp" && req.method === "GET") return send(res, 200, await run(["extensions", "stamp"]));
    if (path === "/api/extensions/requests" && req.method === "GET" && local) return send(res, 200, await run(["extensions", "take"]));
    const finish = path.match(/^\/api\/extensions\/requests\/([0-9a-f]{1,64})$/);
    if (finish && req.method === "POST" && local) {
      await run(["extensions", "finish", finish[1]], await body(req));
      return send(res, 200, "null");
    }
    return send(res, 404, "", "text/plain");
  } catch (e) {
    return send(res, 503, JSON.stringify({ error: String((e as Error).message ?? e) }));
  }
}

export function extensionsDev(): Plugin {
  return {
    name: "formmcp-extensions-dev",
    configureServer(server) {
      server.middlewares.use((req, res, next) => void handle(req, res, next));
    },
    configurePreviewServer(server) {
      server.middlewares.use((req, res, next) => void handle(req, res, next));
    },
  };
}
