//! §6.20.2 — a package parameter's DECLARED type fixes its signedness, as a
//! module parameter's does. The package paths only ever set the flag from
//! the declaration, so a signed initializer leaked into an unsigned
//! parameter: `parameter bit B = 1` read as a signed 1-bit -1, `B === 1` was
//! false and `B + 1` was 0. This is the shape behind the AVIP config
//! `if (MASTER_AGENT_ACTIVE === 1)`, which took the passive branch and never
//! built the driver.
//!
//! Every expected line was cross-checked against the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn package_params_take_declared_signedness() {
    let o = lines(
        r#"
package gp;
  typedef int my_t;
  typedef bit [3:0] u4_t;
  parameter bit MA = 1;
  parameter logic [3:0] P4 = -1;
  parameter [7:0] PD = -3;
  parameter P8 = 8'hF0;
  parameter int PI = -5;
  parameter byte PB = 8'hF0;
  parameter my_t PT = -2;
  parameter u4_t PU = -1;
  parameter signed [3:0] PS = 4'hF;
  parameter PN = -7;
  localparam bit LB = 1;
endpackage
module top;
  import gp::*;
  initial begin
    $display("MA %0d %0d %0d %0d", MA == 1, MA + 1, int'(MA), MA);
    $display("P4 %0d %0d", P4, P4 + 1);
    $display("PD %0d %0d", PD, PD + 1);
    $display("P8 %0d %0d", P8, P8 > 100);
    $display("PI %0d PB %0d PT %0d PU %0d PS %0d PN %0d", PI, PB, PT, PU, PS, PN);
    $display("LB %0d %0d", LB == 1, gp::LB + 1);
    $display("q %0d %0d", gp::MA == 1, gp::PU + 0);
  end
endmodule
"#,
    );
    for want in [
        "MA 1 2 1 1",
        "P4 15 16",
        "PD 253 254",
        "P8 240 1",
        "PI -5 PB -16 PT -2 PU 15 PS -1 PN -7",
        "LB 1 2",
        "q 1 15",
    ] {
        assert!(o.iter().any(|l| l == want), "missing {want:?}: {o:?}");
    }
}

#[test]
fn unsigned_package_bit_selects_branch_in_class_code() {
    let o = lines(
        r#"
package gp;
  parameter bit ACTIVE_P = 1;
endpackage
package p;
  import gp::*;
  typedef enum bit { PASSIVE = 0, ACTIVE = 1 } ap_e;
  class cfg;
    ap_e is_active;
    function void setup();
      if (ACTIVE_P === 1) is_active = ap_e'(ACTIVE);
      else is_active = ap_e'(PASSIVE);
    endfunction
  endclass
endpackage
module top;
  import p::*;
  initial begin
    cfg c = new;
    c.setup();
    $display("is_active=%0d", c.is_active);
  end
endmodule
"#,
    );
    assert!(o.iter().any(|l| l == "is_active=1"), "{o:?}");
}
