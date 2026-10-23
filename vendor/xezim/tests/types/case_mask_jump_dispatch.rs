//! §12.5.1 dense casez/casex with constant wildcard patterns compiles to
//! a two-level dispatch: a jump table over a window of always-defined
//! selector bits, then a short residual chain per bucket, with the full
//! sequential chain kept as the path for selectors carrying x/z in the
//! window (a wildcard selector may match any bucket). Covers: bucket hits,
//! multi-bucket misses to default, an all-z selector (first ??00 arm wins
//! via the chain), z in the dispatch window, and x high bits (not wild in
//! casez) falling to default. Expectations verified against the reference
//! simulator.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
module top;
  logic [15:0] insn;
  logic [4:0] op;
  logic clk = 0;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    unique casez (insn)
      16'b0000_0000_0???_??00: op <= 5'd1;   // bucket by [1:0]+high
      16'b010?_????_????_??00: op <= 5'd2;
      16'b100?_????_????_??00: op <= 5'd3;
      16'b0000_????_????_??01: op <= 5'd4;
      16'b001?_????_????_??01: op <= 5'd5;
      16'b010?_????_????_??01: op <= 5'd6;
      16'b100?_????_????_??10: op <= 5'd7;
      16'b110?_????_????_??10: op <= 5'd8;
      16'b1111_1111_????_??11: op <= 5'd9;
      default:                 op <= 5'd31;
    endcase
  end
  task check(input logic [15:0] i);
    insn = i; @(negedge clk); $display("NOTE: %b %0d", i, op);
  endtask
  initial begin
    check(16'b0000_0000_0110_1000); // arm1
    check(16'b0100_1111_0000_0000); // arm2
    check(16'b1001_1111_1111_1100); // arm3
    check(16'b0000_1010_1010_1001); // arm4
    check(16'b0011_0000_0000_0001); // arm5
    check(16'b0101_0000_0000_0001); // arm6
    check(16'b1000_0000_0000_0010); // arm7
    check(16'b1101_1111_1111_1110); // arm8
    check(16'b1111_1111_0101_0111); // arm9
    check(16'b1111_0000_0000_0011); // default
    check(16'b0110_0000_0000_0000); // default (no arm for 011...00)
    check(16'bzzzz_zzzz_zzzz_zz00); // xz path: z everywhere -> first ??00 arm = arm1
    check(16'b0100_1111_0000_00zz); // xz in window -> chain; z wild -> arm2 first match
    check(16'bxxxx_0000_0000_0000); // casez: x != wild; high x kills 1,2,3 -> default
    $finish;
  end
endmodule

"#;

#[test]
fn wildcard_case_two_level_dispatch_matches_chain() {
    assert_eq!(
        notes(SRC),
        [
            "NOTE: 0000000001101000 1",
            "NOTE: 0100111100000000 2",
            "NOTE: 1001111111111100 3",
            "NOTE: 0000101010101001 4",
            "NOTE: 0011000000000001 5",
            "NOTE: 0101000000000001 6",
            "NOTE: 1000000000000010 7",
            "NOTE: 1101111111111110 8",
            "NOTE: 1111111101010111 9",
            "NOTE: 1111000000000011 31",
            "NOTE: 0110000000000000 31",
            "NOTE: zzzzzzzzzzzzzz00 1",
            "NOTE: 01001111000000zz 2",
            "NOTE: xxxx000000000000 31",
        ]
    );
}
