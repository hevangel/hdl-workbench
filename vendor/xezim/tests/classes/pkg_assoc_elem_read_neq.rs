use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: reading a package-scope associative-array
/// element via its qualified `pkg::arr[key]` form and comparing it against a
/// wider `logic[63:0]` value (the UVM register-kit reset-mirror check /
/// Mantis 7075 shape).
///
/// Regression: the package-qualifier strip in `strip_package_lvalue` cloned
/// the `Ident(<pkg>)` reference node, inheriting its
/// `cached_resolved_name == <pkg>` (once the package name was itself
/// memoized). Every stripped `<member>` then resolved back to the PACKAGE
/// name and read all-x, so ~49 of 50 comparisons saw a 1-bit X and the check
/// failed. Fix resets the clone's `cached_resolved_name`.
///
/// Verified byte-for-byte against reference simulators (errors=0 / TAG_PASS).
#[test]
fn pkg_assoc_elem_read_neq() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/pkg_assoc_elem_read_neq.sv");
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
        !combined.contains("Parse error"),
        "Parse error in pkg_assoc_elem_read_neq.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in pkg_assoc_elem_read_neq.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Package assoc-array element RHS read returned all-x.\nOutput:\n{combined}"
    );
}
