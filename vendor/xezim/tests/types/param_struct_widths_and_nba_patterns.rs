//! Four fixes from two user testbenches, all reference-validated:
//!
//! 1. Instance elaboration: a local typedef's field widths can use body
//!    LOCALPARAMS and a localparam can use `$bits(<local typedef>)` —
//!    registering typedefs once against header params froze the wrong width
//!    ($bits read 15 instead of 53). Now iterated to a fixed point.
//! 2. §20.6.2: `$bits(<expression>)` over concat/replication operands in
//!    constant context (port ranges) computes the structural width.
//! 3. Bit-select WRITE into a packed-struct member (`t.f2[3] = 1'b1`)
//!    vanished (the range form worked); now splices through the container.
//! 4. NBA assignment pattern onto a packed ARRAY (`cdts <= '{default: v}` on
//!    `logic [1:0][9:0]`) filled only element 0.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Fixes 1 + 2 + 3 together, in an instance (the failing configuration).
#[test]
fn localparam_struct_widths_in_instance() {
    let src = r#"
module m #(parameter int A = 8) (
  output logic [ ($bits({{A{1'b0}}, {(A*4){1'b0}}})-1):0 ] po
);
  localparam int LP1 = A * 4;
  typedef struct packed {
    logic [A-1:0]   f1;
    logic [LP1-1:0] f2;
  } st_t;
  localparam int SB = $bits(st_t);
  st_t s;
  logic [31:0] sb_probe;
  initial begin
    sb_probe = SB;
    s = '0;
    s.f1 = {A{1'b1}};
    for (int i = 0; i < LP1; i++) s.f2[i] = (i % 2) ? 1'b1 : 1'b0;
  end
  assign po = {s.f1, s.f2};
endmodule
module tb;
  wire [39:0] w;
  m #(.A(8)) dut(.po(w));
  int sb, portw_ok, f2_ok;
  initial begin
    #1;
    sb = dut.sb_probe;
    portw_ok = ($bits(dut.po) == 40);
    f2_ok = (w[31:0] == 32'hAAAA_AAAA);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "sb"),
        40,
        "$bits(local typedef) sees body localparams"
    );
    assert_eq!(
        u(&sim, "portw_ok"),
        1,
        "$bits(concat-of-replications) port range"
    );
    assert_eq!(
        u(&sim, "f2_ok"),
        1,
        "bit-writes into localparam-sized member land"
    );
}

/// Fix 3 in isolation: bit-select member writes, top level.
#[test]
fn packed_struct_member_bit_write() {
    let src = r#"
module tb;
  typedef struct packed {
    logic [7:0]  f1;
    logic [31:0] f2;
  } tst_t;
  tst_t t;
  int b3, b30, f1b, ps_ok;
  initial begin
    t = '0;
    t.f2[3] = 1'b1;
    t.f2[30] = 1'b1;
    t.f1[2] = 1'b1;
    t.f2[7:4] = 4'hF;
    b3 = t.f2[3]; b30 = t.f2[30]; f1b = t.f1[2];
    ps_ok = (t.f2[7:4] == 4'hF);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "b3"), 1, "member bit write low");
    assert_eq!(u(&sim, "b30"), 1, "member bit write high");
    assert_eq!(u(&sim, "f1b"), 1, "non-tail member bit write");
    assert_eq!(u(&sim, "ps_ok"), 1, "range form still works");
}

/// Fix 4: NBA '{default:} onto a packed 2-D array fills every element.
#[test]
fn nba_default_pattern_on_packed_array() {
    let src = r#"
module tb;
  logic clk = 0, rst_l = 0;
  logic [1:0][9:0] cdts;
  logic [9:0] dv;
  assign dv = 10'd36;
  always #5 clk = ~clk;
  always_ff @(posedge clk) begin
    if (!rst_l) cdts <= '{default: dv};
    else begin
      for (int i = 0; i < 2; i++) cdts[i] <= cdts[i] - 1'b1;
    end
  end
  int r0, r1, d0, d1;
  initial begin
    #12; r0 = cdts[0]; r1 = cdts[1];
    rst_l = 1;
    #10; d0 = cdts[0]; d1 = cdts[1];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 36, "NBA default pattern element 0");
    assert_eq!(u(&sim, "r1"), 36, "NBA default pattern element 1");
    assert_eq!(u(&sim, "d0"), 35, "per-element NBA decrement");
    assert_eq!(u(&sim, "d1"), 35, "per-element NBA decrement lane 1");
}
