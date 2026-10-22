use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: A bare `req.rdata = v` in a subclass method, where `req` is a parent member typed by the PARENT's type parameter (any parameter name), must write through the handle (IEEE 1800-2017 §8.25.1).
/// Verified byte-for-byte against reference simulators.
#[test]
fn type_param_inherited_member_write() {
    let test_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/classes/type_param_inherited_member_write.sv");
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
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !combined.contains("Parse errors"),
        "Parse error:\n{combined}"
    );
    assert!(
        !combined.contains("TAG_FAIL"),
        "type_param_inherited_member_write failed:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "type_param_inherited_member_write: no TAG_PASS.\nOutput:\n{combined}"
    );
}
