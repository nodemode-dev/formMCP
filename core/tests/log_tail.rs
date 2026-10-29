use formmcp_core::log;

const TAIL_MAX: usize = 64 * 1024;

fn lines(extra: usize) -> String {
    let mut text = String::new();
    for i in 0..400 {
        text.push_str(&format!("{{\"t\":{i},\"event\":\"call\",\"note\":\"{}\"}}\n", "\u{e9}".repeat(120)));
    }
    text.push_str(&format!("{{\"t\":400,\"event\":\"last\",\"note\":\"{}\"}}\n", "a".repeat(extra)));
    text
}

#[test]
fn the_tail_of_a_long_log_is_read_when_its_cut_lands_inside_a_character() {
    let home = std::env::temp_dir().join(format!("formmcp-log-tail-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();
    std::env::set_var("FORMMCP_HOME", &home);
    let text = (0..4).map(lines).find(|t| !t.is_char_boundary(t.len() - TAIL_MAX)).expect("a cut inside a character");
    std::fs::write(log::path(), &text).unwrap();

    let (events, next) = log::tail(0);

    assert_eq!(next, text.len() as u64);
    assert!(events.len() > 100, "only {} events came back", events.len());
    assert_eq!(events.last().and_then(|e| e["event"].as_str()), Some("last"));
    let (again, still) = log::tail(next);
    assert!(again.is_empty());
    assert_eq!(still, next);
    let _ = std::fs::remove_dir_all(&home);
}
