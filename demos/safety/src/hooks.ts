export type HookEvent = { event: string; app: string; tool: string; arguments: unknown; client: string; transport: string; formmcp: string; ok?: boolean; ms?: number; result?: unknown };
export type Ran = { code: number | null; stdout: string; stderr: string; ms: number };
export type RunResult = { ran?: Ran; error?: string; ms: number };
export type Script = { file: string; title: string; about: string; source: string; run: (e: HookEvent) => Ran | "timeout" };

export const TIMEOUT_S = 10;
export const DENY_EXIT = 2;

export function split(line: string): string[] {
  const out: string[] = [];
  let word = "";
  let inWord = false;
  let quote: string | null = null;
  const chars = Array.from(line);
  for (let i = 0; i < chars.length; i += 1) {
    const c = chars[i];
    if (quote !== null && c === quote) quote = null;
    else if (quote === "'") word += c;
    else if (c === "\\") {
      i += 1;
      if (i >= chars.length) throw "the hook command ends with a backslash";
      word += chars[i];
    } else if (quote !== null) word += c;
    else if (c === '"' || c === "'") {
      quote = c;
      inWord = true;
    } else if (/\s/.test(c)) {
      if (inWord) {
        out.push(word);
        word = "";
        inWord = false;
      }
    } else {
      word += c;
      inWord = true;
    }
    if (quote !== null) inWord = true;
  }
  if (quote !== null) throw "the hook command has an unclosed quote";
  if (inWord) out.push(word);
  return out;
}

function spin(seed: string): number {
  let h = 7;
  for (const c of seed) h = (h * 31 + c.charCodeAt(0)) % 9973;
  return 8 + (h % 14);
}

const READ_WORDS = ["get", "list", "status", "read", "search", "find", "analyze", "probe", "extract", "verify"];
const PROTECTED = ["/.ssh", "id_rsa", "id_ed25519", ".env", "/etc/"];

const reply = (code: number, stdout: string, stderr: string, seed: string): Ran => ({ code, stdout, stderr, ms: spin(seed) });

export const SCRIPTS: Script[] = [
  {
    file: "guard.py",
    title: "Guard: no deletes, no protected paths",
    about: "Refuses tools that remove things and any call whose arguments name a key, an .env file or /etc. Prints a JSON decision.",
    source: `import json, sys

event = json.load(sys.stdin)
args = json.dumps(event["arguments"])
blocked = ["/.ssh", "id_rsa", "id_ed25519", ".env", "/etc/"]
if event["tool"].startswith(("delete", "remove", "erase")):
    print(json.dumps({"decision": "deny", "reason": event["tool"] + " removes things; ask the person first"}))
elif any(b in args for b in blocked):
    print(json.dumps({"decision": "deny", "reason": "the arguments name a protected path"}))
`,
    run: (e) => {
      const args = JSON.stringify(e.arguments);
      if (/^(delete|remove|erase)/.test(e.tool)) return reply(0, JSON.stringify({ decision: "deny", reason: `${e.tool} removes things; ask the person first` }), "", e.tool);
      if (PROTECTED.some((p) => args.includes(p))) return reply(0, JSON.stringify({ decision: "deny", reason: "the arguments name a protected path" }), "", e.tool);
      return reply(0, "", "", e.tool);
    },
  },
  {
    file: "read-only.py",
    title: "Read-only session",
    about: "Lets only reading tools through (status, list, get, read, search, find, analyze, probe, extract, verify). Everything that changes a file is refused with a reason the agent can pass on.",
    source: `import json, sys

READ = {"get", "list", "status", "read", "search", "find", "analyze", "probe", "extract", "verify"}
event = json.load(sys.stdin)
if not READ & set(event["tool"].split("_")):
    print(json.dumps({"decision": "deny", "reason": "read-only session: " + event["tool"] + " changes things"}))
`,
    run: (e) => (READ_WORDS.some((w) => e.tool.split("_").includes(w)) ? reply(0, "", "", e.tool) : reply(0, JSON.stringify({ decision: "deny", reason: `read-only session: ${e.tool} changes things` }), "", e.tool)),
  },
  {
    file: "no-deletes.sh",
    title: "No deletes (exit code 2)",
    about: "The other way to refuse: exit with code 2 and say why on stderr. The first stderr line becomes the reason.",
    source: `#!/bin/sh
input=$(cat)
case "$input" in
  *'"tool":"delete'*|*'"tool":"remove'*)
    echo "deleting is not allowed in this session" >&2
    exit 2 ;;
esac
`,
    run: (e) => (/^(delete|remove)/.test(e.tool) ? reply(DENY_EXIT, "", "deleting is not allowed in this session", e.tool) : reply(0, "", "", e.tool)),
  },
  {
    file: "chatty.sh",
    title: "Prints text instead of JSON",
    about: "A common mistake: the hook exits 0 but prints something that is not JSON. formMCP refuses the call rather than guess.",
    source: `#!/bin/sh
echo "checking the call..."
`,
    run: (e) => reply(0, "checking the call...", "", e.tool),
  },
  {
    file: "slow.sh",
    title: "Takes too long",
    about: "A hook that does not finish within 10 seconds is stopped and the call is refused. FORMMCP_HOOK_TIMEOUT changes the limit.",
    source: `#!/bin/sh
sleep 12
`,
    run: () => "timeout",
  },
  {
    file: "audit.py",
    title: "Audit trail (after hook)",
    about: "An after-call hook gets the event again with ok, ms and the redacted, shortened result. Its output is ignored; a non-zero exit is noted in the log.",
    source: `import json, sys, time

event = json.load(sys.stdin)
with open("hooks/audit.log", "a") as log:
    log.write(json.dumps({"t": time.time(), "app": event["app"], "tool": event["tool"], "ok": event.get("ok")}) + "\\n")
`,
    run: (e) => reply(0, "", "", e.tool),
  },
];

const INTERPRETERS: Record<string, string> = {
  python3: "python3: can't open file '%': [Errno 2] No such file or directory",
  python: "python: can't open file '%': [Errno 2] No such file or directory",
};
const SHELLS = ["sh", "bash", "zsh"];

function base(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function runLine(line: string, event: HookEvent): RunResult {
  let words: string[];
  try {
    words = split(line);
  } catch (e) {
    return { error: String(e), ms: 0 };
  }
  if (!words.length) return { error: "the hook command is empty", ms: 0 };
  const program = words[0];
  const name = base(program);
  if (name === "echo") return { ran: reply(0, words.slice(1).join(" "), "", event.tool), ms: 6 };
  if (name === "true") return { ran: reply(0, "", "", event.tool), ms: 5 };
  if (name === "false") return { ran: reply(1, "", "", event.tool), ms: 5 };
  const interpreter = name in INTERPRETERS || SHELLS.includes(name);
  const target = interpreter ? (words[1] ?? "") : program;
  const found = SCRIPTS.find((s) => base(target) === s.file);
  if (!interpreter && !found) return { error: `could not start ${program} (No such file or directory (os error 2))`, ms: 0 };
  if (interpreter && !target) return { ran: reply(0, "", "", event.tool), ms: 8 };
  if (!found) {
    if (name in INTERPRETERS) return { ran: reply(DENY_EXIT, "", INTERPRETERS[name].replace("%", target), event.tool), ms: 22 };
    return { ran: reply(127, "", `${name}: 0: cannot open ${target}: No such file`, event.tool), ms: 6 };
  }
  const out = found.run(event);
  if (out === "timeout") return { error: `it did not finish within ${TIMEOUT_S} s`, ms: TIMEOUT_S * 1000 };
  return { ran: out, ms: out.ms };
}

export function judge(ran: Ran): { ok: boolean; reason: string } {
  let said: Record<string, unknown> | null = null;
  if (ran.stdout) {
    try {
      const parsed: unknown = JSON.parse(ran.stdout);
      said = parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as Record<string, unknown>) : null;
      if (said === null) said = {};
    } catch {
      said = null;
    }
  }
  const decision = typeof said?.decision === "string" ? said.decision : "allow";
  const reason = typeof said?.reason === "string" ? said.reason : (ran.stderr.split("\n")[0] ?? "");
  if (ran.code === 0 && !ran.stdout) return { ok: true, reason: "" };
  if (ran.code === 0 && said === null) return { ok: false, reason: "it printed something that is not JSON" };
  if (ran.code === 0) {
    if (decision === "allow") return { ok: true, reason: "" };
    if (decision === "deny") return { ok: false, reason };
    return { ok: false, reason: `it answered decision ${JSON.stringify(decision)}; use allow or deny` };
  }
  if (ran.code === DENY_EXIT) return { ok: false, reason };
  return { ok: false, reason: `it exited with ${ran.code === null ? "a signal" : ran.code}` };
}

const LONG = 64 * 1024;

export function shorten(v: unknown): unknown {
  if (typeof v === "string") return v.length > LONG ? `[${v.length} bytes left out by formMCP]` : v;
  if (Array.isArray(v)) return v.map(shorten);
  if (v && typeof v === "object") return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, shorten(x)]));
  return v;
}
