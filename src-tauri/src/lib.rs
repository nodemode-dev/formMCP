use formmcp_core::mcp;

pub fn run() {
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        let code = match mcp::serve() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("formmcp: {e}");
                1
            }
        };
        std::process::exit(code);
    }
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("formMCP could not start its window");
}
