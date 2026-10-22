use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: a subroutine-local CLASS HANDLE that shadows
/// a same-named module signal keeps its own 32-bit lvalue width (IEEE 1800
/// §6.19 / §23.6). Xezim gave the module signal precedence in
/// `infer_lhs_width`, so under MULTI-TOP (`-s top -s dut`) the wrapper's flat
/// width-1 "top" instance signal truncated `uvm_root top = new()` to 1 bit
/// and the constructed handle was lost — `top != m_inst` fired `UVM/BAD_TOP`
/// and recursed forever (the register-map memory-access and
/// concurrent-update regressions, both previously TIMEOUT/BAD_TOP).
#[test]
fn local_shadows_sig() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/local_shadows_sig.sv");
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
        "Parse error in local_shadows_sig.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in local_shadows_sig.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Shadowed local class-handle lvalue width was truncated by the module signal.\nOutput:\n{combined}"
    );
}
