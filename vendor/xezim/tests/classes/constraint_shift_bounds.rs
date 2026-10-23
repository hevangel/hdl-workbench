//! §18 — constraints that bound a rand variable through a SHIFT.
//!
//! The CSP's affine model covers `+`, `-`, `*` and the comparisons, so
//! `(x >> 8) == 0` was not representable: `csp_ae` failed, the relation fell
//! back to the trial loop, and on a 32-bit rand the draw has probability
//! 2**-24 of satisfying it — `randomize()` reported failure while the exact
//! equivalent `x < 256` solved at once (issue #229).
//!
//! `a << k` with a constant k is `a * 2**k`. `(a >> k) REL c` is floor
//! division by `2**k`, so each relation is one interval on `a`. Both are now
//! modelled, for UNSIGNED `a` and the logical `>>`.
//!
//! These tests assert the drawn VALUES satisfy the constraint, not merely that
//! `randomize()` returned 1 — a solver that answers with illegal values is
//! worse than one that fails loudly.

use xezim::simulate;

fn lines(src: &str, tag: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("sim");
    sim.output
        .iter()
        .filter(|o| o.message.starts_with(tag))
        .map(|o| o.message.clone())
        .collect()
}

/// Draw `reps` times and report `r=<ok> v=<value>` for a single rand scalar.
fn draws(decl: &str, reps: usize) -> Vec<(i64, u64)> {
    let src = format!(
        "{decl}\n\
module t; initial begin C c=new();\n\
  for (int n=0;n<{reps};n++) begin\n\
    int r=c.randomize();\n\
    $display(\"R r=%0d v=%0d\", r, c.x);\n\
  end $finish; end endmodule"
    );
    lines(&src, "R ")
        .iter()
        .map(|l| {
            let mut it = l.split_whitespace().skip(1);
            let r: i64 = it.next().unwrap()[2..].parse().unwrap();
            let v: u64 = it.next().unwrap()[2..].parse().unwrap();
            (r, v)
        })
        .collect()
}

#[test]
fn shr_eq_zero_bounds_the_variable() {
    let d = draws(
        "class C; rand bit[31:0] x; constraint k { (x >> 8) == 0; } endclass",
        24,
    );
    assert_eq!(d.len(), 24);
    for (r, v) in &d {
        assert_eq!(*r, 1, "(x >> 8) == 0 must solve");
        assert!(*v < 256, "x={v} violates (x >> 8) == 0");
    }
    // Not degenerate: a point solution would be a different bug.
    assert!(
        d.iter()
            .map(|x| x.1)
            .collect::<std::collections::HashSet<_>>()
            .len()
            > 1,
        "the solver should spread over 0..255, got {d:?}"
    );
}

#[test]
fn shr_eq_nonzero_bounds_both_ends() {
    // (x >> 8) == 3  <=>  768 <= x <= 1023
    let d = draws(
        "class C; rand bit[31:0] x; constraint k { (x >> 8) == 3; } endclass",
        24,
    );
    for (r, v) in &d {
        assert_eq!(*r, 1, "(x >> 8) == 3 must solve");
        assert!((768..=1023).contains(v), "x={v} is outside 768..=1023");
    }
}

#[test]
fn shr_inequalities_and_disequality() {
    for (c, lo, hi) in [
        ("(x >> 8) <  2", 0u64, 511u64),
        ("(x >> 8) <= 2", 0, 767),
        ("(x >> 4) != 0", 16, u32::MAX as u64),
        ("(x >> 8) >= 2", 512, u32::MAX as u64),
        ("(x >> 8) >  2", 768, u32::MAX as u64),
    ] {
        let d = draws(
            &format!("class C; rand bit[31:0] x; constraint k {{ {c}; }} endclass"),
            12,
        );
        for (r, v) in &d {
            assert_eq!(*r, 1, "{c} must solve");
            assert!(*v >= lo && *v <= hi, "{c}: x={v} outside {lo}..={hi}");
        }
    }
}

/// The shift may sit on either side, and the amount may be a state variable.
#[test]
fn shr_reversed_operands_and_variable_amount() {
    for decl in [
        "class C; rand bit[31:0] x; constraint k { 0 == (x >> 8); } endclass",
        "class C; int w = 8; rand bit[31:0] x; constraint k { (x >> w) == 0; } endclass",
    ] {
        let d = draws(decl, 12);
        for (r, v) in &d {
            assert_eq!(*r, 1, "must solve: {decl}");
            assert!(*v < 256, "x={v} violates the bound in {decl}");
        }
    }
}

/// A left shift by a constant is an exact multiply, so it pins the value.
#[test]
fn shl_by_constant_is_a_multiply() {
    let d = draws(
        "class C; rand bit[31:0] x; constraint k { (x << 2) == 16; } endclass",
        8,
    );
    for (r, v) in &d {
        assert_eq!(*r, 1, "(x << 2) == 16 must solve");
        assert_eq!(*v, 4, "(x << 2) == 16 has the single solution x == 4");
    }
}

/// Wider than 32 bits, and array elements under `foreach`.
#[test]
fn shr_on_wide_scalar_and_foreach_elements() {
    let d = draws(
        "class C; rand bit[63:0] x; constraint k { (x >> 40) == 0; } endclass",
        12,
    );
    for (r, v) in &d {
        assert_eq!(*r, 1, "64-bit (x >> 40) == 0 must solve");
        assert!(*v < (1u64 << 40), "x={v} violates (x >> 40) == 0");
    }

    let src = "\
class A; int w = 8; rand bit[31:0] d[4]; constraint k { foreach (d[i]) (d[i] >> w) == 0; } endclass\n\
module t; initial begin A a=new();\n\
  for (int n=0;n<8;n++) begin\n\
    int r=a.randomize();\n\
    $display(\"R r=%0d a=%0d %0d %0d %0d\", r, a.d[0], a.d[1], a.d[2], a.d[3]);\n\
  end $finish; end endmodule";
    let out = lines(src, "R ");
    assert_eq!(out.len(), 8);
    for l in &out {
        let mut it = l.split_whitespace().skip(1);
        assert_eq!(
            &it.next().unwrap()[2..],
            "1",
            "foreach shift must solve: {l}"
        );
        for f in it {
            // the first element arrives as `a=<n>`, the rest bare
            let v: u64 = f.rsplit('=').next().unwrap().parse().unwrap();
            assert!(v < 256, "element {v} violates (d[i] >> 8) == 0 in {l}");
        }
    }
}
