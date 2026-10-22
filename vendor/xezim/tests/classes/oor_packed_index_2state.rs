use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: an OUT-OF-RANGE element index into a packed
/// multi-dimensional array relativizes x per the element's 4-state/2-state
/// type (IEEE 1800 §11.5.1, §5.8). A 4-STATE (`logic`) out-of-range element
/// reads x; a 2-STATE (`bit`) one reads 0. Xezim returned x for BOTH, leaking
/// x into the byte-3 lane of the `99partial_ro` reg bit-bash scenario.
#[test]
fn oor_packed_index_2state() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/oor_packed_index_2state.sv");
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
        "Parse error in oor_packed_index_2state.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in oor_packed_index_2state.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Out-of-range packed element read did not relativize by two-state type.\nOutput:\n{combined}"
    );
}
