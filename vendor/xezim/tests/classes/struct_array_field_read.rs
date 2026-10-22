use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: `o.arr[i].f` on a class fixed-array property of packed structs must read the field bits (§7.2.1, §8.4).
/// Verified byte-for-byte against reference simulators.
#[test]
fn struct_array_field_read() {
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/classes/struct_array_field_read.sv");
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
        "struct_array_field_read failed:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "struct_array_field_read: no TAG_PASS.\nOutput:\n{combined}"
    );
}
