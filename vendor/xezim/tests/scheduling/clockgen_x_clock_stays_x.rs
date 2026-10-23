//! Pure-SystemVerilog self-test: an uninitialized 4-state `logic` driven by
//! `always #N clk = ~clk` must NOT produce a synthetic clock.
//!
//! In IEEE 1800 4-state semantics `~1'bx == 1'bx`, so a `logic clk;` with no
//! initializer stays X forever and `always #N clk = ~clk` never toggles it —
//! no posedge. xezim's built-in ClockGen fast path used to treat any non-One
//! bit (X/Z included) as 0 and flip it to 1, synthesising a 0->1->0 clock out
//! of an X signal and inventing posedges the reference never fires. A clock
//! with an explicit `= 0` init must keep toggling normally.
//!
//! The driver shells the standalone pure-SV file
//! `tests/clockgen_x_clock_stays_x.sv` through the CLI (same pattern as
//! reg3456_pure_sv / reg4712_visitor_traversal) and asserts both of its
//! TAG_PASS banners.

use std::path::Path;
use std::process::Command;

const SV_FILE: &str = "clockgen_x_clock_stays_x.sv";

#[test]
fn clockgen_x_clock_stays_x() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join(SV_FILE);
    assert!(
        test_file.exists(),
        "Test file not found: {}",
        test_file.display()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute xezim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    // Parse / simulation errors.
    assert!(
        !combined.contains("Parse errors"),
        "Parse error:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error:\n{combined}"
    );

    // X-start `clkX = ~clkX` must stay X forever (reference idles; the bug
    // fired 6 synthetic posedges). Pre-fix this printed TAG_FAIL_clock_active.
    assert!(
        combined.contains("TAG_PASS_clock_stays_x"),
        "X-start self-toggle clock generated posedges (bug present):\n{combined}"
    );
    // An X clock seeded 0 by the TB at t=20 must REVIVE (posedges at
    // 25,35,45,55). A retire-based fix left the generator dead forever.
    assert!(
        combined.contains("TAG_PASS_late_seed"),
        "late-seeded clock did not revive:\n{combined}"
    );
    // A Z-seeded clock's first fire is a real ~Z == X value change; the
    // signal must read X afterwards, not stay stuck at Z.
    assert!(
        combined.contains("TAG_PASS_z_goes_x"),
        "Z-seeded clock did not transition to X:\n{combined}"
    );
    // Explicit `clk0 = 0` self-toggle must still clock normally (pe0 == 6).
    assert!(
        combined.contains("TAG_PASS_explicit_toggles"),
        "explicit clk=0 self-toggle failed to clock (regression):\n{combined}"
    );
}
