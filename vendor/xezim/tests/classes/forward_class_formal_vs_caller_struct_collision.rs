use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for a CLASS-typed formal whose name collides
/// with an unrelated same-named STRUCT local in a CALLER scope.
///
/// `var_typedef_types` is a flat global map keyed by name, so a caller's
/// `BUSOP rw;` leaves `var_typedef_types["rw"] = "BUSOP"`. When the callee
/// method (with a forward-declared-class formal also named `rw`) runs, its
/// `rw.status` member access was resolved through that stale struct layout:
/// the read and the write were struct-spliced over the class handle, the
/// handle was clobbered, and the member write never reached the heap.
///
/// A class-object variable must NEVER be struct-spliced, even when a
/// same-named struct exists in an unrelated live caller frame. This is the
/// RAL predictor scenario (`uvm_reg_predictor`'s local `uvm_reg_bus_op rw`
/// vs `uvm_reg::do_predict`'s `uvm_reg_item rw` formal).
///
/// Reference-verified: the class handle's member write reaches the heap and
/// the object identity is preserved (TAG_PASS).
#[test]
fn forward_class_formal_vs_caller_struct_collision() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/forward_class_formal_vs_caller_struct_collision.sv");
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
        "Parse error in forward_class_formal_vs_caller_struct_collision.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in forward_class_formal_vs_caller_struct_collision.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Test did not pass (a class-typed formal's member write was struct-spliced \
         away by a same-named caller-scope struct).\nOutput:\n{combined}"
    );
}
