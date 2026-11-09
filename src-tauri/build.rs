const COMMANDS: &[&str] = &[
    "app_ready", "snapshot", "set_expose", "set_for", "tool_list", "tool_load", "set_tool", "http_switch", "http_token", "http_rotate", "connect", "log_tail",
    "set_start", "profile_use", "profile_save", "profile_delete", "profile_export", "profile_import", "safety", "secret_set", "secret_remove", "safety_switch",
    "hook_set", "telemetry_set", "verify_trust", "extensions_list", "extensions_stamp", "extension_requests", "extension_finish", "extension_save", "open_portal",
    "settings_mark", "settings_restore", "open_demo",
];

fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS))).expect("the Tauri build step failed");
}
