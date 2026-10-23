//! §4b: array-ELEMENT correctness trio — reference-validated (d1 battery).
//!
//! 1. §10.6.2 `force mem[i]` arms the ELEMENT's signal id (previously the
//!    Index lvalue degraded to a plain write and the next assignment won).
//! 2. §6.11.1 a 2-state element (`bit [7:0] m[0:3]`) drops X/Z on write.
//! 3. §7.4.6 negative-lo arrays (`m[-2:1]`): SIGNED index on both the read
//!    and write paths (to_u64 turned -2 into 4294967294 and the element
//!    vanished).

use xezim::simulate;

fn line(sim: &xezim::compiler::Simulator, tag: &str) -> String {
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .find(|m| m.starts_with(tag))
        .unwrap_or_else(|| panic!("no output line starting with {}", tag))
}

/// Reference: cc 34 ff 00 -3 11 22 xx, then after release mem0=02.
#[test]
fn element_force_two_state_and_negative_lo() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  reg [7:0]        mem  [0:3];
  bit  [7:0]       bmem [0:3];
  reg signed [7:0] smem [0:3];
  reg [7:0]        nmem [-2:1];
  initial begin
    reg [7:0] xv;
    mem[0] = 8'h5A;
    mem[3] = 8'hFF;
    mem[2] = 16'h1234;
    xv = 8'hxx;
    bmem[1] = xv;
    smem[0] = -3;
    nmem[-2] = 8'h11; nmem[1] = 8'h22;
    mem[7] = 8'h77;
    force mem[0] = 8'hCC;
    mem[0] = 8'h01;
    #2 $display("T|%h %h %h %h %0d %h %h %h", mem[0], mem[2], mem[3], bmem[1], smem[0], nmem[-2], nmem[1], mem[1]);
    release mem[0];
    mem[0] = 8'h02;
    #1 $display("T|after mem0=%h", mem[0]);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        line(&sim, "T|c"),
        "T|cc 34 ff 00 -3 11 22 xx",
        "force holds / 2-state fits / negative-lo lives"
    );
    assert_eq!(
        line(&sim, "T|after"),
        "T|after mem0=02",
        "release restores writability"
    );
}
