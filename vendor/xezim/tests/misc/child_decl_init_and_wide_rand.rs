//! Two reported issues, both reference-validated.
//!
//! 1. **#81 — a declaration initializer in an INSTANTIATED module that reads
//!    another module-level variable folded to 0 / "".** Reported as a
//!    `string`/`$sformatf` bug; it is neither — `int n = 5; int m = n + 1;`
//!    in a child module produced `m == 1`, i.e. `n` read as 0. The inlining
//!    path const-evaluated initializers against the PARAMETER map only, where
//!    a sibling variable does not appear; the top-level path instead lowers a
//!    non-constant initializer to a procedural assignment. Child initializers
//!    now defer the same way, onto the static-init list so §6.8's "shall occur
//!    before any initial or always block is started" holds — the instance's
//!    own `initial` blocks travel via a list the scheduler drains FIRST, so
//!    an ordinary initial block would have run too late.
//!
//! 2. **#80 — `randomize()` returned a constant 0 for any rand field wider
//!    than 64 bits.** Two solve paths ended in `if width <= 64 { … }` with no
//!    else, leaving `Value::zero(width)` while `randomize()` still reported
//!    success. Values wider than 64 bits are now filled 64 bits at a time.

use xezim::simulate;

fn text(sim: &xezim::compiler::Simulator, n: &str) -> String {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_sv_string()
}

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A child module's initializers must see sibling module-level variables —
/// for strings, concatenations and plain integers alike.
#[test]
fn child_decl_initializer_sees_sibling_variable() {
    let src = r#"
module child;
  string dir = "abcdef";
  string cmd = $sformatf("X%sY", dir);
  string cat = {dir, "/z"};
  int    n   = 5;
  int    m   = n + 1;
  int    k   = m * 2;          // chains through another deferred initializer
endmodule
module tb;
  child u ();
  initial #1 $finish;
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        text(&sim, "u.cmd"),
        "XabcdefY",
        "$sformatf over a sibling var"
    );
    assert_eq!(
        text(&sim, "u.cat"),
        "abcdef/z",
        "concatenation over a sibling var"
    );
    assert_eq!(
        u(&sim, "u.m"),
        6,
        "int initializer reading a sibling (was 1)"
    );
    assert_eq!(u(&sim, "u.k"), 12, "declaration order is preserved");
}

/// The initializer must land before the instance's own initial block reads it
/// (§6.8), and constant initializers must keep working.
#[test]
fn child_decl_initializer_runs_before_initial_blocks() {
    let src = r#"
module child;
  int  base = 7;
  int  derived = base + 1;
  int  seen;
  string lit = "plain";
  initial seen = derived;      // reads it with no delay
endmodule
module tb;
  child u ();
  initial #1 $finish;
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "u.derived"), 8);
    assert_eq!(
        u(&sim, "u.seen"),
        8,
        "initializer must precede the initial block"
    );
    assert_eq!(
        text(&sim, "u.lit"),
        "plain",
        "constant initializers still fold"
    );
}

/// A rand field wider than 64 bits must actually be randomized.
#[test]
fn randomize_fills_fields_wider_than_64_bits() {
    let src = r#"
class C;
  rand logic [31:0]  w32;
  rand logic [65:0]  w66;
  rand logic [127:0] w128;
  rand logic [255:0] w255;
endclass
module tb;
  int nz32 = 0, nz66 = 0, nz128 = 0, nz255 = 0, ok = 1;
  int hi128 = 0;
  initial begin
    C c;
    c = new();
    repeat (12) begin
      if (!c.randomize()) ok = 0;
      if (c.w32  != 0) nz32++;
      if (c.w66  != 0) nz66++;
      if (c.w128 != 0) nz128++;
      if (c.w255 != 0) nz255++;
      // the HIGH half must vary too, not just the low 64 bits
      if (c.w128[127:64] != 0) hi128++;
    end
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "ok"), 1, "randomize must succeed");
    assert!(u(&sim, "nz32") >= 11, "32-bit sanity");
    assert!(u(&sim, "nz66") >= 11, "66-bit field was always 0");
    assert!(u(&sim, "nz128") >= 11, "128-bit field was always 0");
    assert!(u(&sim, "nz255") >= 11, "255-bit field was always 0");
    assert!(
        u(&sim, "hi128") >= 11,
        "bits above 64 must be randomized, not just the low word"
    );
}
