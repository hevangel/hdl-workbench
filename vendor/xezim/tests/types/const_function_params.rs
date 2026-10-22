//! IEEE 1800-2017 §13.4.3 constant functions in parameter initializers.
//!
//! `parameter LOG_X = log2(X)` used to be DEFERRED with value 0 during
//! elaboration (function calls couldn't be const-evaluated), so every
//! dependent parameter wrapped (`LOG_X - 6` → 4294967290) and dependent port
//! widths collapsed — on a customer gate-level design this destroyed 55-bit
//! request buses (elaborated as 1-bit) and the memory-read flow with them.

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn log2_style_param_resolves_at_elaboration() {
    // The classic vendor ceil-log2: for-loop, assignment to the function
    // name, and an increment in the for-step.
    let sim = simulate(
        r#"
module sub #(
  parameter SIZE = 64,
  parameter LOG_SIZE = clog2f(SIZE),
  parameter LOG_SIZE_64B = LOG_SIZE - 6,
  parameter REQ_W = 55
) (
  input  [REQ_W-1:0] req,
  output [LOG_SIZE-1:0] cnt
);
  function integer clog2f(input integer value);
    integer v;
    begin
      v = value - 1;
      for (clog2f = 0; v > 0; clog2f = clog2f + 1)
        v = v >> 1;
    end
  endfunction
  assign cnt = req[LOG_SIZE-1:0];
  initial $display("W=%0d,%0d,%0d,%0d", LOG_SIZE, LOG_SIZE_64B, $bits(req), $bits(cnt));
endmodule
module top;
  wire [54:0] req = 55'h5A;
  wire [5:0] cnt;
  sub u (.req(req), .cnt(cnt));
  initial #1 $finish;
endmodule
"#,
        100,
    )
    .expect("sim");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "W=6,0,55,6"),
        "const-fn param sizing wrong; output: {:?}",
        msgs
    );
}

#[test]
fn recursive_and_if_based_const_fns() {
    // if/else + recursion + a localparam consumer in the TOP module.
    let sim = simulate(
        r#"
module top;
  function integer rlog2(input integer n);
    if (n <= 1) rlog2 = 0;
    else rlog2 = 1 + rlog2((n + 1) / 2);
  endfunction
  localparam W = rlog2(1024);
  localparam W2 = rlog2(1000);
  reg [W-1:0] bus;
  initial begin
    $display("R=%0d,%0d,%0d", W, W2, $bits(bus));
    $finish;
  end
endmodule
"#,
        100,
    )
    .expect("sim");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "R=10,10,10"),
        "recursive const fn wrong; output: {:?}",
        msgs
    );
}

#[test]
fn package_const_fn_param_via_wildcard_import() {
    let sim = simulate(
        r#"
package szpkg;
  function automatic integer plog2(input integer value);
    integer v;
    begin
      v = value - 1;
      for (plog2 = 0; v > 0; plog2 = plog2 + 1) v = v >> 1;
    end
  endfunction
  parameter DEPTH = 128;
  parameter AW = plog2(DEPTH);
endpackage
module top;
  import szpkg::*;
  reg [AW-1:0] addr;
  initial begin $display("PKG=%0d,%0d", AW, $bits(addr)); $finish; end
endmodule
"#,
        100,
    )
    .expect("sim");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        msgs.iter().any(|m| m == "PKG=7,7"),
        "package const-fn param wrong; output: {:?}",
        msgs
    );
}

#[test]
fn log_fn_audit_case_return_width_and_direct_range() {
    // Reference-verified (byte-identical output): case-based log2, narrow
    // return-type truncation, floor-log2 NAMED clog2 (real body wins over
    // the $clog2 name assumption), and a function call directly in a range
    // dimension with no parameter in between.
    let sim = simulate(
        r#"
module top;
  function integer wlog2(input integer value);
    begin
      wlog2 = 0;
      while (2**wlog2 < value) wlog2 = wlog2 + 1;
    end
  endfunction
  function integer caselog2(input integer d);
    case (d)
      2: caselog2 = 1;
      4: caselog2 = 2;
      8: caselog2 = 3;
      16: caselog2 = 4;
      default: caselog2 = 0;
    endcase
  endfunction
  function [3:0] narrow(input integer v);
    narrow = v;
  endfunction
  function integer clog2(input integer v);
    integer t;
    begin
      t = v; clog2 = 0;
      while (t > 1) begin t = t >> 1; clog2 = clog2 + 1; end
    end
  endfunction
  localparam W1 = wlog2(64);
  localparam W2 = caselog2(16);
  localparam W3 = narrow(20);
  localparam W4 = clog2(64);
  localparam W5 = clog2(65);
  reg [W1-1:0] b1; reg [W2-1:0] b2; reg [W4-1:0] b4; reg [W5-1:0] b5;
  reg [wlog2(256)-1:0] b6;
  initial begin
    $display("A=%0d,%0d,%0d,%0d,%0d bits=%0d,%0d,%0d,%0d,%0d",
      W1, W2, W3, W4, W5, $bits(b1), $bits(b2), $bits(b4), $bits(b5), $bits(b6));
    $finish;
  end
endmodule
"#,
        100,
    )
    .expect("sim");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        msgs.iter().any(|m| m == "A=6,4,4,6,6 bits=6,4,6,6,8"),
        "log-fn audit mismatch; output: {:?}",
        msgs
    );
}

#[test]
fn selected_locals_in_constant_functions() {
    let sim = simulate(
        r#"
module top;
  localparam A = rotate_slot(5);
  localparam B = low_part(21);
  localparam C = gap_count(5, A, 2);
  localparam TOTAL = A + B + C;

  function [1:0] rotate_slot(input integer count);
    integer left;
    reg [2:0] cursor;
    begin
      cursor = 0;
      left = count;
      while (left != 0) begin
        cursor[1:0] = cursor[1:0] - 1'b1;
        left = left - 1;
      end
      rotate_slot = cursor[1:0];
    end
  endfunction

  function [3:0] low_part(input integer value);
    integer temp;
    begin
      temp = value;
      low_part = temp[3:0];
    end
  endfunction

  function [3:0] gap_count(
    input integer needed,
    input reg [1:0] first,
    input reg [1:0] last
  );
    integer loops;
    begin
      loops = 0;
      while (((4 - first) + last + 4*loops) < needed)
        loops = loops + 1;
      gap_count = loops[3:0];
    end
  endfunction

  logic [TOTAL-1:0] payload;
  leaf #(.OFFSET(2)) u_leaf();
  initial begin
    $display("SEL=%0d,%0d,%0d,%0d,%0d", A, B, C, TOTAL, $bits(payload));
    $finish;
  end
endmodule

module leaf #(parameter integer OFFSET = 0);
  localparam SLOT = derive_slot(OFFSET);
  localparam integer COUNT = 5;

  function [1:0] derive_slot(input integer adjustment);
    integer left;
    reg [2:0] cursor;
    begin
      cursor = adjustment;
      left = COUNT;
      while (left != 0) begin
        cursor[1:0] = cursor[1:0] - 1'b1;
        left = left - 1;
      end
      derive_slot = cursor[1:0];
    end
  endfunction

  initial $display("FWD=%0d", SLOT);
endmodule
"#,
        100,
    )
    .expect("sim");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "SEL=3,5,1,9,9"),
        "selected-local constant evaluation mismatch; output: {:?}",
        msgs
    );
    assert!(
        msgs.iter().any(|m| m == "FWD=1"),
        "caller-scope constant dependency mismatch; output: {:?}",
        msgs
    );
}
