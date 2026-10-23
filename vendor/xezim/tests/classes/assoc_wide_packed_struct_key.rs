use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: An associative array keyed by a packed struct wider than 64 bits must round-trip keys through foreach with their members intact (§7.8.2).
/// Verified byte-for-byte against reference simulators.
#[test]
fn assoc_wide_packed_struct_key() {
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/classes/assoc_wide_packed_struct_key.sv");
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
        "assoc_wide_packed_struct_key failed:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "assoc_wide_packed_struct_key: no TAG_PASS.\nOutput:\n{combined}"
    );
}
