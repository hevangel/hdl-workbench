//! IEEE 1800 §11.5.1 on the COMPILED paths: a constant part-select write
//! whose low bound is a negative label (`x[4:-1] = v`) keeps the in-range
//! bits. The bytecode compiler folded the bounds in u32, so `-1` became
//! 4294967295, the range looked ascending, the width came out as -4
//! (clamped to the cap) and the write was dropped in clocked blocks and in
//! process FSMs alike; the non-blocking arm also stored a fifth bit into a
//! 4-bit signal because it never clamped the high bound.
use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn clocked_block_negative_low_bound_keeps_in_range_bits() {
    let src = r#"
`timescale 1ns/1ns
module top;
  reg clk = 0;
  bit [3:0] x = 0, y = 0, z = 0;
  int b_both = 0, b_low = 0, n_both = 0;
  always @(posedge clk) begin
    x = 0; x[4:-1] = 6'b101010; b_both = x;
    y = 0; y[0:-1] = 2'b10;     b_low  = y;
    z = 0; z[4:-1] <= 6'b101010;
  end
  always @(negedge clk) n_both = z;
  initial begin
    #5 clk = 1; #5 clk = 0; #5 clk = 1; #5 clk = 0;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "b_both"),
        0b0101,
        "blocking, both ends out of range"
    );
    assert_eq!(u(&sim, "b_low"), 0b0001, "blocking, low end out");
    assert_eq!(
        u(&sim, "n_both"),
        0b0101,
        "NBA, both ends out of range: no fifth bit"
    );
    assert_eq!(u(&sim, "z"), 0b0101);
}

#[test]
fn stimulus_initial_block_negative_low_bound() {
    // The plain-stimulus initial block runs as a process FSM by default.
    let src = r#"
`timescale 1ns/1ns
module top;
  reg clk = 0;
  always #5 clk = ~clk;
  bit [3:0] x = 0;
  int b_both = 0, n_both = 0;
  initial begin
    @(posedge clk);
    x = 0; x[4:-1] = 6'b101010; b_both = x;
    @(posedge clk);
    x = 0; x[4:-1] <= 6'b101010;
    @(posedge clk);
    n_both = x;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "b_both"), 0b0101);
    assert_eq!(u(&sim, "n_both"), 0b0101);
}
