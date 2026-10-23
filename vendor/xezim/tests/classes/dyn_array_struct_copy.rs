use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: Whole assignment (`b = a`) and argument binding of a dynamic array of UNPACKED structs must copy every element's members (§7.6, §7.2.1).
/// Verified byte-for-byte against reference simulators.
#[test]
fn dyn_array_struct_copy() {
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/classes/dyn_array_struct_copy.sv");
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
        "dyn_array_struct_copy failed:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "dyn_array_struct_copy: no TAG_PASS.\nOutput:\n{combined}"
    );
}
