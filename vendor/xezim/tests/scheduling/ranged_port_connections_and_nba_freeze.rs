//! Three defects behind a 16-bank arbiter testbench that wedged and granted
//! the wrong banks, all reference-validated, each reproduced in ~30 lines.
//!
//! 1. **A top-level edge block got a DERIVED instance scope.** When its clock
//!    resolved to an instance's same-named port copy ("clk" → "d.clk"), every
//!    bare NBA target compiled scope-first to the instance copy — the top
//!    signal was never written (a grant strobe stayed x forever; §22.4-style
//!    shadowing keeps scope-first lookup, so the fix validates the hint
//!    against the block's own write targets).
//! 2. **§10.4.2 — `arr[t].field <= v` never froze its element index.** The
//!    freeze walker had no MemberAccess arm, so a loop-variable index was
//!    stale (or gone) at NBA-apply time and every such write dropped.
//! 3. **§11.5.1 — `.p(arr[15:0])` port connections lost or mis-read the
//!    element select.** Labels pass through a constant part-select, so
//!    `p[11]` must read ELEMENT 11 of `arr`; the inlined
//!    `Index{RangeSelect}` shape read bit 11 of a 16-bit slice (or dropped
//!    the select entirely on the non-Ident graft path).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Top block whose clock also feeds a same-named instance port: the bare NBA
/// targets must stay top-level.
#[test]
fn top_block_nba_targets_stay_top_level() {
    let src = r#"
module inner(input logic clk, output reg [15:0] lp);
  always @(posedge clk) lp <= lp + 1;
endmodule
module duty(input logic clk, input logic lp, output logic q);
  logic [15:0] deep;
  inner i(.clk(clk), .lp(deep));       // inner port ALSO named lp
  always_ff @(posedge clk) q <= lp;
endmodule
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic rst, go;
  logic [1:0] sh;
  logic lp;
  logic q;
  duty d(.clk(clk), .lp(lp), .q(q));
  wire fire = go & (sh == 2'b00);
  int pulses = 0;
  always @(posedge clk) if (lp) pulses++;
  always_ff @(posedge clk) begin
    sh <= (rst) ? 0 : {sh[0], fire};
    lp <= (rst) ? 0 : sh[0];
  end
  initial begin
    rst = 1; go = 0;
    repeat (2) @(posedge clk);
    rst = 0;
    @(negedge clk); go = 1;
    repeat (9) @(posedge clk);
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert!(
        u(&sim, "pulses") >= 2,
        "lp must pulse (was never written at all)"
    );
}

/// NBA member writes with a loop index must capture the index at schedule
/// time.
#[test]
fn nba_member_write_freezes_loop_index() {
    let src = r#"
module tb;
  typedef struct packed {
    logic vld;
    logic [3:0] a;
    logic [1:0] hi;
    logic [1:0] lo;
  } s_t;
  logic clk = 0;
  always #5 clk = ~clk;
  s_t [15:0] arr;
  logic go = 0;
  int h13, l13, a5;
  always @(posedge clk) begin
    for (int t = 0; t < 16; t++) begin
      arr[t].vld <= go;
      if (go) begin
        arr[t].a  <= t[3:0] ^ 4'h3;
        arr[t].hi <= (t >> 2) & 2'b11;
        arr[t].lo <= t & 2'b11;
      end
    end
  end
  initial begin
    @(negedge clk); go = 1;
    repeat (2) @(posedge clk); #1;
    h13 = arr[13].hi; l13 = arr[13].lo; a5 = arr[5].a;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "h13"), 3, "hi of element 13");
    assert_eq!(u(&sim, "l13"), 1, "lo of element 13");
    assert_eq!(u(&sim, "a5"), 6, "a of element 5 (5^3)");
}

/// A ranged port connection of a packed struct array: element and member
/// reads through the port must select the labeled element.
#[test]
fn ranged_port_connection_selects_elements() {
    let src = r#"
package pk;
  typedef struct packed {
    logic vld;
    logic [7:0] a;
    logic [7:0] b;
    logic w;
    logic [1:0] hi;
    logic [1:0] lo;
  } s_t;
endpackage
module duty(input pk::s_t [15:0] p, output logic [21:0] e11, output logic [3:0] hv);
  always_comb begin
    e11 = p[11];
    hv = {p[11].hi, p[11].lo};
  end
endmodule
module tb;
  import pk::*;
  s_t [15:0] arr;
  logic [21:0] e11;
  logic [3:0] hv;
  duty d(.p(arr[15:0]), .e11(e11), .hv(hv));
  int ev, hl;
  initial begin
    arr = '0;
    arr[11] = {1'b1, 8'h21, 8'h00, 1'b0, 2'b10, 2'b11};
    #1;
    ev = e11;
    hl = hv;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "ev"),
        0b1001000010000000001011,
        "p[11] through .p(arr[15:0]) is element 11"
    );
    assert_eq!(
        u(&sim, "hl"),
        0b1011,
        "member reads through the ranged port"
    );
}
