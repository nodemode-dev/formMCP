import { family, isOff, profileHidesApp, sampleTools, settings, toolHidden } from "../../../demo/src/gateway";
import { AGENT_LABEL, initialize, toolsCall } from "./agent";
import { PORT, address, handle, server } from "./httpd";
import { decorateCards, iconNode, pending } from "./media";
import {
  LABELS,
  PLATFORMS,
  changed,
  diffLines,
  nextPlan,
  pathOf,
  reset,
  stateOf,
  world,
  type ClientKey,
  type Link,
  type Platform,
} from "./world";

const LINK: Record<Link, string> = { connected: "Connected", "not-connected": "Not connected", taken: "Another server uses the name", "no-client": "Not installed" };
const CLIENTS: ClientKey[] = ["claude", "codex", "cursor"];
const AGENTS = ["claude", "codex", "cursor", "other"];

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls = "", text = ""): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text) node.textContent = text;
  return node;
}

function setText(node: Element, text: string): void {
  if (node.textContent !== text) node.textContent = text;
}

function button(label: string, cls = "btn btn-small"): HTMLButtonElement {
  const b = el("button", cls, label);
  b.type = "button";
  return b;
}

function field(label: string, input: HTMLElement): HTMLLabelElement {
  const wrap = el("label", "wb-field");
  wrap.append(el("span", "", label), input);
  return wrap;
}

function textInput(value: string, label: string): HTMLInputElement {
  const i = el("input");
  i.type = "text";
  i.value = value;
  i.spellcheck = false;
  i.setAttribute("aria-label", label);
  return i;
}

function download(name: string, text: string, type: string): void {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const a = el("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function paintClients(): void {
  for (const c of CLIENTS) {
    const card = document.querySelector<HTMLElement>(`.client[data-client="${c}"]`);
    if (!card) continue;
    const s = stateOf(c);
    card.dataset.link = s.state;
    const label = card.querySelector("[data-state]");
    if (label) setText(label, LINK[s.state]);
    const btn = card.querySelector<HTMLButtonElement>("[data-connect]");
    if (!btn) continue;
    setText(btn, s.state === "connected" ? "Disconnect" : "Connect");
    btn.disabled = s.state === "taken" || s.state === "no-client";
    btn.title = s.note ?? "";
  }
}

function buildSetup(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  let cur: ClientKey = "cursor";

  const head = el("div", "wb-head");
  const chips = el("div", "chips");
  chips.setAttribute("role", "group");
  const chipButtons = CLIENTS.map((c) => {
    const b = el("button", "chip", LABELS[c]);
    b.type = "button";
    b.setAttribute("aria-pressed", String(c === cur));
    b.translate = false;
    b.addEventListener("click", () => {
      cur = c;
      paint();
    });
    chips.append(b);
    return b;
  });
  const stateLabel = el("span", "wb-state");
  const spacer = el("span", "wb-spacer");
  const platform = el("select");
  for (const [k, v] of Object.entries(PLATFORMS)) {
    const o = el("option", "", v.label);
    o.value = k;
    platform.append(o);
  }
  platform.value = world.platform;
  platform.setAttribute("aria-label", "Computer");
  platform.title = "Sets the home folder and launcher of the virtual computer and starts its sample files again";
  const launcher = textInput(world.launcher, "Launcher program");
  launcher.title = "The program written into each agent's entry: FORMMCP_LAUNCHER, else the running formMCP program";
  const shellBtn = button("Shell settings");
  head.append(chips, stateLabel, spacer, field("Computer", platform), field("Launcher", launcher), shellBtn);

  const cols = el("div", "wb-cols");
  const left = el("div", "wb-col");
  const leftTitle = el("div", "wb-title");
  const disk = el("textarea", "wb-code");
  disk.spellcheck = false;
  disk.setAttribute("aria-label", "The agent's file as it is on disk");
  const claudeNow = el("pre", "wb-code");
  left.append(leftTitle, disk, claudeNow);
  const right = el("div", "wb-col");
  const rightTitle = el("div", "wb-title");
  const result = el("div", "wb-code wb-diff");
  result.tabIndex = 0;
  right.append(rightTitle, result);
  const shellPanel = el("div", "wb-shell");
  shellPanel.hidden = true;
  cols.append(left, right, shellPanel);

  const foot = el("div", "wb-foot");
  const note = el("span", "wb-note");
  const openBtn = button("Open a file");
  const fileIn = el("input");
  fileIn.type = "file";
  fileIn.hidden = true;
  const saveBtn = button("Save result");
  const sampleA = button("");
  const sampleB = button("");
  const sampleC = button("");
  const resetBtn = button("Start over");
  foot.append(note, sampleA, sampleB, sampleC, openBtn, saveBtn, resetBtn, fileIn);
  pane.append(head, cols, foot);

  const shellRows = world.shell.map((v) => {
    const row = el("div", "wb-shell-row");
    const sw = el("button", "switch");
    sw.type = "button";
    sw.setAttribute("role", "switch");
    sw.append(el("span", "switch-track"), el("span", "switch-label", v.name));
    const input = textInput(v.value, `${v.name} value`);
    sw.addEventListener("click", () => {
      v.on = !v.on;
      paint();
      changed();
    });
    input.addEventListener("input", () => {
      v.value = input.value;
      paint();
    });
    row.append(sw, input);
    return { v, sw, input, row };
  });
  shellPanel.append(el("p", "wb-note", "Settings that are set in the shell that starts formMCP. Connect copies the ones that are on into the entry, so the agent's server sees them."), ...shellRows.map((r) => r.row));
  shellBtn.addEventListener("click", () => {
    shellPanel.hidden = !shellPanel.hidden;
    shellBtn.setAttribute("aria-expanded", String(!shellPanel.hidden));
  });

  function paint(): void {
    chipButtons.forEach((b, i) => b.setAttribute("aria-pressed", String(CLIENTS[i] === cur)));
    const s = stateOf(cur);
    setText(stateLabel, LINK[s.state]);
    stateLabel.dataset.link = s.state;
    stateLabel.title = s.note ?? "";
    if (platform.value !== world.platform) platform.value = world.platform;
    if (launcher.value !== world.launcher) launcher.value = world.launcher;
    shellBtn.textContent = `Shell settings (${world.shell.filter((v) => v.on).length} on)`;
    for (const r of shellRows) {
      r.sw.setAttribute("aria-checked", String(r.v.on));
      if (r.input.value !== r.v.value) r.input.value = r.v.value;
    }
    const isClaude = cur === "claude";
    disk.hidden = isClaude;
    claudeNow.hidden = !isClaude;
    const path = pathOf(cur);
    const next = nextPlan(cur);
    let current = "";
    if (isClaude) {
      setText(leftTitle, "Claude Code, user scope (read with: claude mcp get formmcp)");
      const c = world.claude;
      setText(claudeNow, !world.claudeInstalled ? "The claude command is not installed on this computer." : c ? `formmcp\n  Scope: User\n  Command: ${c.command}\n  Args: ${c.args.join(" ")}${c.env.length ? `\n  Environment: ${c.env.map(([k, v]) => `${k}=${v}`).join(", ")}` : ""}\n\nOnly a user-scope entry counts as connected. A project's .mcp.json (like this repository's) is left alone.` : "No server named formmcp in the user scope.\n\nOnly a user-scope entry counts as connected. A project's .mcp.json (like this repository's) is left alone.");
    } else {
      const text = cur === "codex" ? world.codex : world.cursor;
      current = text ?? "";
      setText(leftTitle, text === null ? `${path} does not exist yet` : `On disk now: ${path}`);
      if (document.activeElement !== disk && disk.value !== current) disk.value = current;
      disk.placeholder = "The file does not exist yet. Type or open one.";
    }
    setText(rightTitle, `If you press ${next.on ? "Connect" : "Disconnect"}${isClaude ? ", Claude Code is asked to" : ""}`);
    const rows: HTMLElement[] = [];
    if (next.plan.error) {
      rows.push(el("div", "wb-line is-fail", next.plan.error));
    } else if (isClaude) {
      for (const c of next.plan.commands) rows.push(el("div", "wb-line", `$ ${c}`));
      if (!next.plan.commands.length) rows.push(el("div", "wb-line", "Nothing to run."));
    } else if (next.plan.text === null) {
      rows.push(el("div", "wb-line", "Nothing to change in this file."));
    } else {
      for (const d of diffLines(current, next.plan.text)) {
        const line = el("div", `wb-line wb-${d.kind}`);
        line.append(el("span", "wb-gutter", d.kind === "add" ? "+" : d.kind === "del" ? "-" : ""), el("span", "wb-text", d.text || " "));
        rows.push(line);
      }
    }
    result.replaceChildren(...rows);
    const last = world.backups[world.backups.length - 1];
    setText(note, isClaude ? "Claude Code stores the entry itself; formMCP only runs these commands." : next.plan.backup && !next.plan.error ? `Backs the file up first as ${path?.split(/[\\/]/).pop()}.bak-<seconds>${last ? `. Last backup: ${last.path.split(/[\\/]/).pop()}` : ""}` : last ? `Last backup: ${last.path.split(/[\\/]/).pop()}` : "No file yet, so nothing to back up.");
    note.title = world.backups.map((b) => b.path).join("\n");
    saveBtn.hidden = isClaude;
    openBtn.hidden = isClaude;
    const samples = isClaude
      ? [["Pretend claude is missing", () => (world.claudeInstalled = !world.claudeInstalled)]]
      : cur === "codex"
        ? [["No file yet", () => (world.codex = null)], ["Empty file", () => (world.codex = "")]]
        : [["Taken by another server", () => (world.cursor = '{\n  "mcpServers": {\n    "formmcp": {\n      "command": "node",\n      "args": ["server.js"]\n    }\n  }\n}\n')], ["Not valid JSON", () => (world.cursor = '{ "mcpServers": { "team-docs": ')], ["No file yet", () => (world.cursor = null)]];
    [sampleA, sampleB, sampleC].forEach((b, i) => {
      const s = samples[i] as [string, () => unknown] | undefined;
      b.hidden = !s;
      if (s) {
        setText(b, i === 0 && isClaude ? (world.claudeInstalled ? s[0] : "Claude is installed again") : s[0]);
        b.onclick = () => {
          s[1]();
          paint();
          paintClients();
          changed();
        };
      }
    });
  }

  disk.addEventListener("input", () => {
    if (cur === "codex") world.codex = disk.value;
    else if (cur === "cursor") world.cursor = disk.value;
    paint();
    paintClients();
    changed();
  });
  platform.addEventListener("change", () => {
    reset(platform.value as Platform);
    paint();
    paintClients();
    changed();
  });
  launcher.addEventListener("input", () => {
    world.launcher = launcher.value;
    paint();
  });
  resetBtn.addEventListener("click", () => {
    reset(world.platform);
    paint();
    paintClients();
    changed();
  });
  openBtn.addEventListener("click", () => fileIn.click());
  const load = async (file: File | undefined): Promise<void> => {
    if (!file) return;
    if (file.size > 512 * 1024) {
      setText(note, `${file.name} is too large for the demo.`);
      return;
    }
    const text = await file.text();
    if (cur === "codex") world.codex = text;
    else world.cursor = text;
    paint();
    paintClients();
    changed();
  };
  fileIn.addEventListener("change", () => {
    const f = fileIn.files?.[0];
    fileIn.value = "";
    void load(f);
  });
  for (const t of [left, disk]) {
    t.addEventListener("dragover", (e) => e.preventDefault());
    t.addEventListener("drop", (e) => {
      e.preventDefault();
      void load((e as DragEvent).dataTransfer?.files?.[0]);
    });
  }
  saveBtn.addEventListener("click", () => {
    const next = nextPlan(cur);
    const text = next.plan.text ?? (cur === "codex" ? world.codex : world.cursor) ?? "";
    download(cur === "codex" ? "config.toml" : "mcp.json", text, cur === "codex" ? "text/plain" : "application/json");
  });
  return { pane, paint };
}

function buildHttp(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  const cols = el("div", "wb-cols wb-cols-http");
  const left = el("div", "wb-col");
  const info = el("div", "wb-info");
  const addr = el("code", "wb-addr");
  const tokenText = el("code", "wb-token");
  const showBtn = button("Show token");
  const stateText = el("span", "wb-state");
  info.append(addr, tokenText, showBtn, stateText);

  const presets = el("div", "wb-presets");
  const method = el("select");
  for (const m of ["POST", "GET", "DELETE"]) {
    const o = el("option", "", m);
    o.value = m;
    method.append(o);
  }
  const host = textInput(`127.0.0.1:${PORT}`, "Host header");
  const origin = textInput("", "Origin header");
  origin.placeholder = "none";
  const auth = textInput("Bearer {token}", "Authorization header");
  const version = textInput("2025-06-18", "MCP-Protocol-Version header");
  const session = textInput("", "Mcp-Session-Id header");
  session.placeholder = "none";
  const agent = textInput("", "client query");
  agent.placeholder = "none, or cursor";
  const body = el("textarea", "wb-code wb-body-in");
  body.spellcheck = false;
  body.setAttribute("aria-label", "Request body");
  const fields = el("div", "wb-fields");
  fields.append(field("Method", method), field("Host", host), field("Origin", origin), field("Protocol", version), field("Authorization", auth), field("Session", session), field("?client=", agent));
  const send = button("Send", "btn btn-small wb-send");
  const curl = el("code", "wb-curl");
  left.append(info, presets, fields, body, el("div", "wb-sendrow"));
  (left.lastElementChild as HTMLElement).append(send, curl);

  const right = el("div", "wb-col");
  const rtitle = el("div", "wb-title", "What 127.0.0.1 answers");
  const out = el("pre", "wb-code wb-out");
  out.tabIndex = 0;
  right.append(rtitle, out);
  cols.append(left, right);
  pane.append(cols);

  let shown = false;
  let previous = "";
  let seen = server.token;
  const initBody = (name: string): string => JSON.stringify({ jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name, version: "1.0.0" } } });
  const callBody = (name: string, args: unknown): string => JSON.stringify({ jsonrpc: "2.0", id: 2, method: "tools/call", params: { name, arguments: args } });
  const listBody = JSON.stringify({ jsonrpc: "2.0", id: 2, method: "tools/list" });

  const set = (p: { method?: string; origin?: string; host?: string; auth?: string; version?: string; session?: string; agent?: string; body?: string }): void => {
    method.value = p.method ?? "POST";
    host.value = p.host ?? `127.0.0.1:${PORT}`;
    origin.value = p.origin ?? "";
    auth.value = p.auth ?? "Bearer {token}";
    version.value = p.version ?? "2025-06-18";
    session.value = p.session ?? session.value;
    agent.value = p.agent ?? "";
    body.value = p.body ?? "";
  };

  const specs: [string, () => void][] = [
    ["Initialize", () => set({ body: initBody("cursor-vscode"), session: "" })],
    ["List tools", () => set({ body: listBody })],
    ["suggest_app", () => set({ body: callBody("suggest_app", { task: "cut the silences out of this interview video" }) })],
    ["Wrong token", () => set({ auth: "Bearer 0123456789abcdef0123456789abcdef", body: listBody })],
    ["Previous token", () => set({ auth: "Bearer {old}", body: listBody })],
    ["A web page's Origin", () => set({ origin: "https://example.com", body: listBody })],
    ["Wrong Host", () => set({ host: "formmcp.example:5625", body: listBody })],
    ["GET", () => set({ method: "GET" })],
    ["Old protocol", () => set({ version: "2024-01-01", body: listBody })],
    ["No session", () => set({ session: " ", body: listBody })],
    ["End session", () => set({ method: "DELETE", body: "" })],
  ];
  for (const [label, fill] of specs) {
    const chip = el("button", "chip", label);
    chip.type = "button";
    chip.addEventListener("click", () => {
      const keep = session.value;
      fill();
      if (label === "No session") session.value = "";
      else if (label !== "Initialize" && !session.value.trim()) session.value = keep;
      run();
    });
    presets.append(chip);
  }

  const resolve = (text: string, masked: boolean): string => text.replace(/\{token\}/g, masked ? "$(./formmcp cli http token)" : server.token).replace(/\{old\}/g, masked ? "$(old token)" : previous || "0".repeat(64));

  function run(): void {
    const headers: Record<string, string> = {};
    const put = (k: string, v: string): void => {
      if (v.trim()) headers[k] = v.trim();
    };
    put("host", host.value);
    put("origin", origin.value);
    put("authorization", resolve(auth.value, false));
    put("mcp-protocol-version", version.value);
    put("mcp-session-id", session.value);
    let text: string;
    let tone = "";
    if (!server.running) {
      text = `Connection refused\nNothing listens on 127.0.0.1:${PORT} while HTTP is off. Press Start in the HTTP bar.`;
      tone = "is-fail";
    } else {
      const q = agent.value.trim();
      const reply = handle({ method: method.value, path: "/mcp", query: q ? `client=${q}` : "", headers, body: body.value });
      const sid = reply.headers.find(([k]) => k === "Mcp-Session-Id")?.[1];
      if (sid) session.value = sid;
      let pretty = reply.body;
      try {
        pretty = JSON.stringify(JSON.parse(reply.body), null, 2);
      } catch {
        pretty = reply.body;
      }
      text = `HTTP/1.1 ${reply.status} ${reply.reason}\n${reply.headers.map(([k, v]) => `${k}: ${v}`).join("\n")}${reply.headers.length ? "\n" : ""}\n${pretty}`;
      tone = reply.status >= 400 ? "is-fail" : "is-ok";
    }
    out.className = `wb-code wb-out ${tone}`;
    out.textContent = text;
    out.scrollTop = 0;
    paint();
    changed();
  }

  function paint(): void {
    if (server.token !== seen) {
      previous = seen;
      seen = server.token;
    }
    setText(addr, address());
    setText(tokenText, shown ? server.token : `${"•".repeat(12)}${server.token.slice(-4)}`);
    showBtn.textContent = shown ? "Hide token" : "Show token";
    setText(stateText, server.running ? "Serving" : "Off");
    stateText.dataset.link = server.running ? "connected" : "not-connected";
    const parts = ["curl -s"];
    if (method.value !== "POST") parts.push(`-X ${method.value}`);
    const q = agent.value.trim();
    parts.push(`"http://${host.value.trim() || "127.0.0.1"}/mcp${q ? `?client=${q}` : ""}"`);
    if (origin.value.trim()) parts.push(`-H "Origin: ${origin.value.trim()}"`);
    parts.push(`-H "Authorization: ${resolve(auth.value, true)}"`);
    if (version.value.trim()) parts.push(`-H "MCP-Protocol-Version: ${version.value.trim()}"`);
    if (session.value.trim()) parts.push(`-H "Mcp-Session-Id: ${session.value.trim()}"`);
    if (method.value === "POST" && body.value.trim()) parts.push(`-d '${body.value.trim()}'`);
    setText(curl, parts.join(" "));
  }

  showBtn.addEventListener("click", () => {
    shown = !shown;
    paint();
  });
  send.addEventListener("click", run);
  for (const i of [method, host, origin, auth, version, session, agent, body]) i.addEventListener("input", paint);
  set({ body: initBody("cursor-vscode") });
  return { pane, paint };
}

type Cell = { node: HTMLElement; text: Text };

function buildView(): { pane: HTMLElement; paint: () => void } {
  const pane = el("div", "wb-pane");
  const cols = el("div", "wb-cols wb-cols-view");
  const left = el("div", "wb-col");
  const grid = el("div", "wb-matrix");
  const headRow = el("div", "wb-mrow wb-mhead");
  headRow.append(el("span", "", "MCP apps"));
  const heads = AGENTS.map((a) => {
    const h = el("span", "wb-mcell");
    const t = el("strong", "", AGENT_LABEL[a] === "other agents" ? "Other" : AGENT_LABEL[a]);
    t.translate = false;
    const p = el("small", "");
    h.append(t, p);
    headRow.append(h);
    return p;
  });
  grid.append(headRow);
  const rows = family.apps
    .filter((a) => a.mcp)
    .map((app) => {
      const r = el("div", "wb-mrow");
      const who = el("span", "wb-mwho");
      const nm = el("span", "", app.name);
      nm.translate = false;
      who.append(iconNode(app.id, 18), nm);
      r.append(who);
      const cells: Cell[] = AGENTS.map(() => {
        const c = el("span", "wb-mcell wb-pill");
        const text = document.createTextNode("");
        c.append(text);
        r.append(c);
        return { node: c, text };
      });
      grid.append(r);
      return { app, cells };
    });
  const summary = el("div", "wb-note wb-summary");
  left.append(grid, summary);

  const right = el("div", "wb-col");
  const form = el("div", "wb-ask");
  const who = el("select");
  AGENTS.forEach((a) => {
    const o = el("option", "", AGENT_LABEL[a]);
    o.value = a;
    who.append(o);
  });
  who.setAttribute("aria-label", "Ask as");
  const toolPick = el("select");
  for (const t of ["family_apps", "suggest_app", "app_tools", "call_app", "app_resources"]) {
    const o = el("option", "", t);
    o.value = t;
    toolPick.append(o);
  }
  toolPick.setAttribute("aria-label", "Tool");
  toolPick.value = "call_app";
  const args = el("textarea", "wb-code wb-args");
  args.spellcheck = false;
  args.setAttribute("aria-label", "Arguments");
  args.value = '{ "app": "formcut", "tool": "formcut_render", "arguments": { "preset": "web 1080p" } }';
  const go = button("Ask");
  const profilesBtn = button("Profiles panel");
  form.append(field("As", who), field("Tool", toolPick), go, profilesBtn);
  const chips = el("div", "wb-presets");
  const asks: [string, string, string][] = [
    ["Render a cut", "call_app", '{ "app": "formcut", "tool": "formcut_render", "arguments": { "preset": "web 1080p" } }'],
    ["Add a layer", "call_app", '{ "app": "formvis", "tool": "add_layer", "arguments": { "name": "Sky" } }'],
    ["Open a PDF app", "app_tools", '{ "app": "formpdf" }'],
    ["Its tools", "app_tools", '{ "app": "formwav" }'],
    ["Which app?", "suggest_app", '{ "task": "clean up podcast audio and master it" }'],
    ["The family", "family_apps", '{ "mcp_only": true }'],
  ];
  const out = el("pre", "wb-code wb-out");
  out.tabIndex = 0;
  const started = new Set<string>();
  const ask = (): void => {
    const agentKey = who.value;
    let parsed: Record<string, unknown>;
    try {
      parsed = JSON.parse(args.value || "{}") as Record<string, unknown>;
    } catch (e) {
      out.className = "wb-code wb-out is-fail";
      out.textContent = `The arguments are not JSON: ${String((e as Error).message)}`;
      return;
    }
    if (!started.has(agentKey)) {
      started.add(agentKey);
      initialize(agentKey, "stdio", agentKey === "other" ? "some-agent" : { claude: "claude-code", codex: "codex-mcp-client", cursor: "cursor-vscode" }[agentKey] ?? agentKey, "1.0.0");
    }
    const answer = toolsCall(agentKey, "stdio", toolPick.value, parsed);
    const first = (answer.content as { text: string }[])[0]?.text ?? "";
    let shownText = first;
    try {
      shownText = JSON.stringify(JSON.parse(first), null, 2);
    } catch {
      shownText = first;
    }
    out.className = `wb-code wb-out ${answer.isError ? "is-fail" : "is-ok"}`;
    out.textContent = `${answer.isError ? "isError: true" : "isError: false"}  (${AGENT_LABEL[agentKey]}, over stdio)\n\n${shownText}`;
    out.scrollTop = 0;
    changed();
  };
  for (const [label, tool, a] of asks) {
    const chip = el("button", "chip", label);
    chip.type = "button";
    chip.addEventListener("click", () => {
      toolPick.value = tool;
      args.value = a;
      ask();
    });
    chips.append(chip);
  }
  go.addEventListener("click", ask);
  profilesBtn.title = "Opens the window's Profiles panel, where profiles are saved, used and imported";
  profilesBtn.addEventListener("click", () => {
    const toggle = document.getElementById("profiles-toggle") as HTMLButtonElement | null;
    const box = document.getElementById("profiles");
    if (toggle && box?.hidden) toggle.click();
    document.getElementById("tools")?.scrollIntoView({ block: "start" });
  });
  right.append(form, chips, args, out);
  cols.append(left, right);
  pane.append(cols);

  function paint(): void {
    AGENTS.forEach((a, i) => {
      const name = settings.use_profile[a];
      setText(heads[i], name ? `profile ${name}` : "no profile");
    });
    const onCount = AGENTS.map(() => 0);
    for (const { app, cells } of rows) {
      AGENTS.forEach((a, i) => {
        const cell = cells[i];
        const byProfile = profileHidesApp(a, app.id);
        const byAgent = !!settings.off[a]?.includes(app.id);
        const off = isOff(a, app.id);
        const hiddenTools = off ? 0 : sampleTools(app).filter((t) => toolHidden(a, app.id, t.name)).length;
        const state = byProfile ? "profile" : byAgent ? "agent" : hiddenTools ? "partly" : "on";
        const label = byProfile ? "off by profile" : byAgent ? "off" : hiddenTools ? `${hiddenTools} tool${hiddenTools === 1 ? "" : "s"} off` : "on";
        if (cell.node.dataset.state !== state) cell.node.dataset.state = state;
        if (cell.text.data !== label) cell.text.data = label;
        cell.node.title = byProfile ? `Turned off for ${AGENT_LABEL[a]} by the profile ${settings.use_profile[a] ?? ""}` : byAgent ? `Turned off for ${AGENT_LABEL[a]} with its On for switch` : hiddenTools ? `${hiddenTools} of its tools are hidden from ${AGENT_LABEL[a]}` : `Offered to ${AGENT_LABEL[a]}`;
        if (!off) onCount[i] += 1;
      });
    }
    setText(summary, `Offered of ${rows.length} apps with an MCP server: ${AGENTS.map((a, i) => `${AGENT_LABEL[a] === "other agents" ? "other" : AGENT_LABEL[a]} ${onCount[i]}`).join(" · ")}. Tool counts come from the demo's sample tool lists.`);
  }
  return { pane, paint };
}

function build(): void {
  const wb = document.getElementById("wb") as HTMLElement;
  const tabs = el("div", "chips wb-tabs");
  tabs.setAttribute("role", "group");
  const names = ["Agent setup", "HTTP and token", "What each agent sees"];
  const parts = [buildSetup(), buildHttp(), buildView()];
  const buttons = names.map((n, i) => {
    const b = el("button", "chip", n);
    b.type = "button";
    b.setAttribute("aria-pressed", String(i === 0));
    b.addEventListener("click", () => select(i));
    tabs.append(b);
    return b;
  });
  const body = el("div", "wb-body");
  parts.forEach((p, i) => {
    p.pane.hidden = i !== 0;
    body.append(p.pane);
  });
  wb.append(tabs, body);
  function select(i: number): void {
    buttons.forEach((b, j) => b.setAttribute("aria-pressed", String(i === j)));
    parts.forEach((p, j) => (p.pane.hidden = i !== j));
    parts[i].paint();
  }
  const all = (): void => {
    paintClients();
    for (const p of parts) p.paint();
  };
  document.addEventListener("world-change", all);
  all();
}

build();
decorateCards();
const root = document.documentElement;
root.classList.add("booting");
const failsafe = new Promise((resolve) => window.setTimeout(resolve, 1500));
void Promise.race([Promise.all(pending), failsafe]).then(() => root.classList.remove("booting"));
