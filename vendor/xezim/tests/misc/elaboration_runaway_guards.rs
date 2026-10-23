//! Two elaboration runaways surfaced by the sv-tests sweep (both imported
//! ivtest should-fail cases):
//!
//! 1. §27.3: recursive module instantiation whose terminating generate
//!    branch never fires — ivtest pr2728812b reached 37 GB RSS before the
//!    kernel killed the process (the machine-wide OOM in the sweep). A
//!    depth cap (default 200, XEZIM_MAX_INST_DEPTH overrides) turns it
//!    into a clean error.
//! 2. §6.18/§6.19: `typedef T; typedef enum T {A,B} T;` — an enum whose
//!    BASE type closes the cycle on the name being defined. Undetected,
//!    every type resolver recursed until the stack overflowed (SIGABRT).

use xezim::simulate;

#[test]
fn unbounded_recursive_instantiation_errors_at_depth_cap() {
    let err = simulate(
        r#"
module sum #(parameter n = 4) (input clk, output s);
  generate
    if (n == -1) assign s = clk;      // never reached from n >= 0
    else begin
      wire s0w;
      sum #(n/2) s0 (clk, s0w);
      assign s = s0w;
    end
  endgenerate
endmodule
module top;
  logic clk;
  wire s;
  sum #(5) u (clk, s);
endmodule
"#,
        100,
    )
    .map(|_| ())
    .err()
    .expect("must be rejected, not expanded forever");
    assert!(
        err.contains("instantiation depth exceeds") && err.contains("XEZIM_MAX_INST_DEPTH"),
        "clean depth-cap error naming the override knob; got: {}",
        err
    );
    assert!(
        err.contains(" ... "),
        "the repeating instance path is truncated for readability; got: {}",
        err
    );
}

#[test]
fn enum_base_typedef_cycle_is_a_clean_error() {
    let err = simulate(
        r#"
module test;
  typedef T;
  typedef enum T { A, B } T;
  initial $display("FAILED");
endmodule
"#,
        100,
    )
    .map(|_| ())
    .err()
    .expect("circular enum-base typedef must be rejected");
    assert!(
        err.contains("circular") && err.contains("§6.18"),
        "the §6.18 circular-definition diagnostic; got: {}",
        err
    );
}
