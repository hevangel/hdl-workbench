use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for visitor traversal:
/// Tree traversal algorithms (top-down, bottom-up, by-level) used by
/// uvm_visitor / uvm_bottom_up_visitor_adapter / uvm_top_down_visitor_adapter.
#[test]
fn reg4712_visitor_traversal() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("reg4712_visitor_traversal.sv");
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

    assert!(
        !combined.contains("Parse errors"),
        "Parse error in reg4712_visitor_traversal.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in reg4712_visitor_traversal.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Test did not pass.\nOutput:\n{combined}"
    );
}
