import { record, remember } from "../../../demo/src/gateway";
import { answer, clientKey, SUPPORTED } from "./agent";

export const PORT = 5625;
export const PATH = "/mcp";
export const SESSIONS_MAX = 64;

export type Req = { method: string; path: string; query: string; headers: Record<string, string>; body: string };
export type Reply = { status: number; reason: string; headers: [string, string][]; body: string };

export const server = { running: false, token: "", sessions: new Map<string, string>(), order: [] as string[] };

export function address(): string {
  return `http://127.0.0.1:${PORT}${PATH}`;
}

export function newToken(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  const token = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  remember(token);
  return token;
}

function text(status: number, reason: string, body: string): Reply {
  return { status, reason, headers: [["Content-Type", "text/plain; charset=utf-8"]], body };
}

function json(status: number, reason: string, value: unknown): Reply {
  return { status, reason, headers: [["Content-Type", "application/json"]], body: JSON.stringify(value) };
}

export function localHost(value: string, port: number | null): boolean {
  const rest = value.trim().replace(/^http:\/\//, "").replace(/^https:\/\//, "");
  const hostPort = rest.split("/")[0] ?? "";
  let name: string;
  let p: string;
  if (hostPort.startsWith("[")) {
    const close = hostPort.indexOf("]");
    if (close < 0) return false;
    name = hostPort.slice(0, close + 1);
    const tail = hostPort.slice(close + 1);
    p = tail.startsWith(":") ? tail.slice(1) : "";
  } else {
    const at = hostPort.lastIndexOf(":");
    name = at < 0 ? hostPort : hostPort.slice(0, at);
    p = at < 0 ? "" : hostPort.slice(at + 1);
  }
  return ["127.0.0.1", "localhost", "[::1]"].includes(name) && (port === null || p === "" || p === String(port));
}

function same(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let acc = 0;
  for (let i = 0; i < a.length; i++) acc |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return acc === 0;
}

function param(query: string, key: string): string | null {
  for (const kv of query.split("&")) {
    const at = kv.indexOf("=");
    if (at > 0 && kv.slice(0, at) === key) return kv.slice(at + 1);
  }
  return null;
}

function refuse(reason: string, status: number, why: string): Reply {
  record("http-refused", { reason, status });
  const reply = text(status, why, reason);
  if (status === 401) reply.headers.push(["WWW-Authenticate", 'Bearer realm="formmcp"']);
  return reply;
}

function sessionId(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

export function handle(req: Req): Reply {
  if (req.path !== PATH) return text(404, "Not Found", "formMCP answers at /mcp");
  const h = req.headers;
  if (!h["host"] || !localHost(h["host"], PORT)) return refuse("host is not this computer", 403, "Forbidden");
  if (h["origin"] !== undefined && !localHost(h["origin"], null)) return refuse("origin is not this computer", 403, "Forbidden");
  const auth = h["authorization"] ?? "";
  const given = (auth.startsWith("Bearer ") ? auth.slice(7) : auth.startsWith("bearer ") ? auth.slice(7) : "").trim();
  if (!same(given, server.token)) return refuse("missing or wrong access token", 401, "Unauthorized");
  const version = h["mcp-protocol-version"];
  if (version !== undefined && !SUPPORTED.includes(version.trim())) return text(400, "Bad Request", "unsupported MCP-Protocol-Version");
  const session = h["mcp-session-id"];
  if (req.method === "GET") {
    const reply = text(405, "Method Not Allowed", "formMCP sends no server stream; POST to /mcp");
    reply.headers.push(["Allow", "POST, DELETE"]);
    return reply;
  }
  if (req.method === "DELETE") {
    if (!session) return text(400, "Bad Request", "Mcp-Session-Id is required");
    if (server.sessions.delete(session)) {
      server.order = server.order.filter((s) => s !== session);
      return { status: 200, reason: "OK", headers: [], body: "" };
    }
    return { status: 404, reason: "Not Found", headers: [], body: "" };
  }
  if (req.method !== "POST") {
    const reply = text(405, "Method Not Allowed", "POST to /mcp");
    reply.headers.push(["Allow", "POST, DELETE"]);
    return reply;
  }
  let message: unknown;
  try {
    message = JSON.parse(req.body);
  } catch {
    return json(400, "Bad Request", { jsonrpc: "2.0", id: null, error: { code: -32700, message: "Parse error" } });
  }
  const messages = (Array.isArray(message) ? message : [message]) as Record<string, unknown>[];
  const initializing = messages.some((m) => m && m.method === "initialize");
  let client = "other";
  if (initializing) {
    const asked = param(req.query, "client");
    client = asked ? clientKey(asked) : "other";
    const info = (messages.find((m) => m.method === "initialize")?.params as Record<string, unknown> | undefined)?.clientInfo as Record<string, unknown> | undefined;
    if (!asked && info && typeof info.name === "string") client = clientKey(info.name);
  } else {
    if (!session) return text(400, "Bad Request", "Mcp-Session-Id is required after initialize");
    const known = server.sessions.get(session);
    if (known === undefined) return text(404, "Not Found", "this session is over; initialize again");
    client = known;
  }
  const replies = messages.map((m) => answer(client, "http", m)).filter((r): r is Record<string, unknown> => r !== null);
  let created: string | null = null;
  if (initializing) {
    created = sessionId();
    server.sessions.set(created, client);
    server.order.push(created);
    while (server.order.length > SESSIONS_MAX) {
      const old = server.order.shift();
      if (old) server.sessions.delete(old);
    }
  }
  if (!replies.length) return { status: 202, reason: "Accepted", headers: [], body: "" };
  const reply = json(200, "OK", Array.isArray(message) ? replies : replies[0]);
  if (created) reply.headers.push(["Mcp-Session-Id", created]);
  return reply;
}
