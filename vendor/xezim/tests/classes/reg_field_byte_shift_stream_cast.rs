use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for casting a STREAMING concat to an
/// UNPACKED queue typedef — the exact construct uvm_reg_map::do_bus_access
/// uses to place a partial-register field's value onto its byte lane.
///
/// A field at bit offset `lsb` inside a byte-addressed map is shifted up by
/// `bit_shift = lsb % (n_bytes*8)` using `bit_q_t'({<< {bits}})` (cast a bit
/// queue's stream back to an unpacked `bit [$]`) and then re-packed with
/// `{<< 8 { ... }}`. The cast target is an unpacked collection; collapsing
/// it to the queue element width (1) truncates the shift and sends a wrong
/// `data` byte — a register field write produced 0x00 instead of 0x7E (the
/// `3641` reg-map byte-lane write divergence).
///
/// A 0x3F field byte at lsb=1 must yield bus bytes {0x7E, 0x00}.
#[test]
fn reg_field_byte_shift_stream_cast() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/reg_field_byte_shift_stream_cast.sv");
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
        "Parse error in reg_field_byte_shift_stream_cast.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in reg_field_byte_shift_stream_cast.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Stream->queue cast byte-shift did not produce {{0x7E,0x00}}.\nOutput:\n{combined}"
    );
}
