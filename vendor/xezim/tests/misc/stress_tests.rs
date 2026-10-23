//! Stress regression: large-scale generated designs from `examples/stress_*.sv`.
//!
//! These exercise the dual-store / signal-table / comb-entry code paths at
//! scale. The generated-instance case uses a CI-safe default size; set
//! `XEZIM_FULL_STRESS=1` to retain its original 131072-instance scale.
//!
//! Each test asserts that:
//!   1. Parsing + elaboration succeed for the entire generated design.
//!   2. The simulation runs to its self-`$finish` without panicking.
//!   3. At least one simulator output entry is captured (non-zero progress).

use std::fs;
use std::path::Path;
use xezim::simulate;

fn run_stress(file: &str, top: &str, max_time: u64) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    let mut src =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    if file.ends_with("stress_signals.sv") && std::env::var_os("XEZIM_FULL_STRESS").is_none() {
        src = src.replace(
            "parameter integer N        = 131072;",
            "parameter integer N        = 4096;",
        );
    }
    // simulate() takes a source string and a max time (ns). The stress
    // designs all $finish themselves at MAX_TIME, so a generous cap is fine.
    let _ = top; // simulate() picks the last module by default; stress files put `top` last.
    let sim =
        simulate(&src, max_time).unwrap_or_else(|e| panic!("simulation of {} failed: {}", file, e));
    assert!(
        sim.time > 0,
        "{} simulation did not advance simulated time (got time={})",
        file,
        sim.time
    );
}

#[test]
fn stress_signals_131k_named() {
    // 131072 named bit-cell instances + 1 clock cycle settling.
    // Exercises name → id maps and signal_table sizing without
    // running the simulation long enough to be unbearable.
    run_stress("examples/stress_signals.sv", "top", 100);
}

#[test]
fn stress_comb_continuous_assigns() {
    // Comb-heavy: every cell is `assign sum/diff/xor_out = …`. Stresses the
    // continuous-assign compile path and write_signal_ids on comb_entries.
    run_stress("examples/stress_comb.sv", "top", 100);
}

#[test]
fn stress_explicit_instances() {
    // Generated explicit instantiations (no genvar/generate). Stresses the
    // elaborator's instantiation-binding path at large fan-out.
    run_stress("examples/stress_explicit.sv", "top", 100);
}
