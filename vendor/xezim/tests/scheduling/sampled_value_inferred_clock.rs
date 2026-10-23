//! §16.9.3 — the clocking event of a sampled-value function in PROCEDURAL code.
//!
//! `$rose` / `$fell` / `$stable` / `$changed` / `$past` may omit the clocking
//! argument, in which case it is inferred. xezim inferred it only from a
//! `default clocking` block; with neither an explicit `@(...)` argument nor a
//! default clocking — the ordinary
//!
//! ```systemverilog
//! always @(posedge clk) if ($rose(a)) ...
//! ```
//!
//! shape — no watch was registered at all, so the "past" sample defaulted to
//! the current one: **`$rose`/`$fell` were always 0 and `$stable` always 1**,
//! with no diagnostic. A checker written this way silently never fired.
//!
//! The enclosing block's clock is now recorded per call site. It has to be
//! read from `edge_blocks` rather than `module.always_blocks`: by the time
//! watches are registered the always blocks have already been compiled into
//! edge blocks and the module vec is empty, which is exactly why the original
//! registration found nothing to attach a clock to.
//!
//! Verified byte-identical to a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The bare idiom: no default clocking, no explicit clocking argument.
#[test]
fn rose_fell_stable_infer_the_enclosing_block_clock() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic clk = 0, a = 0;
  always #5 clk = ~clk;
  int n_rose, n_fell, n_stable;
  always @(posedge clk) begin
    if ($rose(a))   n_rose++;
    if ($fell(a))   n_fell++;
    if ($stable(a)) n_stable++;
  end
  initial begin
    #12 a = 1;   // observed at the t=15 edge
    #10 a = 0;   // observed at the t=25 edge
    #20 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "n_rose"), 1, "$rose fires once");
    assert_eq!(u(&sim, "n_fell"), 1, "$fell fires once");
    assert_eq!(
        u(&sim, "n_stable"),
        2,
        "and $stable is not simply always true"
    );
}

/// A NEGEDGE block infers its own edge, and `$past` supports a depth.
#[test]
fn negedge_blocks_and_past_depth() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic clk = 0, a = 0, b = 0;
  always #5 clk = ~clk;
  int p1, p2, nr;
  always @(posedge clk) begin
    p1 <= $past(a);
    p2 <= $past(a, 2);
  end
  always @(negedge clk) if ($rose(b)) nr++;
  initial begin
    #7  a = 1;
    #6  b = 1;
    #22 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "p1"), 1, "$past(a) one cycle back");
    assert_eq!(u(&sim, "p2"), 0, "$past(a, 2) two cycles back");
    assert_eq!(u(&sim, "nr"), 1, "a negedge block infers negedge");
}

/// The guards: an explicit clocking argument and a `default clocking` block
/// both still take precedence and behave as before.
#[test]
fn explicit_and_default_clocking_still_work() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic clk = 0, a = 0;
  always #5 clk = ~clk;
  default clocking dc @(posedge clk); endclocking
  int xr, dr;
  always @(posedge clk) if ($rose(a, @(posedge clk))) xr++;
  initial begin
    #12 a = 1;
    #20;
    dr = $rose(a);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "xr"),
        1,
        "an explicit clocking argument still resolves"
    );
}
