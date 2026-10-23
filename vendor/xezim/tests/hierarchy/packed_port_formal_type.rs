//! IEEE 1800-2023 §7.4.1 / §23.3.3: a port's declared type belongs to the
//! FORMAL. The connection is an assignment, not a rename, so neither the
//! port's width nor its packed element stride may be inherited from the
//! actual expression bound to it.
//!
//! An input port used to be substituted with its actual throughout the
//! inlined child body, which discarded the formal's type. `input logic
//! [1:0][3:0] p` driven by a flat `logic [7:0]` turned `p[i]` into a one-BIT
//! select ($bits(p[0]) == 1), so an indexing loop read bits 0/1 where it
//! should read nibbles [3:0]/[7:4]. A differently-shaped packed actual handed
//! over ITS stride (2 for `[3:0][1:0]`); a narrower/wider actual even changed
//! $bits(p) inside the child.
//!
//! Expected values here were cross-checked against a reference simulator —
//! see `tests/sv_ref/` for the standalone form of these same cases.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z, expected a defined value", n))
}

/// One probe module, instantiated once per shape of driving expression. All
/// eight drivers carry the same 8 bits (8'h21 = 8'b0010_0001), so every
/// instance must report the identical element view.
const SRC_ELEM: &str = r#"
module elem_probe (
   input  logic [1:0][3:0] pa,
   output int              bw,   // $bits(pa)
   output int              ew,   // $bits(pa[0])
   output logic [3:0]      e0,
   output logic [3:0]      e1
);
   assign bw = $bits(pa);
   assign ew = $bits(pa[0]);
   assign e0 = pa[0];
   assign e1 = pa[1];
endmodule

module tb;
   logic [1:0][3:0]      d_shaped;
   logic [7:0]           d_flat;
   wire  [7:0]           d_wire;
   logic [3:0]           d_hi, d_lo;
   logic [15:0]          d_wide;
   logic [3:0][1:0]      d_other;
   logic [3:0][1:0][3:0] d_big;
   logic [1:0][3:0]      d_unpk [0:3];

   assign d_wire = d_flat;

   int bw_sh, ew_sh; logic [3:0] e0_sh, e1_sh;
   int bw_fl, ew_fl; logic [3:0] e0_fl, e1_fl;
   int bw_wr, ew_wr; logic [3:0] e0_wr, e1_wr;
   int bw_cc, ew_cc; logic [3:0] e0_cc, e1_cc;
   int bw_ps, ew_ps; logic [3:0] e0_ps, e1_ps;
   int bw_ot, ew_ot; logic [3:0] e0_ot, e1_ot;
   int bw_sl, ew_sl; logic [3:0] e0_sl, e1_sl;
   int bw_up, ew_up; logic [3:0] e0_up, e1_up;

   elem_probe u_sh (.pa(d_shaped),      .bw(bw_sh), .ew(ew_sh), .e0(e0_sh), .e1(e1_sh));
   elem_probe u_fl (.pa(d_flat),        .bw(bw_fl), .ew(ew_fl), .e0(e0_fl), .e1(e1_fl));
   elem_probe u_wr (.pa(d_wire),        .bw(bw_wr), .ew(ew_wr), .e0(e0_wr), .e1(e1_wr));
   elem_probe u_cc (.pa({d_hi, d_lo}),  .bw(bw_cc), .ew(ew_cc), .e0(e0_cc), .e1(e1_cc));
   elem_probe u_ps (.pa(d_wide[7:0]),   .bw(bw_ps), .ew(ew_ps), .e0(e0_ps), .e1(e1_ps));
   elem_probe u_ot (.pa(d_other),       .bw(bw_ot), .ew(ew_ot), .e0(e0_ot), .e1(e1_ot));
   elem_probe u_sl (.pa(d_big[2]),      .bw(bw_sl), .ew(ew_sl), .e0(e0_sl), .e1(e1_sl));
   elem_probe u_up (.pa(d_unpk[1]),     .bw(bw_up), .ew(ew_up), .e0(e0_up), .e1(e1_up));

   initial begin
      d_shaped = 8'h21;
      d_flat   = 8'h21;
      d_hi     = 4'h2;
      d_lo     = 4'h1;
      d_wide   = 16'hBA21;
      d_other  = 8'h21;
      d_big[2] = 8'h21;
      d_unpk[1] = 8'h21;
      #1;
   end
endmodule
"#;

#[test]
fn packed_multi_d_port_element_stride_comes_from_the_formal() {
    let sim = simulate(SRC_ELEM, 50).expect("simulate failed");
    // (suffix, human description of the DRIVER's shape)
    let cases = [
        ("sh", "packed [1:0][3:0] (matches the formal)"),
        ("fl", "flat logic [7:0]"),
        ("wr", "flat wire [7:0]"),
        ("cc", "concatenation {hi,lo}"),
        ("ps", "part-select wide[7:0]"),
        ("ot", "differently-shaped packed [3:0][1:0]"),
        ("sl", "slice of a bigger packed array"),
        ("up", "element of an unpacked array"),
    ];
    for (sfx, what) in cases {
        assert_eq!(
            u(&sim, &format!("bw_{sfx}")),
            8,
            "$bits(pa) with driver = {what}: the formal is 8 bits"
        );
        assert_eq!(
            u(&sim, &format!("ew_{sfx}")),
            4,
            "$bits(pa[0]) with driver = {what}: the formal's element is 4 bits, \
             not the driver's stride"
        );
        assert_eq!(
            u(&sim, &format!("e0_{sfx}")),
            0x1,
            "pa[0] with driver = {what}: nibble [3:0] of 8'h21"
        );
        assert_eq!(
            u(&sim, &format!("e1_{sfx}")),
            0x2,
            "pa[1] with driver = {what}: nibble [7:4] of 8'h21 — a one-bit \
             select would read bit 1 and yield 0"
        );
    }
}

/// The reported failure shape: a lane loop indexing a packed multi-D input
/// port with a loop variable. Both instances are the same module and differ
/// only in how the parent declares the net feeding the port.
const SRC_LOOP: &str = r#"
module lane_decoder #(parameter int NSLOT = 4) (
   input  logic [1:0]             sel_vld,
   input  logic [1:0]             sel_grp,
   input  logic [1:0] [NSLOT-1:0] sel_dec,
   output logic       [NSLOT-1:0] slot_en,
   output logic                   any_en,
   output logic [1:0] [NSLOT-1:0] slot_dec_o
);
   logic [1:0] [NSLOT-1:0] slot_dec;
   assign slot_dec_o = slot_dec;
   always_comb begin
      slot_en = 'd0;
      any_en  = 1'b0;
      for (int j = 0; j < 2; j++) begin
         slot_dec[j] = (sel_vld[j] & sel_grp[j]) ? sel_dec[j] : 0;
         slot_en |= slot_dec[j];
      end
      any_en = |slot_en;
   end
endmodule

module tb;
   logic [1:0]      sel_vld, sel_grp;
   logic [7:0]      dec_flat;      // parent net is FLAT
   logic [1:0][3:0] dec_shaped;    // parent net matches the port shape

   logic [3:0]      en_flat,  en_shaped;
   logic            any_flat, any_shaped;
   logic [1:0][3:0] dcd_flat, dcd_shaped;

   lane_decoder #(.NSLOT(4)) u_flat (
      .sel_vld(sel_vld), .sel_grp(sel_grp), .sel_dec(dec_flat),
      .slot_en(en_flat), .any_en(any_flat), .slot_dec_o(dcd_flat));

   lane_decoder #(.NSLOT(4)) u_shaped (
      .sel_vld(sel_vld), .sel_grp(sel_grp), .sel_dec(dec_shaped),
      .slot_en(en_shaped), .any_en(any_shaped), .slot_dec_o(dcd_shaped));

   initial begin
      sel_vld = 2'b00; sel_grp = 2'b00; dec_flat = 8'h00; dec_shaped = 8'h00;
      #10;
      sel_vld = 2'b11; sel_grp = 2'b11; dec_flat = 8'h21; dec_shaped = 8'h21;
      #10;
   end
endmodule
"#;

#[test]
fn lane_loop_over_packed_multi_d_input_port_decodes_both_lanes() {
    let sim = simulate(SRC_LOOP, 100).expect("simulate failed");
    for (sfx, what) in [("flat", "flat parent net"), ("shaped", "shaped parent net")] {
        assert_eq!(
            u(&sim, &format!("dcd_{sfx}")),
            0x21,
            "{what}: both lanes decode (8'h21), not just lane 0 (8'h01)"
        );
        assert_eq!(
            u(&sim, &format!("en_{sfx}")),
            0x3,
            "{what}: slot_en is the OR of both lanes"
        );
        // any_en stays 1 in the buggy case too — that is exactly why the
        // dropped lane went unnoticed downstream.
        assert_eq!(u(&sim, &format!("any_{sfx}")), 1, "{what}: any_en");
    }
}

/// §23.3.3: a width-mismatched connection truncates or extends at the
/// boundary; the formal keeps its declared size.
const SRC_WIDTH: &str = r#"
module width_probe (
   input  logic [1:0][3:0] pa,
   output int              bw,
   output int              ew,
   output logic [7:0]      val
);
   assign bw  = $bits(pa);
   assign ew  = $bits(pa[0]);
   assign val = pa;
endmodule

module tb;
   logic [3:0]  d_narrow;
   logic [15:0] d_wider;

   int bw_n, ew_n; logic [7:0] v_n;
   int bw_w, ew_w; logic [7:0] v_w;
   int bw_u, ew_u; logic [7:0] v_u;

   width_probe u_narrow (.pa(d_narrow), .bw(bw_n), .ew(ew_n), .val(v_n));
   width_probe u_wider  (.pa(d_wider),  .bw(bw_w), .ew(ew_w), .val(v_w));
   width_probe u_unconn (              .bw(bw_u), .ew(ew_u), .val(v_u));

   initial begin
      d_narrow = 4'h5;
      d_wider  = 16'hBA21;
      #1;
   end
endmodule
"#;

#[test]
fn port_width_comes_from_the_formal_not_the_connection() {
    let sim = simulate(SRC_WIDTH, 50).expect("simulate failed");
    for (sfx, what) in [
        ("n", "narrower actual (logic [3:0])"),
        ("w", "wider actual (logic [15:0])"),
        ("u", "unconnected"),
    ] {
        assert_eq!(u(&sim, &format!("bw_{sfx}")), 8, "$bits(pa) with {what}");
        assert_eq!(u(&sim, &format!("ew_{sfx}")), 4, "$bits(pa[0]) with {what}");
    }
    assert_eq!(
        u(&sim, "v_n"),
        0x05,
        "narrow actual zero-extends to the formal"
    );
    assert_eq!(
        u(&sim, "v_w"),
        0x21,
        "wide actual truncates to the formal's low 8"
    );
}

/// Guard for the leniency this fix had to preserve: writing an INPUT port
/// from inside the module. Illegal per §23.3.3, but xezim runs such designs,
/// and it only works because the port is substituted away so the write lands
/// on the parent net. A port the body drives must therefore KEEP its
/// substitution even when its packed shape would otherwise argue against it.
const SRC_DRIVEN_INPUT: &str = r#"
typedef logic [7:0] byte_t;
module duT (
  input  clk,
  byte_t [1:0] typed_port,        // no direction keyword -> inherits `input`
  input  plain
);
  assign typed_port = 16'hA55A;
endmodule
module tb;
  logic clk = 0, p = 1;
  wire [15:0] tp;
  int seen;
  duT u (.clk(clk), .typed_port(tp), .plain(p));
  initial begin #1; seen = tp; end
endmodule
"#;

#[test]
fn body_driven_input_port_still_reaches_the_parent_net() {
    let sim = simulate(SRC_DRIVEN_INPUT, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "seen"),
        0xA55A,
        "a packed multi-D input port written by the body must keep driving \
         the parent net, not fight its own connection assign"
    );
}

/// A port whose packed multi-D type arrives via a TYPEDEF. Both helpers that
/// register element metadata bail immediately on a `TypeReference` carrying no
/// dimensions of its own, so such a port registered nothing and `p[i]`
/// degraded to a one-BIT select — the exact failure the inline
/// `logic [1:0][3:0]` form was fixed for, reached by a different spelling of
/// the same type. The inline form working is what made this look
/// type-specific rather than a missing typedef resolution.
const SRC_TYPEDEF_PORT: &str = r#"
typedef logic [1:0][3:0] pair_t;

module tsink (input pair_t tp, output logic [3:0] t0, output logic [3:0] t1, output int tew);
   assign t0  = tp[0];
   assign t1  = tp[1];
   assign tew = $bits(tp[0]);
endmodule

module tb;
   logic [7:0] flat;          // deliberately NOT declared with the packed dims
   logic [3:0] t0, t1;
   int         tew;
   tsink u (.tp(flat), .t0(t0), .t1(t1), .tew(tew));
   initial begin
      flat = 8'h2D;           // asymmetric: elem[0]=D, elem[1]=2
      #1;
   end
endmodule
"#;

#[test]
fn typedefd_packed_multi_d_port_keeps_its_element_stride() {
    let sim = simulate(SRC_TYPEDEF_PORT, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "tew"),
        4,
        "$bits(tp[0]) through a typedef'd packed multi-D port"
    );
    assert_eq!(u(&sim, "t0"), 0xD, "tp[0] must be the low nibble of 8'h2D");
    assert_eq!(
        u(&sim, "t1"),
        0x2,
        "tp[1] must be the high nibble; a one-bit select would read bit 1 and give 0"
    );
}
