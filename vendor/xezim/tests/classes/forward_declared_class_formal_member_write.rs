use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test for a FORWARD-DECLARED class used as a
/// static-method FORMAL whose class-handle MEMBER is written inside a fork.
///
/// `typedef class FWD;` registers `FWD` as a forward declaration (and, as a
/// side effect, as a typedef placeholder). A static method's formal of that
/// type (`spawnit(FWD guard)`) must STILL be classified as a CLASS OBJECT, so
/// `guard.m_guard_process = process::self()` in the forked body stores the
/// watcher handle into the heap object rather than being silently dropped.
/// `clear()` on the same object (via a different call text) must then see a
/// non-null watcher and be able to kill it.
///
/// This is the `uvm_process_guard_base::m_process_guard` scenario: when the
/// formal was misclassified as a plain typedef, `receiver_may_be_handle`
/// returned false in the child process, the member write vanished, the guard
/// kept a null watcher, and downstream sequence cancellation stormed on
/// (the UVM sequence-parent-killed regression).
///
/// Reference-verified: the member write lands on the heap object and `clear()`
/// observes the non-null watcher (TAG_PASS).
#[test]
fn forward_declared_class_formal_member_write() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/forward_declared_class_formal_member_write.sv");
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
        "Parse error in forward_declared_class_formal_member_write.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in forward_declared_class_formal_member_write.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Test did not pass (forward-declared-class formal's member write was dropped in the forked child).\nOutput:\n{combined}"
    );
}
