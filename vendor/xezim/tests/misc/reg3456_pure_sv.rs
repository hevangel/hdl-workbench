use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for empty-string comparison:
/// Empty string comparison must return 1, not X.
#[test]
fn reg3456_pure_sv() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("reg3456_pure_sv.sv");
    assert!(
        test_file.exists(),
        "Test file not found: {}",
        test_file.display()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg(test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute xezim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    // Check for parse errors
    assert!(
        !combined.contains("Parse errors"),
        "Parse error in reg3456_pure_sv.sv:\n{combined}"
    );

    // Check for simulation error
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in reg3456_pure_sv.sv:\n{combined}"
    );

    // Check for PASS
    assert!(
        combined.contains("TAG_PASS"),
        "Test did not pass.\nOutput:\n{combined}"
    );
}
