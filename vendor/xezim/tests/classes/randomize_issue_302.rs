//! Array sum constraints with a random window centre (issue #302, reported
//! by Ganesh-TNL).
//!
//! - §11.8.2 / §6.24.1: `v0 == (v2 * 32'(p0) * (p1 + 1))` is compared as
//!   unsigned 32 bits, so the signed `v0` is reinterpreted, not extended.
//!   The solver's "must not re-sign" side condition on `v0` blocked every
//!   propagation of the equality, including its modulo-2^32 reading, and
//!   `v0` was guessed over 2^32 values. Such a condition only guards a
//!   reinterpretation, which an equality at that width (or a sum's carry
//!   identity) does not see.
//! - §7.12.3: the two halves of `sum() inside {[lo:hi]}` with rand bounds,
//!   or a `>=`/`<=` pair, each made their own carry pair for the same sum;
//!   decided inconsistently, they conflicted only at the last element. The
//!   same sum now shares one pair.
//!
//! The reference simulator passes every case.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn issue_302_sum_window_cases() {
    assert_eq!(
        t_lines(include_str!("randomize_issue_302_sum_window.sv")),
        [
            "T|B1 preassign+expr+varrange: PASS",
            "T|B2 expr+constrange: PASS",
            "T|B3 center-fixed+varrange: PASS",
            "T|B4 literal+varrange: PASS",
            "T|C2 sum in two items: PASS",
        ]
    );
}

/// Windows built from `>=`/`<=` pairs and `inside` ranges, casts in the
/// summed term, signed byte sums, and `product()` / `xor()` reductions.
#[test]
fn sum_window_shapes() {
    let expected: Vec<String> = [
        "W1", "W2", "W3", "W4", "W5", "P1", "P2", "X1", "X2", "S1", "S2",
    ]
    .iter()
    .map(|k| format!("T|{k} PASS"))
    .collect();
    assert_eq!(
        t_lines(include_str!("randomize_sum_window_shapes.sv")),
        expected
    );
}
