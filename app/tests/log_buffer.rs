use phanerite::state::{LiveLogLine, LiveLogStore, LogSource};

#[test]
fn live_logs_are_bounded_and_keep_source() {
    let mut store = LiveLogStore::default();
    for i in 0..=LiveLogStore::MAX_LINES {
        store.append(
            "session",
            LiveLogLine {
                source: if i % 2 == 0 {
                    LogSource::Stdout
                } else {
                    LogSource::Stderr
                },
                text: i.to_string(),
            },
        );
    }
    let lines = store.lines("session").unwrap();
    assert_eq!(lines.len(), LiveLogStore::MAX_LINES);
    assert_eq!(lines.front().unwrap().text, "1");
    assert_eq!(lines.back().unwrap().source, LogSource::Stdout);
}
