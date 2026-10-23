//! Flop skip (event-edge, write-armed) for blocks that read or write an
//! unpacked array through a DYNAMIC index. Such blocks used to be permanently
//! non-gateable; now every element of the array is an arm-only input, so a
//! skip is only possible while no element (and no scalar input) was written.
//! Each case below would print a stale value if an element write failed to
//! re-arm the block.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn element_write_rearms_read_port_flops() {
    let src = r#"
module tb;
  reg clk = 0;
  always #5 clk = ~clk;
  reg [7:0] mem [0:15];
  reg [3:0] raddr = 4'd3;
  reg [7:0] q, q2;
  // fused read port (NbaAssignArrayRead) and a computed read (LoadArrayElem)
  always @(posedge clk) q <= mem[raddr];
  always @(posedge clk) q2 <= mem[raddr] + 8'd1;
  reg [7:0] r_q, r_q2, s_q, s_q2;
  integer i;
  initial begin
    for (i = 0; i < 16; i = i + 1) mem[i] = i;
    mem[3] = 8'h11;
    repeat (6) @(posedge clk);          // inputs stable: the flops skip
    #1 s_q = q; s_q2 = q2;
    mem[3] = 8'h22;                     // element write, raddr unchanged
    @(posedge clk); #1 r_q = q; r_q2 = q2;
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    assert_eq!(u(&sim, "s_q"), 0x11);
    assert_eq!(u(&sim, "s_q2"), 0x12);
    assert_eq!(
        u(&sim, "r_q"),
        0x22,
        "read port did not re-execute after mem[3] changed"
    );
    assert_eq!(
        u(&sim, "r_q2"),
        0x23,
        "computed read did not re-execute after mem[3] changed"
    );
}

#[test]
fn foreign_element_write_rearms_write_port_flop() {
    let src = r#"
module tb;
  reg clk = 0;
  always #5 clk = ~clk;
  reg [7:0] mem2 [0:3];
  reg [1:0] waddr = 2'd1;
  reg [7:0] wdata = 8'h5a;
  // write port with stable inputs: re-writes mem2[1] every posedge
  always @(posedge clk) mem2[waddr] <= wdata;
  reg [7:0] before_edge, after_edge;
  initial begin
    repeat (6) @(posedge clk);
    #1 mem2[1] = 8'h00;                  // clobbered by another process
    before_edge = mem2[1];
    @(posedge clk); #1 after_edge = mem2[1];
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    assert_eq!(u(&sim, "before_edge"), 0x00);
    assert_eq!(
        u(&sim, "after_edge"),
        0x5a,
        "write port did not re-execute after mem2[1] was clobbered"
    );
}
