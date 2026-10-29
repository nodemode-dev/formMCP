use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use formmcp_core::settings::{self, Profile, Settings};

fn big_settings(n: usize) -> Settings {
    let mut s = Settings::default();
    for i in 0..400 {
        s.profiles.insert(format!("profile-{n}-{i}"), Profile { apps_off: vec![format!("app-{i}"); 8], ..Profile::default() });
    }
    s
}

#[test]
fn a_reader_never_sees_a_half_written_settings_file_and_no_part_file_is_left() {
    let home = std::env::temp_dir().join(format!("formmcp-settings-save-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::env::set_var("FORMMCP_HOME", &home);
    big_settings(0).save().unwrap();

    let done = Arc::new(AtomicBool::new(false));
    let writers: Vec<_> = (0..4)
        .map(|w| {
            let done = Arc::clone(&done);
            thread::spawn(move || {
                let mut n = 0;
                while !done.load(Ordering::SeqCst) {
                    big_settings(w * 1000 + n % 7).save().unwrap();
                    n += 1;
                }
            })
        })
        .collect();

    let mut torn = 0;
    for _ in 0..400 {
        let text = std::fs::read_to_string(settings::path()).unwrap_or_default();
        if serde_json::from_str::<Settings>(&text).is_err() {
            torn += 1;
        }
    }
    done.store(true, Ordering::SeqCst);
    for w in writers {
        w.join().unwrap();
    }

    assert_eq!(torn, 0, "settings.json was read half written {torn} times out of 400");
    assert_eq!(Settings::saved().profiles.len(), 400);
    let left: Vec<_> = std::fs::read_dir(&home).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n != "settings.json").collect();
    assert!(left.is_empty(), "files left beside settings.json: {left:?}");
    let _ = std::fs::remove_dir_all(&home);
}
