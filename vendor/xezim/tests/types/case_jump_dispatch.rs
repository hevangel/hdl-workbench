//! §12.5 dense plain `case` compiles to a computed-goto jump table.
//!
//! Gates that keep the table exactly equivalent to the `===` chain it
//! replaces: unsigned selector <=64 bits (raw numeric dispatch equals `===`
//! only under zero-extension), constant x/z-free patterns < 4096, >=8
//! pattern values. Covered here: multi-pattern arms sharing one body,
//! duplicate pattern (first arm wins), a hole and an out-of-range selector
//! (both -> default), an x selector (-> default, since fully defined
//! patterns can never case-match x), and a default declared mid-case.

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
  logic [3:0] sel;
  logic [7:0] a, b;
  logic clk = 0;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    case (sel)
      4'd0: begin a <= 8'h10; b <= 8'h01; end
      4'd1: begin a <= 8'h11; b <= 8'h02; end
      4'd2, 4'd3: begin a <= 8'h12; b <= 8'h03; end
      4'd4: a <= 8'h14;
      default: begin a <= 8'hdd; b <= 8'hee; end
      4'd6: a <= 8'h16;
      4'd7: a <= 8'h17;
      4'd8: a <= 8'h18;
      4'd9: a <= 8'h19;
      4'd2: a <= 8'hbb; // duplicate pattern: first arm must win
    endcase
  end
  initial begin
    sel = 0;    @(negedge clk) $display("NOTE: %h %h", a, b);
    sel = 2;    @(negedge clk) $display("NOTE: %h %h", a, b);
    sel = 3;    @(negedge clk) $display("NOTE: %h %h", a, b);
    sel = 5;    @(negedge clk) $display("NOTE: %h %h", a, b);
    sel = 9;    @(negedge clk) $display("NOTE: %h %h", a, b);
    sel = 4'hx; @(negedge clk) $display("NOTE: %h %h", a, b);
    sel = 15;   @(negedge clk) $display("NOTE: %h %h", a, b);
    $finish;
  end
endmodule
"#;

#[test]
fn dense_case_dispatch_matches_chain_semantics() {
    assert_eq!(
        notes(SRC),
        [
            "NOTE: 10 01", // exact hit
            "NOTE: 12 03", // multi-pattern arm; duplicate later arm ignored
            "NOTE: 12 03", // second value of the shared arm
            "NOTE: dd ee", // hole in the table -> default
            "NOTE: 19 ee", // arm that writes only `a`; `b` holds
            "NOTE: dd ee", // x selector matches no defined pattern -> default
            "NOTE: dd ee", // beyond the largest pattern -> default
        ]
    );
}
