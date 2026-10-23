use std::path::PathBuf;
use std::process::Command;

#[test]
fn packed_record_member_loop_stays_compiled_and_matches() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/perf/packed_record_edge.sv");
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "record_edge_check",
            source.to_str().unwrap(),
            "--no-cache",
        ])
        .env("XEZIM_PROFILE_TIMING", "1")
        .output()
        .expect("run packed record edge workload");

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "packed record workload failed:\n{text}"
    );
    assert!(
        text.contains("PACKED_RECORD_OK"),
        "packed record values did not match:\n{text}"
    );
    assert!(
        text.lines().any(|line| line.contains("fallbacks=0")),
        "packed record edge loop used an interpreter fallback:\n{text}"
    );
    assert!(
        text.contains("2 edge blocks, 2 gateable"),
        "wide packed reads were excluded from edge gating:\n{text}"
    );
}
