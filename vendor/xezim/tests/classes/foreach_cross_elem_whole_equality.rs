use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: WHOLE-STRUCT cross-element equalities in a
/// rand FIXED 2-D ARRAY OF PACKED STRUCTS — the 5446 reg-map coupling
/// (`cfg[0][0] == cfg[1][0]`). The Eq arm of `solve_forced` only wrote a
/// rand collection element when exactly ONE side of `==` was an element key
/// (and `coll_elem_expr_key` does not even peel a 2-D fixed-array element),
/// so a whole-struct equality between two array elements died in the
/// generate-and-test backstop and randomize() returned 0. Now both sides are
/// recognised as whole array elements and RHS's value is copied into LHS.
#[test]
fn foreach_cross_elem_whole_equality() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/foreach_cross_elem_whole_equality.sv");
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
        "Parse error in foreach_cross_elem_whole_equality.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in foreach_cross_elem_whole_equality.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Whole-struct cross-element equality was not solved.\nOutput:\n{combined}"
    );
}
