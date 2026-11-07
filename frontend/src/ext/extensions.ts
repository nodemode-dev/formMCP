import { ERR, ExtensionHost, HostError, PERMISSION_WORDS, type ExtensionRecord, type Frame, type Permission, type Theme } from "./host-bridge";

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
type FamilyApp = { id: string; name: string; kind: string; summary: string; mcp: boolean; state: string };
type Listing = { stamp: number[]; extensions: ExtensionRecord[]; root?: string };
type Request = { rid: string; id: string; command: string; args?: unknown; client?: string };
type Op = { command: string; args?: unknown };

const PRODUCT = "formmcp";
const APP = "FormMCP";
const PORTAL = `formai://extensions?product=${PRODUCT}`;
const SOURCES: Record<string, string> = { catalogue: "Catalogue", sideload: "File", development: "Development" };
const AGENT_LABELS: Record<string, string> = { claude: "Claude Code", codex: "Codex", cursor: "Cursor", other: "other agents" };
const MUTATING = new Set(["set_for", "set_expose", "set_start", "set_tool", "tool_load", "profile_save", "profile_use", "profile_delete", "profile_import", "http_switch", "http_rotate", "connect", "safety_switch", "secret_set", "secret_remove", "hook_set", "telemetry_set", "verify_trust", "settings_restore"]);
const FILES_MAX = 200 * 1024 * 1024;

type Entry = { id: string; title: string; args?: Record<string, string>; reads?: boolean; window?: boolean };
const CATALOGUE: Entry[] = [
  { id: "set_for", title: "Offer an app to an agent", args: { client: "claude, codex, cursor or other", id: "app id", on: "boolean" } },
  { id: "set_expose", title: "Always show an app's tools", args: { id: "app id", on: "boolean" } },
  { id: "set_start", title: "Set when an app's server starts", args: { id: "app id", policy: "demand, session or keep" } },
  { id: "set_tool", title: "Offer one of an app's tools", args: { id: "app id", tool: "tool name", on: "boolean" } },
  { id: "tool_list", title: "List an app's known tools", args: { id: "app id" }, reads: true },
  { id: "tool_load", title: "Start an app to list its tools", args: { id: "app id" }, reads: true },
  { id: "profile_save", title: "Save an agent's settings as a profile", args: { name: "profile name", from: "claude, codex, cursor or other" } },
  { id: "profile_use", title: "Use a profile for an agent", args: { client: "claude, codex, cursor or other", name: "profile name or null" } },
  { id: "profile_delete", title: "Delete a profile", args: { name: "profile name" } },
  { id: "select_agent", title: "Show the apps for an agent", args: { agent: "claude, codex, cursor or other" }, window: true },
  { id: "select_filter", title: "Filter the app list", args: { filter: "all or mcp" }, window: true },
  { id: "refresh", title: "Read this computer's state again", reads: true },
];

let listener: ((command: string) => void) | null = null;
let lastSnapshot = "";

export function watchInvoke(raw: Invoke | undefined): Invoke | undefined {
  if (!raw) return undefined;
  return async (cmd, args) => {
    const result = await raw(cmd, args);
    if (MUTATING.has(cmd)) listener?.(cmd);
    else if (cmd === "snapshot") {
      const text = JSON.stringify(result);
      if (lastSnapshot && text !== lastSnapshot) listener?.("refresh");
      lastSnapshot = text;
    }
    return result;
  };
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls = "", text = ""): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text) node.textContent = text;
  return node;
}

function button(label: string, cls = "btn btn-small"): HTMLButtonElement {
  const b = el("button", cls, label);
  b.type = "button";
  return b;
}

function setText(node: Element | null, text: string): void {
  if (node && node.textContent !== text) node.textContent = text;
}

function base64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let out = "";
  for (let i = 0; i < bytes.length; i += 0x8000) out += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(out);
}

export interface Options {
  invoke: Invoke | undefined;
  refresh: () => Promise<void>;
  family: FamilyApp[];
  appVersion: string;
}

export function mountExtensions(o: Options): void {
  const tauri = "__TAURI_INTERNALS__" in window;
  const status = document.getElementById("status") as HTMLElement;
  const safetyToggle = document.getElementById("safety-toggle") as HTMLButtonElement;
  const profilesToggle = document.getElementById("profiles-toggle") as HTMLButtonElement;
  const profilesBox = document.getElementById("profiles") as HTMLElement;
  const safetyBox = document.getElementById("safety") as HTMLElement;
  const agentPick = document.getElementById("agent") as HTMLSelectElement;
  const chips = Array.from(document.querySelectorAll<HTMLButtonElement>(".chip"));

  const wrap = el("span", "ext-wrap");
  const toggle = button("Extensions");
  toggle.id = "ext-toggle";
  toggle.setAttribute("aria-haspopup", "menu");
  toggle.setAttribute("aria-expanded", "false");
  const menu = el("div", "ext-menu");
  menu.id = "ext-menu";
  menu.setAttribute("role", "menu");
  menu.setAttribute("aria-label", "Extensions");
  menu.hidden = true;
  wrap.append(toggle, menu);
  const undo = button("Undo", "btn btn-small ext-undo");
  undo.id = "ext-undo";
  undo.disabled = true;
  safetyToggle.after(wrap, undo);

  const panel = el("section", "ext-panel");
  panel.id = "ext-panel";
  panel.setAttribute("aria-label", "Extensions");
  panel.hidden = true;
  const panelHead = el("div", "ext-head");
  const note = el("p", "profile-note ext-note", "Extensions add commands and panels to formMCP. FormPortal installs them; form.ext runs one from a folder.");
  const reloadButton = button("Reload");
  reloadButton.title = "Read FormPortal's list and form.ext's development list again";
  const manageButton = button("Manage in FormPortal");
  panelHead.append(note, reloadButton, manageButton);
  const list = el("ul", "profile-list ext-list");
  list.id = "ext-list";
  const empty = el("li", "ext-empty");
  const emptyText = el("span", "profile-what", "No extensions yet. Find them in FormPortal or make one with form.ext.");
  const emptyManage = button("Manage in FormPortal");
  empty.append(emptyText, emptyManage);
  list.append(empty);
  panel.append(panelHead, list);

  const dock = el("section", "ext-dock");
  dock.id = "ext-dock";
  dock.hidden = true;
  const dockHead = el("div", "ext-dock-head");
  const dockTitle = el("span", "ext-dock-title");
  const dockClose = button("Close");
  dockHead.append(dockTitle, dockClose);
  const dockBody = el("div", "ext-dock-body");
  dock.append(dockHead, dockBody);
  safetyBox.after(panel, dock);

  const dialog = el("dialog", "ext-dialog");
  dialog.id = "ext-dialog";
  const dialogTitle = el("p", "ext-dialog-title");
  const dialogBody = el("div", "ext-dialog-body");
  const dialogFoot = el("div", "ext-dialog-foot");
  const dialogClose = button("Close");
  dialogFoot.append(dialogClose);
  dialog.append(dialogTitle, dialogBody, dialogFoot);

  const consentBox = el("dialog", "ext-dialog ext-consent");
  consentBox.id = "ext-consent";
  const consentTitle = el("p", "ext-dialog-title");
  const consentText = el("p", "ext-consent-text");
  const consentList = el("ul", "ext-consent-list");
  const consentFoot = el("div", "ext-dialog-foot");
  const notNow = button("Not Now");
  const allow = button("Allow", "btn btn-small ext-allow");
  consentFoot.append(notNow, allow);
  consentBox.append(consentTitle, consentText, consentList, consentFoot);

  const saveBox = el("dialog", "ext-dialog ext-consent");
  saveBox.id = "ext-save";
  const saveTitle = el("p", "ext-dialog-title");
  const saveText = el("p", "ext-consent-text");
  const saveRow = el("div", "profile-form");
  const saveName = el("input");
  saveName.type = "text";
  saveName.spellcheck = false;
  saveName.setAttribute("aria-label", "File name");
  saveRow.append(saveName);
  const saveFoot = el("div", "ext-dialog-foot");
  const saveCancel = button("Cancel");
  const saveOk = button("Save", "btn btn-small ext-allow");
  saveFoot.append(saveCancel, saveOk);
  saveBox.append(saveTitle, saveText, saveRow, saveFoot);

  const headless = el("div", "ext-headless");
  headless.setAttribute("aria-hidden", "true");
  document.body.append(dialog, consentBox, saveBox, headless);

  let records: ExtensionRecord[] = [];
  let stamp = "";
  let menuKey = "";
  let listNote = "";
  let loadedOnce = false;
  const rows = new Map<string, { li: HTMLLIElement; key: string }>();
  let consentQueue: Promise<void> = Promise.resolve();
  let dockFrame: Frame | null = null;
  let dialogFrame: Frame | null = null;
  let undoMark: { label: string; text: string } | null = null;
  let applying = 0;
  let levelTimer = 0;

  const say = (text: string, level = "info"): void => {
    setText(status, text);
    status.dataset.level = level;
    window.clearTimeout(levelTimer);
    levelTimer = window.setTimeout(() => {
      if (status.textContent === text) delete status.dataset.level;
    }, 6000);
  };

  const tokenMap: [string, string][] = [["--bg", "--ground"], ["--panel", "--pane"], ["--text", "--ink"], ["--muted", "--ink-faint"], ["--accent", "--accent"], ["--border", "--edge-strong"], ["--radius", "--radius-s"]];
  const theme = (): Theme => {
    const style = getComputedStyle(document.documentElement);
    const tokens: Record<string, string> = {};
    for (const [name, mine] of tokenMap) tokens[name] = style.getPropertyValue(mine).trim();
    for (const extra of ["--ink-soft", "--live", "--fault", "--font", "--fs-m"]) tokens[extra] = style.getPropertyValue(extra).trim();
    return { mode: window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark", tokens };
  };

  const selection = () => ({ agent: agentPick.value, filter: chips.find((c) => c.getAttribute("aria-pressed") === "true")?.dataset.filter ?? "all" });

  const snapshotDoc = async (what: string): Promise<unknown> => {
    const sel = selection();
    const head = { app: PRODUCT, version: o.appVersion, live: !!o.invoke };
    if (!o.invoke) {
      const apps = o.family.map((a) => ({ id: a.id, name: a.name, kind: a.kind, summary: a.summary, mcp: a.mcp, state: a.state }));
      const all: Record<string, unknown> = { selection: sel, apps, agents: [], profiles: [], http: null, source: "bundled list" };
      return what === "all" ? { ...head, ...all } : { ...head, [what]: all[what] };
    }
    const s = (await o.invoke("snapshot")) as {
      source: string;
      apps: { id: string; installed: boolean; version: string; reach: string | null; exposed: boolean; off_for: string[]; profile_off_for: string[]; start: string; hidden_tools: number }[];
      clients: { client: string; state: string }[];
      profiles: { list: unknown[]; in_use: Record<string, string> };
      http: unknown;
    };
    const live = new Map(s.apps.map((a) => [a.id, a]));
    const apps = o.family.map((a) => {
      const l = live.get(a.id);
      const offFor = l?.off_for ?? [];
      const profileOff = l?.profile_off_for ?? [];
      return { id: a.id, name: a.name, kind: a.kind, summary: a.summary, mcp: a.mcp, state: a.state, installed: l?.installed ?? false, version: l?.version ?? "", reach: l?.reach ?? null, exposed: l?.exposed ?? false, on_for_agent: !offFor.includes(sel.agent) && !profileOff.includes(sel.agent), off_for: offFor, profile_off_for: profileOff, start: l?.start ?? "demand", hidden_tools: l?.hidden_tools ?? 0 };
    });
    const states = new Map(s.clients.map((c) => [c.client, c.state]));
    const agents = Object.keys(AGENT_LABELS).map((c) => ({ client: c, label: AGENT_LABELS[c], state: states.get(c) ?? null, profile: s.profiles.in_use[c] ?? null }));
    const all: Record<string, unknown> = { selection: sel, apps, agents, profiles: s.profiles.list, http: s.http, source: s.source };
    return what === "all" ? { ...head, ...all } : { ...head, [what]: all[what] };
  };

  const select = (agent?: string, filter?: string): { agent: string; filter: string } => {
    if (agent !== undefined) {
      if (!(agent in AGENT_LABELS)) throw new HostError(ERR.INVALID, `agent is claude, codex, cursor or other, not ${JSON.stringify(agent)}`);
      if (agentPick.value !== agent && !agentPick.disabled) {
        agentPick.value = agent;
        agentPick.dispatchEvent(new Event("change"));
      }
    }
    if (filter !== undefined) {
      const chip = chips.find((c) => c.dataset.filter === filter);
      if (!chip) throw new HostError(ERR.INVALID, `filter is all or mcp, not ${JSON.stringify(filter)}`);
      if (chip.getAttribute("aria-pressed") !== "true") chip.click();
    }
    return selection();
  };

  const runOne = async (op: Op): Promise<unknown> => {
    const entry = CATALOGUE.find((c) => c.id === op.command);
    if (!entry) throw new HostError(ERR.REFUSED, `formMCP has no command ${op.command} for extensions`);
    const args = (op.args && typeof op.args === "object" ? op.args : {}) as Record<string, unknown>;
    if (op.command === "select_agent") return select(String(args.agent ?? ""));
    if (op.command === "select_filter") return select(undefined, String(args.filter ?? ""));
    if (op.command === "refresh") return snapshotDoc("all");
    if (!o.invoke) throw new HostError(ERR.UNAVAILABLE, "this preview has no formMCP settings to change; open the formMCP app");
    try {
      return await o.invoke(op.command, args);
    } catch (e) {
      throw new HostError(ERR.REFUSED, String(e));
    }
  };

  const applyDocument = async (ext: ExtensionRecord, label: string, ops: Op[]): Promise<{ applied: number; results: unknown[] }> => {
    for (const op of ops) if (!CATALOGUE.some((c) => c.id === op.command)) throw new HostError(ERR.REFUSED, `formMCP has no command ${op.command} for extensions`);
    const changes = ops.some((op) => !CATALOGUE.find((c) => c.id === op.command)?.reads && !CATALOGUE.find((c) => c.id === op.command)?.window);
    if (changes && !o.invoke) throw new HostError(ERR.UNAVAILABLE, "this preview has no formMCP settings to change; open the formMCP app");
    const mark = changes && o.invoke ? String(await o.invoke("settings_mark")) : "";
    const results: unknown[] = [];
    applying += 1;
    try {
      for (const op of ops) results.push(await runOne(op));
    } catch (e) {
      if (mark && o.invoke) {
        await o.invoke("settings_restore", { text: mark }).catch(() => undefined);
        await o.refresh();
      }
      throw e;
    } finally {
      applying -= 1;
    }
    if (changes) {
      undoMark = { label: `${ext.name}: ${label}`, text: mark };
      undo.disabled = false;
      undo.title = `Undo ${undoMark.label}`;
      undo.classList.add("is-ready");
      await o.refresh();
      say(`${ext.name}: ${label}. Undo puts the settings back.`);
    }
    return { applied: results.length, results };
  };

  const openFiles = (accept: string[], multiple: boolean): Promise<{ name: string; type: string; size: number; data: string }[]> =>
    new Promise((resolve, reject) => {
      const input = el("input");
      input.type = "file";
      input.accept = accept.join(",");
      input.multiple = multiple;
      input.hidden = true;
      let done = false;
      const finish = (fn: () => void) => {
        if (done) return;
        done = true;
        input.remove();
        fn();
      };
      input.addEventListener("cancel", () => finish(() => reject(new HostError(ERR.CANCELLED, "no file was chosen"))));
      input.addEventListener("change", async () => {
        const files = Array.from(input.files ?? []);
        if (!files.length) return finish(() => reject(new HostError(ERR.CANCELLED, "no file was chosen")));
        const total = files.reduce((n, f) => n + f.size, 0);
        if (total > FILES_MAX) return finish(() => reject(new HostError(ERR.REFUSED, "the chosen files are larger than 200 MB together")));
        const out = await Promise.all(files.map(async (f) => ({ name: f.name, type: f.type, size: f.size, data: base64(await f.arrayBuffer()) })));
        finish(() => resolve(out));
      });
      document.body.append(input);
      input.click();
    });

  const confirmName = (ext: ExtensionRecord, suggested: string): Promise<string> =>
    new Promise((resolve, reject) => {
      setText(saveTitle, `${ext.name} wants to save a file`);
      setText(saveText, "formMCP saves it in your Downloads folder, with a number added when the name is taken. Name:");
      saveName.value = suggested;
      const end = (name: string | null) => {
        saveOk.removeEventListener("click", yes);
        saveCancel.removeEventListener("click", no);
        saveBox.removeEventListener("cancel", no);
        saveName.removeEventListener("keydown", enter);
        if (saveBox.open) saveBox.close();
        if (name) resolve(name);
        else reject(new HostError(ERR.CANCELLED, "the file was not saved"));
      };
      const yes = () => end(saveName.value.trim() || suggested);
      const no = (e?: Event) => {
        e?.preventDefault();
        end(null);
      };
      const enter = (e: KeyboardEvent) => {
        if (e.key === "Enter") yes();
      };
      saveOk.addEventListener("click", yes);
      saveCancel.addEventListener("click", no);
      saveBox.addEventListener("cancel", no);
      saveName.addEventListener("keydown", enter);
      saveBox.showModal();
      saveName.focus();
      saveName.select();
    });

  const saveFile = async (ext: ExtensionRecord, suggested: string, type: string, data: string): Promise<{ saved: boolean; name: string }> => {
    const name = await confirmName(ext, suggested);
    if (tauri && o.invoke) {
      const path = String(await o.invoke("extension_save", { name, data }));
      say(`${ext.name} saved ${path}`, "success");
      return { saved: true, name: path.split(/[\\/]/).pop() ?? name };
    }
    const bytes = Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
    const url = URL.createObjectURL(new Blob([bytes], { type }));
    const a = el("a");
    a.href = url;
    a.download = name;
    document.body.append(a);
    a.click();
    a.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 2000);
    say(`${ext.name} saved ${name} to your downloads`, "success");
    return { saved: true, name };
  };

  const consent = (ext: ExtensionRecord): Promise<boolean> =>
    new Promise((resolve) => {
      setText(consentTitle, `Allow ${ext.name} ${ext.version}?`);
      setText(consentText, `${ext.name} (${SOURCES[ext.source] ?? ext.source}) asks to:`);
      consentList.replaceChildren(...ext.permissions.map((p) => el("li", "", PERMISSION_WORDS[p as Permission] ?? p)));
      const end = (ok: boolean) => {
        allow.removeEventListener("click", yes);
        notNow.removeEventListener("click", no);
        consentBox.removeEventListener("cancel", no);
        if (consentBox.open) consentBox.close();
        resolve(ok);
      };
      const yes = () => end(true);
      const no = (e?: Event) => {
        e?.preventDefault();
        end(false);
      };
      allow.addEventListener("click", yes);
      notNow.addEventListener("click", no);
      consentBox.addEventListener("cancel", no);
      consentBox.showModal();
      allow.focus();
    });

  const host = new ExtensionHost({
    product: PRODUCT,
    app: APP,
    appVersion: o.appVersion,
    theme,
    notify: (ext, message, level) => say(`${ext.name}: ${message}`, level),
    setTitle: (ext, frame, title) => {
      const text = `${ext.name}: ${title}`;
      if (dockFrame?.frame === frame) setText(dockTitle, text);
      if (dialogFrame?.frame === frame) setText(dialogTitle, text);
    },
    readDocument: (_ext, params) => {
      const what = params.what ?? "all";
      if (!["all", "apps", "agents", "profiles", "http", "selection"].includes(what)) throw new HostError(ERR.INVALID, "what is all, apps, agents, profiles, http or selection");
      return snapshotDoc(what);
    },
    applyDocument,
    commands: () => CATALOGUE.map((c) => ({ id: c.id, title: c.title, ...(c.args ? { args: c.args } : {}) })),
    run: async (ext, command, args) => {
      const entry = CATALOGUE.find((c) => c.id === command);
      if (!entry) throw new HostError(ERR.REFUSED, `formMCP has no command ${command} for extensions`);
      if (entry.reads || entry.window) return runOne({ command, args });
      return (await applyDocument(ext, entry.title, [{ command, args }])).results[0];
    },
    openFiles: (_ext, accept, multiple) => openFiles(accept, multiple),
    saveFile,
    consent: (ext) => {
      const answer = consentQueue.then(() => (host.isGranted(ext) ? true : consent(ext)));
      consentQueue = answer.then(() => undefined);
      return answer;
    },
  });

  const stateOf = (r: ExtensionRecord): string => (r.state === "refused" ? "refused" : host.isGranted(r) ? "ready" : "needs-permission");
  const stateText = (r: ExtensionRecord): string => (r.state === "refused" ? `Refused: ${r.reason ?? ""}` : host.isGranted(r) ? "Ready" : "Needs Permission");

  const closeMenu = (): void => {
    menu.hidden = true;
    toggle.setAttribute("aria-expanded", "false");
  };

  const closePanel = (): void => {
    panel.hidden = true;
  };

  const closeDock = (): void => {
    dockFrame?.dispose();
    dockFrame = null;
    dock.hidden = true;
  };

  const closeDialog = (): void => {
    dialogFrame?.dispose();
    dialogFrame = null;
    if (dialog.open) dialog.close();
  };

  const openPanel = async (r: ExtensionRecord, panelId: string, activation?: { reason: "command"; command: string; args?: unknown }): Promise<void> => {
    const def = r.panels.find((p) => p.id === panelId);
    if (!def || r.state !== "ready") throw new HostError(ERR.REFUSED, `${r.name} has no panel ${panelId}`);
    const title = `${r.name}: ${def.title}`;
    const start = activation ? { ...activation, panel: panelId } : { reason: "panel" as const, panel: panelId };
    if (def.location === "dialog") {
      closeDialog();
      dialogBody.style.height = "";
      const frame = await host.open(r, dialogBody, { onResize: (h) => (dialogBody.style.height = `${Math.max(120, Math.min(h, 560))}px`) });
      updateRows();
      if (!frame) throw new HostError(ERR.PERMISSION, `${r.name} was not allowed to open`);
      dialogFrame = frame;
      setText(dialogTitle, title);
      dialog.showModal();
      await frame.activate(start).catch(() => undefined);
      return;
    }
    closeDock();
    const frame = await host.open(r, dockBody);
    updateRows();
    if (!frame) throw new HostError(ERR.PERMISSION, `${r.name} was not allowed to open`);
    dockFrame = frame;
    setText(dockTitle, title);
    dock.hidden = false;
    dock.dataset.ext = r.id;
    await frame.activate(start).catch(() => undefined);
  };

  const runCommand = async (r: ExtensionRecord, commandId: string, args: unknown = {}): Promise<unknown> => {
    const command = r.commands.find((c) => c.id === commandId);
    if (!command) throw new HostError(ERR.NOT_FOUND, `${r.name} has no command ${commandId}`);
    if (r.state !== "ready") throw new HostError(ERR.REFUSED, `${r.name} is refused: ${r.reason ?? ""}`);
    if (command.panel) {
      await openPanel(r, command.panel, { reason: "command", command: commandId, args });
      return null;
    }
    return host.runCommand(r, commandId, args, headless).finally(updateRows);
  };

  const menuItem = (label: string, run: () => void, cls = "ext-item"): HTMLButtonElement => {
    const item = button(label, cls);
    item.setAttribute("role", "menuitem");
    item.addEventListener("click", () => {
      closeMenu();
      run();
    });
    return item;
  };

  const portal = (): void => {
    if (tauri && o.invoke) void o.invoke("open_portal").then(() => say("Opening FormPortal's extensions for formMCP."), (e) => say(String(e), "error"));
    else {
      window.location.href = PORTAL;
      say("Asked the system to open FormPortal's extensions for formMCP.");
    }
  };

  const buildMenu = (): void => {
    const items: HTMLElement[] = [];
    for (const r of records) {
      if (r.state !== "ready") continue;
      for (const c of r.commands) {
        items.push(
          menuItem(`${r.name}: ${c.title}`, () =>
            void runCommand(r, c.id).then(
              () => {
                if (!c.panel) say(`${r.name}: ${c.title} finished.`, "success");
              },
              (e) => say(`${r.name}: ${c.title} failed: ${(e as Error).message ?? e}`, "error"),
            ),
          ),
        );
      }
    }
    if (items.length) items.push(el("hr", "ext-sep"));
    items.push(menuItem("Extensions Panel", () => showPanel()), menuItem("Reload Extensions", () => void reload(true)), menuItem("Manage in FormPortal", portal));
    menu.replaceChildren(...items);
  };

  const fillRow = (li: HTMLLIElement, r: ExtensionRecord): void => {
    li.dataset.state = stateOf(r);
    const head = el("div", "ext-row-head");
    const name = el("span", "profile-name ext-name", r.name);
    name.translate = false;
    head.append(name, el("span", "ext-meta", `${r.version} · ${SOURCES[r.source] ?? r.source}`));
    const chipsBox = el("span", "ext-chips");
    for (const p of r.permissions) {
      const chip = el("span", "ext-chip", p);
      chip.title = PERMISSION_WORDS[p as Permission] ?? p;
      chipsBox.append(chip);
    }
    head.append(chipsBox, el("span", `ext-state ext-state-${stateOf(r)}`, stateText(r)));
    const actions = el("div", "ext-actions");
    if (r.state === "ready") {
      for (const p of r.panels) {
        const open = button(r.panels.length > 1 ? `Open ${p.title}` : "Open");
        open.addEventListener("click", () => void openPanel(r, p.id).catch((e) => say(String((e as Error).message ?? e), "warning")));
        actions.append(open);
      }
    }
    if (r.path) {
      const path = el("span", "ext-path", r.path);
      path.title = r.path;
      actions.append(path);
    }
    if (r.description) li.title = r.description;
    li.replaceChildren(head, actions);
  };

  const updateRows = (): void => {
    const seen = new Set<string>();
    for (const r of records) {
      seen.add(r.id);
      const key = `${JSON.stringify(r)}|${stateOf(r)}`;
      let row = rows.get(r.id);
      if (!row) {
        row = { li: el("li", "profile-row ext-row"), key: "" };
        row.li.dataset.ext = r.id;
        rows.set(r.id, row);
        list.insertBefore(row.li, empty.parentElement === list ? empty : null);
      }
      if (row.key !== key) {
        fillRow(row.li, r);
        row.key = key;
      }
    }
    for (const [id, row] of rows) {
      if (seen.has(id)) continue;
      row.li.remove();
      rows.delete(id);
    }
    if (empty.parentElement !== list) list.append(empty);
    empty.hidden = records.length > 0;
    if (!records.length && listNote) setText(emptyText, listNote);
    else setText(emptyText, "No extensions yet. Find them in FormPortal or make one with form.ext.");
  };

  const fetchJson = async (path: string, init?: RequestInit): Promise<unknown> => {
    const reply = await fetch(path, { cache: "no-store", ...init, headers: { "X-FormMCP": "1", ...(init?.headers ?? {}) } });
    if (!reply.ok) throw new Error(`${reply.status}`);
    const type = reply.headers.get("content-type") ?? "";
    return type.includes("json") ? reply.json() : null;
  };

  const readListing = async (): Promise<Listing> => {
    if (tauri && o.invoke) return (await o.invoke("extensions_list")) as Listing;
    return (await fetchJson("/api/extensions")) as Listing;
  };

  const readStamp = async (): Promise<string> => JSON.stringify(tauri && o.invoke ? await o.invoke("extensions_stamp") : await fetchJson("/api/extensions/stamp"));

  const apply = (next: ExtensionRecord[]): void => {
    const old = new Map(records.map((r) => [r.id, r]));
    for (const r of next) {
      const was = old.get(r.id);
      if (was && (was.version !== r.version || was.base !== r.base || was.state !== r.state || was.entryUi !== r.entryUi)) {
        if (dock.dataset.ext === r.id) closeDock();
        if (dialogFrame?.ext.id === r.id) closeDialog();
        host.disposeExtension(r.id);
      }
      old.delete(r.id);
    }
    for (const id of old.keys()) {
      if (dock.dataset.ext === id) closeDock();
      if (dialogFrame?.ext.id === id) closeDialog();
      host.disposeExtension(id);
    }
    records = next;
    updateRows();
    const key = JSON.stringify(records.filter((r) => r.state === "ready").map((r) => [r.id, r.name, r.commands]));
    if (key !== menuKey) {
      menuKey = key;
      buildMenu();
    }
  };

  const reload = async (asked = false): Promise<void> => {
    try {
      const listing = await readListing();
      stamp = JSON.stringify(listing.stamp);
      listNote = "";
      loadedOnce = true;
      apply(listing.extensions ?? []);
      if (asked) say(`Extensions read again: ${records.length} found${listing.root ? ` in ${listing.root}` : ""}.`);
    } catch {
      if (asked) say("Could not read the extensions again; the list shows what was read before.", "warning");
      if (loadedOnce) return;
      listNote = "No extensions here: this preview cannot read FormPortal's folder. Open the formMCP app, or run the window with ./formmcp.";
      apply([]);
    }
  };

  const poll = async (): Promise<void> => {
    if (!document.hasFocus()) return;
    try {
      const now = await readStamp();
      if (now !== stamp) await reload();
    } catch {
      return;
    }
  };

  let serving = false;
  const serveAgents = async (): Promise<void> => {
    if (serving) return;
    serving = true;
    try {
      const asked = (tauri && o.invoke ? await o.invoke("extension_requests") : await fetchJson("/api/extensions/requests")) as Request[] | null;
      for (const req of asked ?? []) {
        let outcome: { result?: unknown; error?: { code: number; message: string } };
        try {
          let r = records.find((x) => x.id === req.id);
          if (!r) {
            await reload();
            r = records.find((x) => x.id === req.id);
          }
          if (!r) throw new HostError(ERR.NOT_FOUND, `there is no extension ${req.id}`);
          say(`${AGENT_LABELS[req.client ?? ""] ?? req.client ?? "An agent"} runs ${r.name}: ${r.commands.find((c) => c.id === req.command)?.title ?? req.command}`);
          outcome = { result: (await runCommand(r, req.command, req.args ?? {})) ?? null };
        } catch (e) {
          const err = e as { code?: number; message?: string };
          outcome = { error: { code: typeof err.code === "number" ? err.code : ERR.REFUSED, message: String(err.message ?? e) } };
        }
        if (tauri && o.invoke) await o.invoke("extension_finish", { rid: req.rid, outcome }).catch(() => undefined);
        else await fetchJson(`/api/extensions/requests/${encodeURIComponent(req.rid)}`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(outcome) }).catch(() => undefined);
      }
    } catch {
      return;
    } finally {
      serving = false;
    }
  };

  const showPanel = (): void => {
    profilesBox.hidden = true;
    profilesToggle.setAttribute("aria-expanded", "false");
    safetyBox.hidden = true;
    safetyToggle.setAttribute("aria-expanded", "false");
    panel.hidden = false;
  };

  toggle.addEventListener("click", () => {
    const open = menu.hidden;
    menu.hidden = !open;
    toggle.setAttribute("aria-expanded", String(open));
    if (open) (menu.querySelector("button") as HTMLButtonElement | null)?.focus();
  });
  menu.addEventListener("keydown", (e) => {
    const items = Array.from(menu.querySelectorAll<HTMLButtonElement>("button"));
    const at = items.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === "Escape") {
      closeMenu();
      toggle.focus();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      items[(at + (e.key === "ArrowDown" ? 1 : items.length - 1)) % items.length]?.focus();
    }
  });
  document.addEventListener("click", (e) => {
    if (!menu.hidden && !wrap.contains(e.target as Node)) closeMenu();
  });
  for (const other of [profilesToggle, safetyToggle]) other.addEventListener("click", closePanel);
  reloadButton.addEventListener("click", () => void reload(true));
  manageButton.addEventListener("click", portal);
  emptyManage.addEventListener("click", portal);
  dockClose.addEventListener("click", closeDock);
  dialogClose.addEventListener("click", closeDialog);
  dialog.addEventListener("cancel", (e) => {
    e.preventDefault();
    closeDialog();
  });
  undo.addEventListener("click", async () => {
    if (!undoMark || !o.invoke) return;
    const mark = undoMark;
    undoMark = null;
    undo.disabled = true;
    undo.classList.remove("is-ready");
    try {
      applying += 1;
      await o.invoke("settings_restore", { text: mark.text });
      await o.refresh();
      say(`Undid ${mark.label}.`);
    } catch (e) {
      say(`Could not undo ${mark.label}: ${String(e)}`, "error");
    } finally {
      applying -= 1;
    }
  });
  agentPick.addEventListener("change", () => host.broadcast("selection.changed", selection()));
  for (const chip of chips) chip.addEventListener("click", () => host.broadcast("selection.changed", selection()));
  window.matchMedia("(prefers-color-scheme: light)").addEventListener("change", () => host.setTheme(theme()));

  let pendingChange = "";
  let changeTimer = 0;
  listener = (command) => {
    if (!applying && undoMark && command !== "refresh") {
      undoMark = null;
      undo.disabled = true;
      undo.classList.remove("is-ready");
    }
    pendingChange = command;
    window.clearTimeout(changeTimer);
    changeTimer = window.setTimeout(() => host.broadcast("document.changed", { command: pendingChange }), 50);
  };

  buildMenu();
  void reload();
  window.setInterval(() => void poll(), 2000);
  window.setInterval(() => void serveAgents(), 1000);
  window.addEventListener("focus", () => void poll());
}
