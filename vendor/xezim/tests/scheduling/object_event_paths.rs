use std::process::Command;

/// IEEE 1800 §9.4.2, §13.3 and §15.5: event identity and suspension
/// survive instance paths, object properties and local collection receivers.
#[test]
fn object_event_paths() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/scheduling/object_event_paths.sv"
    );
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "--no-cache", "-s", "top", source])
        .output()
        .expect("run simulator");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{text}");
    assert!(text.contains("T|hier=3,3 prop=1,1 blocked=0"), "{text}");
    assert!(text.contains("T|selected=1 qualified=3"), "{text}");
    assert!(text.contains("T|local=3,3 deferred=1"), "{text}");
}
