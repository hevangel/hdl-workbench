//! A failed `std::randomize()` must leave every target untouched (§18.12).
//!
//! §18.6.1/§18.12: `std::randomize()` returns 0 and the variables keep the
//! values they had. The call snapshots its targets before the trial draws, but
//! the snapshot kept only the integral SCALAR targets — the filter
//! `array_operand_name(a).is_none()` excluded arrays. The trial draws and the
//! per-item repair write the array targets as well, and the joint solver
//! commits an array element by element, so a call that ended unsatisfied left
//! the rejected draw behind in the array while the scalars were put back. The
//! class `rand` path already restores both, and is checked here too so the
//! fix cannot drift from it.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| {
            m.starts_with("T|") || m.starts_with("TEST_") || m.starts_with("CHECK_FAIL")
        })
        .collect()
}

#[test]
fn std_randomize_failed_restores_array_targets() {
    assert_eq!(
        t_lines(include_str!("randomize_std_failed_restores_arrays_515.sv")),
        [
            "T|checks=19 failed=0",
            "TEST_PASS",
        ]
    );
}