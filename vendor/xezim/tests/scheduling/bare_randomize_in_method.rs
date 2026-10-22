use std::path::Path;
use std::process::Command;

/// §18.11: a bare `randomize()` (no `this.`) inside a class METHOD is the
/// implicit per-object randomize built-in — it must dispatch to the solver and
/// honour the object's declared constraints, exactly like `this.randomize()`.
///
/// Regression: a bare call is NOT a *declared* class method (the source never
/// lists it), so the unqualified-method-in-class dispatch guarded on
/// `class_has_method` was false and let the call fall through and silently
/// return 0 WITHOUT solving. A class whose constructor/draw method called
/// bare `randomize()` left `rand` fields unconstrained at their default (0):
/// UVM's `obj_example_seq.count` (`rand int count; count inside {[5:10]};`,
/// randomized from inside a method) stayed 0, so `repeat(count)` ran an
/// unrandomized/inside range and the 35objections/03basic/06xbus transfer
/// bench drove an unbounded number of transactions (endless recorder/stream
/// growth) instead of the bounded 5..10 the reference simulator produces.
///
/// Both the bare and the `this.`-qualified call must yield 400 in-range draws
/// from 400 solves. Matched byte-for-byte against the reference simulator
/// (the reference simulator):
///   TAG_BARESOLVE bare=400 qualified=400
///   TAG_PASS bare=400 qualified=400
#[test]
fn bare_randomize_in_method() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/scheduling");
    let test_file = test_dir.join("bare_randomize_in_method.sv");
    assert!(
        test_file.exists(),
        "Test file not found: {}",
        test_file.display()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg("--simulate")
        .arg("-s")
        .arg("bare_rand")
        .arg(test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute xezim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    // Both the bare and the qualified forms must honour the inside constraint.
    assert!(
        combined.contains("TAG_PASS bare=400 qualified=400"),
        "bare randomize() inside a class method did not solve its constraint.\nOutput:\n{combined}"
    );
}
