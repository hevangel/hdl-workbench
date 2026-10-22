use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for `$typename` of parameterized classes
/// (IEEE 1800-2017 §21.7). Validates the UVM support case
/// (xezim UVM support case): a parameterized class handle renders as
/// "class <name> #(<args>)" with type args as "class <name>" recursively
/// and value args as their literal.
#[test]
fn typename_param_class() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("typename_param_class.sv");
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
        "Parse error in typename_param_class.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in typename_param_class.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Test did not pass.\nOutput:\n{combined}"
    );
}
