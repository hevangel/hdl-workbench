use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for long procedural loop (>100,000 iterations):
/// Procedural statements inside a single process activation must not trigger
/// zero-delay livelock stall errors.
#[test]
fn procedural_loop_stall() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/scheduling");
    let test_file = test_dir.join("procedural_loop_stall.sv");
    assert!(
        test_file.exists(),
        "Test file not found: {}",
        test_file.display()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute xezim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        !combined.contains("simulation STALLED"),
        "Simulation falsely reported STALLED for long procedural loop:\n{combined}"
    );

    assert!(
        combined.contains("TAG_PASS"),
        "Test did not pass.\nOutput:\n{combined}"
    );
}
