//! §23.9/§6.10 + §23.3.3: testbench idioms around instance ports, both
//! reference-verified.
//!
//! * A HIERARCHICAL continuous assign driving a deep sub-instance INPUT
//!   through unconnected port chains (`assign dut.mid.core.clk = tb_clk;`
//!   with `.top_clk()` no-connect). Identity connects collapse by
//!   substitution and different readers bind at DIFFERENT chain levels, so
//!   a drive left on one name silently missed the flop's clock — the DUT
//!   never clocked and every check failed. The drive now fans out across
//!   the whole port-alias chain.
//! * An EXPRESSION port actual over a parent net named like the FORMAL
//!   (`.din(din ^ 1)`) — the connect assign's RHS is parent-scoped by
//!   construction (rhs_parent_scoped), where the child scope hint made it
//!   a self-loop reading x forever.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_hpdc_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb_top",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

const HIER_DRIVE_SRC: &str = r#"package pkt_defs;
   typedef struct packed {
      logic [7:0]  payload;
      logic        strobe;
   } beat_t;
endpackage

module leaf_acc (
   input  logic clk_i,
   input  logic srst_i,
   input  pkt_defs::beat_t beat_i,
   output logic [7:0] sum_o,
   output logic tick_o
);
   always_ff @(posedge clk_i) begin
      if (srst_i) begin
         sum_o  <= '0;
         tick_o <= 1'b1;
      end else if (beat_i.strobe) begin
         sum_o  <= sum_o + beat_i.payload;
         tick_o <= !tick_o;
      end
   end
endmodule

module mid_shell (
   input  logic clk_i,
   input  logic srst_i,
   input  pkt_defs::beat_t beat_i,
   output logic [7:0] sum_o,
   output logic tick_o
);
   leaf_acc u_leaf (.clk_i(clk_i), .srst_i(srst_i), .beat_i(beat_i),
                    .sum_o(sum_o), .tick_o(tick_o));
endmodule

module top_shell (
   input  logic top_clk_i,
   input  logic top_srst_i,
   input  pkt_defs::beat_t top_beat_i,
   output logic [7:0] top_sum_o,
   output logic top_tick_o
);
   mid_shell u_mid (.clk_i(top_clk_i), .srst_i(top_srst_i), .beat_i(top_beat_i),
                    .sum_o(top_sum_o), .tick_o(top_tick_o));
endmodule

module tb_top;
   import pkt_defs::*;
   int bad = 0;
   logic tb_clk, tb_rst;
   beat_t tb_beat;
   logic [7:0] got_sum;
   logic got_tick;

   top_shell dut (
      .top_clk_i  (),
      .top_srst_i (),
      .top_beat_i (tb_beat),
      .top_sum_o  (got_sum),
      .top_tick_o (got_tick)
   );

   assign dut.u_mid.u_leaf.clk_i  = tb_clk;
   assign dut.u_mid.u_leaf.srst_i = tb_rst;

   initial begin tb_clk = 0; forever #5 tb_clk = ~tb_clk; end

   initial begin
      tb_rst = 1; tb_beat = '0;
      repeat(3) @(posedge tb_clk); #1;
      tb_rst = 0;
      if (!(got_sum === 8'h00 && got_tick === 1'b1)) bad++;
      tb_beat.payload = 8'h2C; tb_beat.strobe = 1'b1;
      @(posedge tb_clk); #1;
      if (!(got_sum === 8'h2C && got_tick === 1'b0)) bad++;
      tb_beat.payload = 8'h04;
      @(posedge tb_clk); #1;
      if (!(got_sum === 8'h30 && got_tick === 1'b1)) bad++;
      tb_beat.strobe = 1'b0;
      @(posedge tb_clk); #1;
      if (!(got_sum === 8'h30 && got_tick === 1'b1)) bad++;
      if (bad == 0) $display("TEST_PASS"); else $display("TEST_FAIL n=%0d", bad);
      $finish;
   end
endmodule
"#;

#[test]
fn hierarchical_assign_drives_deep_input_through_unconnected_ports() {
    let text = run("hier_drive", HIER_DRIVE_SRC);
    assert!(text.contains("TEST_PASS"), "hierarchical drive:\n{text}");
}

/// The same drive onto ports declared as NETS (`input wire logic clk_i`).
/// The multi-driver net fold counted the hierarchical assign and the port's
/// connect assign as two drivers of one net and merged them on a single
/// level of the alias chain, so the flop, reading another level, never
/// clocked. §23.2.2.3 makes the `input logic` ports above nets too.
#[test]
fn hierarchical_assign_drives_deep_wire_input_through_unconnected_ports() {
    let src = HIER_DRIVE_SRC
        .replace("input  logic clk_i", "input  wire logic clk_i")
        .replace("input  logic srst_i", "input  wire logic srst_i");
    assert!(src.contains("input  wire logic clk_i"));
    let text = run("hier_drive_wire", &src);
    assert!(
        text.contains("TEST_PASS"),
        "hierarchical drive (wire ports):\n{text}"
    );
}

#[test]
fn expression_actual_over_samename_parent_net_not_x() {
    let text = run(
        "expr_actual",
        r#"module dff (input clk, input [31:0] din, output reg [31:0] q);
  initial q = 0;
  always @(posedge clk) q <= din;
endmodule
module tb_top;
  reg clk = 0; always #5 clk = ~clk;
  reg [31:0] src = 32'h11111111;
  wire [31:0] din = src;
  wire [31:0] q0;
  dff u0 (.clk(clk), .din(din), .q(q0));
  integer cyc = 0;
  always @(posedge clk) begin
    src <= src + 32'h01010101;
    cyc <= cyc + 1;
    if (cyc == 3) begin
      if (q0 !== 32'hx && q0 === 32'h13131313) $display("TEST_PASS");
      else $display("TEST_FAIL q0=%h", q0);
      $finish;
    end
  end
endmodule
"#,
    );
    assert!(
        text.contains("TEST_PASS"),
        "same-name identity actual:\n{text}"
    );
}

/// §23.3.3: an EXPRESSION actual over a parent net named like the child's
/// FORMAL (`.d(d ^ 1)` where the parent also has a `d`). Reference-verified:
/// the substituted actual keeps PARENT-scope resolution inside the child
/// (child sees `src^1`, never double-applies the map), and the parent's own
/// processes read the parent's `d` even right after the child's block ran
/// (the resolve hint must not leak across edge blocks).
#[test]
fn expression_actual_name_collision_resolves_parent_scope() {
    let text = run(
        "expr_collide",
        r#"module ff (input clk, input [31:0] d, output reg [31:0] q);
  initial q = 0;
  always @(posedge clk) begin
    q <= d;
    $display("CHILD d=%h", d);
  end
endmodule
module tb_top;
  reg clk = 0; always #5 clk = ~clk;
  reg [31:0] src = 32'h11111111;
  wire [31:0] d = src;
  wire [31:0] q1;
  ff u1 (.clk(clk), .d(d ^ 1), .q(q1));
  integer cyc = 0;
  always @(posedge clk) begin
    src <= src + 32'h01010101;
    cyc <= cyc + 1;
    if (cyc >= 2 && cyc <= 3) $display("TOP src=%h d=%h u1d=%h q1=%h", src, d, u1.d, q1);
    if (cyc == 4) $finish;
  end
endmodule
"#,
    );
    // Child reads the PARENT d (src) through the substituted actual: src^1.
    assert!(
        text.contains("CHILD d=11111110"),
        "child first sample:\n{text}"
    );
    assert!(
        text.contains("CHILD d=12121213"),
        "child second sample:\n{text}"
    );
    // Parent reads its own d (== src), u1.d == src^1, q1 == previous u1.d.
    assert!(
        text.contains("TOP src=13131313 d=13131313 u1d=13131312 q1=12121213"),
        "parent cycle 2:\n{text}"
    );
    assert!(
        text.contains("TOP src=14141414 d=14141414 u1d=14141415 q1=13131312"),
        "parent cycle 3:\n{text}"
    );
}

/// A TB-level hierarchical assign whose RHS name COLLIDES with a net inside
/// the target scope (reference-verified): `assign dut.core.rst_in = grst_l`
/// must read the TB's `grst_l`, not the DUT-internal dead net of the same
/// name. The LHS resolution ratchets the resolve hint to the TARGET's
/// parent scope, under which the bare RHS bound to the deep twin and
/// forwarded x forever — while a sibling clock assign (no deep twin) worked,
/// hiding the bug behind that asymmetry.
#[test]
fn hier_assign_rhs_survives_target_scope_name_collision() {
    let text = run(
        "hier_shadow",
        r#"module m_core (
  input logic i_clk,
  input logic grst_l_deep,
  output logic [7:0] o_q
);
  logic grst_l;   // DUT-internal same-named net, undriven (x forever)
  logic unused;
  assign unused = grst_l;
  always_ff @(posedge i_clk or negedge grst_l_deep)
    if (!grst_l_deep) o_q <= '0;
    else o_q <= o_q + 8'h1;
endmodule
module m_dut (
  input logic i_clk,
  output logic [7:0] o_q
);
  m_core u_core (.i_clk(i_clk), .grst_l_deep(), .o_q(o_q));
endmodule
module tb_top;
  logic tb_clk = 0;
  logic grst_l;
  logic [7:0] q;
  m_dut uut (.i_clk(tb_clk), .o_q(q));
  assign tb_top.uut.u_core.grst_l_deep = grst_l;
  always #5 tb_clk = ~tb_clk;
  int bad = 0;
  initial begin
    grst_l = 1'bx;
    #2 grst_l = 0;
    @(posedge tb_clk); #1;
    if (q !== 8'h00) bad++;
    grst_l = 1;
    @(posedge tb_clk); #1;
    @(posedge tb_clk); #1;
    if (q !== 8'h02) bad++;
    if (bad == 0) $display("TEST_PASS"); else $display("TEST_FAIL n=%0d", bad);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("TEST_PASS"),
        "shadowed hier-assign rhs:\n{text}"
    );
}

/// Two reference-verified mechanisms from one TB (§23.10.1 + §10.3.3):
/// (a) `assign #N` INSIDE an inlined module keeps its delay — the pending
///     inline path used to drop it, so a `#3` clock echo in the DUT tracked
///     undelayed; (b) a bound module's upward reference (`u_core.clk` from a
///     `bind` harness) resolves at entry-build time — unresolved, the echo
///     copy evaluated on unrelated settle passes (right value, wrong time).
/// The XOR err detector catches either failure as a >=1ns misalignment.
#[test]
fn bound_harness_sees_delayed_clock_echo_aligned() {
    let text = run(
        "bind_echo",
        r#"module m_core (input logic i_clk, output logic [7:0] o_q);
  logic clk;
  assign #3 clk = i_clk;
  always_ff @(posedge i_clk) o_q <= o_q + 8'h1;
endmodule
module m_wrap (input logic i_clk, output logic [7:0] o_q);
  m_core u_core (.i_clk(i_clk), .o_q(o_q));
endmodule
module m_watch ();
  wire clk;
  assign clk = u_core.clk;
endmodule
bind m_wrap m_watch v_watch ();
module tb_top;
  logic tb_clk = 0;
  wire tb_clk_d;
  logic [7:0] q;
  assign #3 tb_clk_d = tb_clk;
  m_wrap uut (.i_clk(), .o_q(q));
  assign tb_top.uut.u_core.i_clk = tb_clk;
  logic err;
  assign err = tb_clk_d ^ uut.v_watch.clk;
  int bad = 0;
  always @(posedge err) begin
    #1;
    if (err !== 0) bad++;
  end
  always #5 tb_clk = ~tb_clk;
  initial begin
    #200;
    if (bad == 0) $display("TEST_PASS");
    else $display("TEST_FAIL n=%0d", bad);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("TEST_PASS"),
        "delayed echo through bind:\n{text}"
    );
}
