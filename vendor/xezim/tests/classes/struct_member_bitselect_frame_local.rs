use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for a BIT-SELECT write into a
/// FRAME-LOCAL (block-scoped) struct member — the exact shape
/// uvm_reg_map::do_bus_access uses to build a bus op's byte-enable:
///
///     foreach (adr[i]) begin
///       uvm_reg_bus_op rw_access;            // block-scoped struct
///       ...
///       rw_access.byte_en[z] = be[bus_width*i+z];   // indexed member write
///     end
///
/// The member (`byte_en`, a packed `bit [7:0]`) is pre-registered as its
/// own leaf in the current call/local frame (`rw_access.byte_en`), NOT as
/// a registered design signal. The indexed write minted a phantom
/// `rw_access.byte_en[z]` element that no whole-member read consults, so
/// the byte-enable came back 0 and the reg bit-bash predictor skipped the
/// field predict (the 4168 reg bit-bash failure).
#[test]
fn struct_member_bitselect_frame_local() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/struct_member_bitselect_frame_local.sv");
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
        "Parse error in struct_member_bitselect_frame_local.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in struct_member_bitselect_frame_local.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Block-local struct member bit-select write was lost.\nOutput:\n{combined}"
    );
}
