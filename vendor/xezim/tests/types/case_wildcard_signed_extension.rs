//! §12.5.1 + §11.6.1: casez/casex (and `==?`, §11.4.6) extend unequal-width
//! operands to the common width SIGN-replicated when both operands are
//! signed — including an X/Z sign bit, which replicates as itself (so a Z
//! sign bit extends into wildcard fill for casez).
//!
//! The `===` paths already did this; the wildcard comparisons zero-filled
//! past the narrower operand's width in both the packed fast path and the
//! per-bit slow path, so `casez (signed_sel) 8'sb11111111:` never matched a
//! negative 4-bit selector. All expectations verified against the reference
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
  logic signed [3:0] ss;
  logic signed [7:0] sel8;
  logic [3:0] rz, rx, rzs;
  logic wq;
  logic clk = 0;
  always #5 clk = ~clk;
  always @(posedge clk) begin
    casez (ss)
      8'sb11111111: rz <= 4'd1;  // matches only via sign extension
      8'b00001111:  rz <= 4'd2;  // what plain zero extension would hit
      default:      rz <= 4'd15;
    endcase
    casex (ss)
      8'sb11111110: rx <= 4'd1;
      default:      rx <= 4'd15;
    endcase
    casez (sel8)
      4'sb?111: rzs <= 4'd1;     // Z sign bit replicates as wildcard fill
      default:  rzs <= 4'd15;
    endcase
    wq <= (sel8 ==? -4'sd2);
  end
  initial begin
    ss = -4'sd1; sel8 = 8'shA7;
    @(negedge clk) $display("NOTE: %0d %0d %0d %b", rz, rx, rzs, wq);
    ss = -4'sd2; sel8 = -8'sd2;
    @(negedge clk) $display("NOTE: %0d %0d %0d %b", rz, rx, rzs, wq);
    $finish;
  end
endmodule
"#;

#[test]
fn wildcard_case_sign_extends_when_both_operands_signed() {
    assert_eq!(
        notes(SRC),
        [
            "NOTE: 1 15 1 0",  // -1: casez matches signed arm; A7 low bits 111 wild-match
            "NOTE: 15 1 15 1", // -2: casex matches; FE fails ?111; ==? -2 true
        ]
    );
}
