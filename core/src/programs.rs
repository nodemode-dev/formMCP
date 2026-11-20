use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::Duration;

pub const SHELL_LIMIT: Duration = Duration::from_secs(3);
pub const NO_SHELL: &str = "FORMMCP_NO_SHELL_PATH";
const OPEN: &str = "__FORMMCP_PATH_OPEN__";
const CLOSE: &str = "__FORMMCP_PATH_CLOSE__";
const SHELLS: &[&str] = &["zsh", "bash", "fish", "ksh", "sh", "dash"];

const HOME_DIRS: &[&str] = &[
    ".local/bin",
    ".claude/local",
    ".claude/bin",
    ".npm-global/bin",
    ".npm-packages/bin",
    ".volta/bin",
    ".asdf/shims",
    ".local/share/mise/shims",
    ".nodenv/shims",
    ".bun/bin",
    ".yarn/bin",
    ".cargo/bin",
    ".nix-profile/bin",
    "bin",
];
const MACOS_HOME_DIRS: &[&str] = &["Library/pnpm", "Library/Application Support/fnm/aliases/default/bin"];
const LINUX_HOME_DIRS: &[&str] = &[".local/share/pnpm", ".local/share/fnm/aliases/default/bin", ".linuxbrew/bin"];
const MACOS_DIRS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"];
const LINUX_DIRS: &[&str] = &["/usr/local/bin", "/home/linuxbrew/.linuxbrew/bin", "/snap/bin"];
const WINDOWS_HOME_DIRS: &[&str] = &[".local\\bin", ".claude\\local", ".claude\\bin", ".bun\\bin", ".volta\\bin", ".cargo\\bin", "scoop\\shims"];
const WINDOWS_VAR_DIRS: &[(&str, &str)] = &[
    ("APPDATA", "npm"),
    ("LOCALAPPDATA", "pnpm"),
    ("LOCALAPPDATA", "Volta\\bin"),
    ("LOCALAPPDATA", "Microsoft\\WinGet\\Links"),
    ("LOCALAPPDATA", "Programs\\claude"),
    ("ProgramFiles", "nodejs"),
    ("NVM_SYMLINK", ""),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    Macos,
    Windows,
}

impl Os {
    pub fn current() -> Os {
        if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        }
    }
}

pub fn override_var(program: &str) -> String {
    format!("FORMMCP_{}_BIN", program.to_uppercase().replace('-', "_"))
}

fn version_key(name: &str) -> Vec<u64> {
    name.trim_start_matches('v').split('.').map(|part| part.parse().unwrap_or(0)).collect()
}

fn nvm_dirs(home: &Path) -> Vec<PathBuf> {
    let base = home.join(".nvm").join("versions").join("node");
    let Ok(entries) = std::fs::read_dir(&base) else { return Vec::new() };
    let mut versions: Vec<String> = entries.flatten().filter_map(|e| e.file_name().to_str().map(str::to_string)).collect();
    versions.sort_by(|a, b| version_key(b).cmp(&version_key(a)).then_with(|| a.cmp(b)));
    versions.into_iter().map(|v| base.join(v).join("bin")).collect()
}

pub fn known_dirs(os: Os, home: &Path, var: &dyn Fn(&str) -> Option<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if os == Os::Windows {
        out.extend(WINDOWS_HOME_DIRS.iter().map(|d| home.join(d)));
        for (name, rest) in WINDOWS_VAR_DIRS {
            if let Some(base) = var(name) {
                out.push(if rest.is_empty() { base } else { base.join(rest) });
            }
        }
        return out;
    }
    if let Some(prefix) = var("NPM_CONFIG_PREFIX") {
        out.push(prefix.join("bin"));
    }
    out.extend(HOME_DIRS.iter().map(|d| home.join(d)));
    out.extend(nvm_dirs(home));
    let (own, shared) = if os == Os::Macos { (MACOS_HOME_DIRS, MACOS_DIRS) } else { (LINUX_HOME_DIRS, LINUX_DIRS) };
    out.extend(own.iter().map(|d| home.join(d)));
    out.extend(shared.iter().map(PathBuf::from));
    out
}

pub fn shell_dirs(shell: &Path, limit: Duration) -> Option<Vec<PathBuf>> {
    let name = shell.file_name()?.to_str()?;
    if !SHELLS.contains(&name) {
        return None;
    }
    let script = if name == "fish" { format!("printf '%s%s%s' {OPEN} (string join : $PATH) {CLOSE}") } else { format!("printf '%s%s%s' {OPEN} \"$PATH\" {CLOSE}") };
    let mut child = Command::new(shell).args(["-l", "-i", "-c", &script]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    let mut out = child.stdout.take()?;
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut seen = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            match out.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    seen.extend_from_slice(&chunk[..n]);
                    if String::from_utf8_lossy(&seen).contains(CLOSE) {
                        break;
                    }
                }
            }
        }
        let _ = send.send(seen);
    });
    let got = receive.recv_timeout(limit);
    let _ = child.kill();
    let _ = child.wait();
    let text = String::from_utf8_lossy(&got.ok()?).to_string();
    let inner = text.split(OPEN).nth(1)?.split(CLOSE).next()?.to_string();
    let dirs: Vec<PathBuf> = std::env::split_paths(&inner).filter(|d| d.is_absolute()).collect();
    (!dirs.is_empty()).then_some(dirs)
}

fn session_shell() -> &'static [PathBuf] {
    static ASKED: OnceLock<Vec<PathBuf>> = OnceLock::new();
    ASKED.get_or_init(|| {
        if cfg!(windows) || std::env::var_os(NO_SHELL).is_some() {
            return Vec::new();
        }
        let fallback = if cfg!(target_os = "macos") { "/bin/zsh" } else { "/bin/bash" };
        let shell = std::env::var_os("SHELL").map(PathBuf::from).filter(|s| s.is_file()).unwrap_or_else(|| PathBuf::from(fallback));
        shell_dirs(&shell, SHELL_LIMIT).unwrap_or_default()
    })
}

pub fn order(first: Vec<PathBuf>, second: Vec<PathBuf>, third: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for dir in first.into_iter().chain(second).chain(third) {
        if !dir.as_os_str().is_empty() && !out.contains(&dir) {
            out.push(dir);
        }
    }
    out
}

fn home() -> PathBuf {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn process_dirs() -> Vec<PathBuf> {
    std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default()
}

fn known() -> Vec<PathBuf> {
    known_dirs(Os::current(), &home(), &|name| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from))
}

pub fn runnable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata().map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0).unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

pub fn find_in(dirs: &[PathBuf], program: &str) -> Option<PathBuf> {
    let names = if cfg!(windows) { vec![format!("{program}.exe"), format!("{program}.cmd"), format!("{program}.bat")] } else { vec![program.to_string()] };
    dirs.iter().flat_map(|dir| names.iter().map(move |name| dir.join(name))).find(|candidate| runnable(candidate))
}

pub fn find_with(program: &str, chosen: Option<PathBuf>, process: &[PathBuf], known: &[PathBuf], shell: &dyn Fn() -> Vec<PathBuf>) -> Option<PathBuf> {
    chosen.filter(|path| runnable(path)).or_else(|| find_in(process, program)).or_else(|| find_in(known, program)).or_else(|| find_in(&shell(), program))
}

pub fn find(program: &str) -> Option<PathBuf> {
    let chosen = std::env::var_os(override_var(program)).filter(|v| !v.is_empty()).map(PathBuf::from);
    find_with(program, chosen, &process_dirs(), &known(), &|| session_shell().to_vec())
}

pub fn child_path(exe: &Path, shell: &[PathBuf], known: Vec<PathBuf>, process: Vec<PathBuf>) -> OsString {
    let mut first: Vec<PathBuf> = Vec::new();
    if let Some(dir) = exe.parent() {
        first.push(dir.to_path_buf());
    }
    if let Some(dir) = exe.canonicalize().ok().and_then(|real| real.parent().map(Path::to_path_buf)) {
        first.push(dir);
    }
    first.extend(shell.iter().cloned());
    let all = order(first, known.into_iter().filter(|d| d.is_dir()).collect(), process);
    std::env::join_paths(all.iter().filter(|d| !d.to_string_lossy().contains(if cfg!(windows) { ';' } else { ':' }))).unwrap_or_default()
}

pub fn command(exe: &Path) -> Command {
    let mut command = Command::new(exe);
    command.env("PATH", child_path(exe, session_shell(), known(), process_dirs()));
    command
}
