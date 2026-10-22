//! Issue #118 — a non-blocking write to an unpacked array whose dimensions
//! come from MODULE PARAMETERS (`logic [DW-1:0] mem [NR];`) was silently
//! dropped (the element read back x/0), while the identical array with
//! literal dimensions worked. Root cause was the element-select width
//! resolution fixed in 960f16d (`expr_max_width` returned 1 for an
//! unpacked-array element select once the block compiled); these tests pin
//! the WRITE-side shapes from the report, which shipped broken in a release
//! and had no direct guard (960f16d's own test covers the compare side).

use xezim::simulate;

fn hex32(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n));
    v.to_u64().unwrap_or_else(|| panic!("{} has x/z bits", n))
}

/// NBA to `store[sel]` inside a parameterized register file: the write must
/// land whether the dimensions are parameters or literals.
#[test]
fn param_dimensioned_array_nba_lands() {
    let src = r#"
module cfg_regfile #(parameter int WIDTH = 32, parameter int DEPTH = 16) (
    input logic clk, input logic wen,
    input logic [3:0] sel, input logic [WIDTH-1:0] din,
    output logic [WIDTH-1:0] dout3
);
  logic [WIDTH-1:0] store [DEPTH];
  always_ff @(posedge clk)
    if (wen) store[sel] <= din;
  assign dout3 = store[3];
endmodule
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic wen = 0; logic [3:0] sel = 4'd3; logic [31:0] din = 32'hCAFEBABE;
  logic [31:0] dout3;
  logic [31:0] readback;
  cfg_regfile u0(.clk, .wen, .sel, .din, .dout3);
  initial begin
    @(posedge clk);
    wen <= 1'b1; @(posedge clk);
    wen <= 1'b0; @(posedge clk); @(posedge clk);
    readback = dout3;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        hex32(&sim, "readback"),
        0xCAFE_BABE,
        "NBA to a parameter-dimensioned array element must commit"
    );
}

/// NBA with an INDEXED PART-SELECT LHS on an array element in a loop
/// (`store[k][b*8+:8] <= din[b*8+:8];`) — the second dropped-write shape
/// from the same report.
#[test]
fn array_elem_indexed_partselect_nba_lands() {
    let src = r#"
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [31:0] store [4];
  logic [31:0] din = 32'hA1B2C3D4;
  logic [31:0] readback;
  int cyc = 0;
  always @(posedge clk) begin
    cyc <= cyc + 1;
    if (cyc == 1) begin
      for (int b = 0; b < 4; b++)
        store[2][b*8+:8] <= din[b*8+:8];
    end
    if (cyc == 3)
      readback = store[2];
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        hex32(&sim, "readback"),
        0xA1B2_C3D4,
        "byte-lane NBA splices into an array element must all commit"
    );
}

/// Mask built by replicating a condition bit into an indexed part-select in
/// a loop (`m[b*8+:8] = {8{s[b]}};`) — reported alongside the above; guard
/// it so the working behavior stays working.
#[test]
fn replicated_bit_partselect_mask_build() {
    let src = r#"
module tb;
  logic [3:0] lane_en = 4'b1010;
  logic [31:0] mask;
  logic [31:0] readback;
  initial begin
    mask = '0;
    for (int b = 0; b < 4; b++)
      mask[b*8+:8] = {8{lane_en[b]}};
    readback = mask;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        hex32(&sim, "readback"),
        0xFF00_FF00,
        "replication into an indexed part-select must fill the full lane"
    );
}
