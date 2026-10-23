use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: a class CONSTRUCTOR's bare-name
/// (unqualified) dynamic-array MEMBER write (`value = new[1]`) must size
/// the new object's own member, NOT an enclosing (inlined) caller task's
/// same-named `ref` dynamic-array formal.
///
/// UVM 4251: `uvm_reg_item::new` runs `value = new[1]` on its `value[]`
/// member, while `burst_read(ref uvm_reg_data_t value[])` hands a
/// 64-element `read_block` in; the constructor's bare `value` also names
/// that live `ref` formal. Per §8.9/§8.10 an unqualified name in a
/// method/constructor resolves to (1) the CURRENT frame's own local/
/// formal, (2) a class member of `this`, and only then (3) an ENCLOSING
/// frame's same-named `ref` formal. xezim routed the `= new[1]` write
/// target through the enclosing frame's rename, shrinking the caller's
/// 64-element array to 1 instead of sizing the member (the mem burst-read
/// "size 1 different from burst_size 64" error). The reference simulator
/// keeps the caller's array at 64 and writes the member to 1.
#[test]
fn ctor_bare_member_dynarray_size() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/ctor_bare_member_dynarray_size.sv");
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
        "Parse error in ctor_bare_member_dynarray_size.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in ctor_bare_member_dynarray_size.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Constructor bare-name dynamic-array member write collapsed the caller's ref array.\nOutput:\n{combined}"
    );
}
