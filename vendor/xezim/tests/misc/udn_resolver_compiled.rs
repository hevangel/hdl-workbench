//! Issue #137: the §6.6.7 resolver dispatch — `sum_currents('{...})`, an
//! assignment-pattern actual into a DYNAMIC-ARRAY formal — never compiled,
//! so every RNM node re-ran its resolution function on the AST path each
//! settle (~25 µs per eval for a 4-term multiply-accumulate; 95% of runtime
//! on the reporter's DAC model).
//!
//! Three gaps, now closed together:
//!  * the ports gate rejected ANY dimensioned formal — a single-unsized-dim
//!    Input formal fed by a FIXED assignment pattern now binds as per-element
//!    registers (the call site is monomorphic: the nettype machinery emits
//!    one Ordered element per driver);
//!  * `foreach` over a register-bound local array had no compiled form — it
//!    now UNROLLS at the constant element count, the loop var folding to a
//!    direct element register;
//!  * the purity walker had no `Foreach` arm, so the implicitly-declared
//!    loop variable read as a free module name and the body was branded
//!    impure (the issue's `Expr_Call_impure` sightings).
//!
//! Values are pinned against the AST path (bit-identical before/after) and
//! the issue's own reference numbers. Measured on the issue's reproducer:
//! wall 2.92 s -> 0.35 s, settle_ca 2455 ms -> 0.

use xezim::simulate;

fn msgs(src: &str) -> Vec<String> {
    simulate(src, 4_000_000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn resolver_with_dynamic_array_formal_compiles_and_matches() {
    // The issue's reproducer, shortened to 2000 steps: 4 drivers onto one
    // isum_net node, closed through a passive load.
    let out = msgs(
        r#"
package isum_pkg;
  function automatic real sum_currents(input real drivers[]);
    real total;
    total = 0.0;
    foreach (drivers[i]) total += drivers[i];
    return total;
  endfunction
  nettype real isum_net with sum_currents;
endpackage

module blk #(parameter real G = 1.0e-3, parameter real WP = 2.5e8)
            (input real in_val, input real v_node, output isum_net out_val);
  localparam real DT = 1.0e-11;
  real x, dx;
  assign dx      = -WP * x + WP * (in_val - v_node) * G;
  assign out_val = x * G;
  initial x = 0.0;
  always #1 x = x + DT * dx;
endmodule

module node_load (input isum_net i_in, output real v_out);
  assign v_out = i_in * 500.0;
endmodule

module top;
  import isum_pkg::*;
  real in_val, out_val;
  isum_net vout_isum;
  blk #(.G(1.0e-3), .WP(2.5e8)) b0 (.in_val(in_val), .v_node(out_val), .out_val(vout_isum));
  blk #(.G(2.0e-3), .WP(3.0e8)) b1 (.in_val(in_val), .v_node(out_val), .out_val(vout_isum));
  blk #(.G(4.0e-3), .WP(3.5e8)) b2 (.in_val(in_val), .v_node(out_val), .out_val(vout_isum));
  blk #(.G(8.0e-3), .WP(4.0e8)) b3 (.in_val(in_val), .v_node(out_val), .out_val(vout_isum));
  node_load nl (.i_in(vout_isum), .v_out(out_val));
  integer i;
  initial begin
    in_val = 1.0;
    for (i = 0; i < 2000; i = i + 1)
      #1 in_val = 1.0 + 0.1 * ((i % 200) - 100) / 100.0;
    $display("R_%.9f_%.9f", out_val, vout_isum);
    $finish;
  end
endmodule
"#,
    );
    // Verified bit-identical between the AST path (pre-fix binary) and the
    // compiled path at 2000 steps.
    assert!(
        out.contains(&"R_0.041253076_0.000082506".to_string()),
        "{out:?}"
    );
}

#[test]
fn foreach_over_local_array_in_pure_fn() {
    // The unroll + const-fold on a #129-style local buffer, integral typed,
    // with the loop var used BOTH as index and as a value.
    let out = msgs(
        r#"
module top;
  function automatic int weighted(input int a, input int b);
    int buf_[4];
    int acc;
    buf_[0] = a; buf_[1] = b; buf_[2] = a + b; buf_[3] = a - b;
    acc = 0;
    foreach (buf_[k]) acc += buf_[k] * (k + 1);
    return acc;
  endfunction
  logic [31:0] x, y, r;
  assign r = weighted(x, y);
  initial begin
    x = 10; y = 3;
    // 10*1 + 3*2 + 13*3 + 7*4 = 83
    #1 $display("W_%0d", r);
    x = 100; y = 50;
    #1 $display("X_%0d", r);
  end
endmodule
"#,
    );
    assert!(out.contains(&"W_83".to_string()), "{out:?}");
    // 100 + 100 + 450 + 200 = 850
    assert!(out.contains(&"X_850".to_string()), "{out:?}");
}

#[test]
fn resolver_result_tracks_driver_changes() {
    // The resolver must RE-EVALUATE when a driver changes — the CA
    // dependency machinery has to keep following the inlined callee's reads.
    let out = msgs(
        r#"
package p2;
  function automatic real rsum(input real d[]);
    real t;
    t = 0.0;
    foreach (d[i]) t += d[i];
    return t;
  endfunction
  nettype real rnet with rsum;
endpackage
module drv (input real v, output p2::rnet o);
  assign o = v;
endmodule
module top;
  import p2::*;
  real a, b;
  rnet n;
  drv d0 (.v(a), .o(n));
  drv d1 (.v(b), .o(n));
  initial begin
    a = 1.5; b = 2.25;
    #1 $display("S_%.3f", n);
    a = -0.5;
    #1 $display("T_%.3f", n);
  end
endmodule
"#,
    );
    assert!(out.contains(&"S_3.750".to_string()), "{out:?}");
    assert!(out.contains(&"T_1.750".to_string()), "{out:?}");
}
