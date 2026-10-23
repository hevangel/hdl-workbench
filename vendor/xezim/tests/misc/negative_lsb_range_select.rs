//! Regression test for vector range select on a signal declared with negative LSB bound.
//! IEEE 1800-2023 §7.4.1 / §11.5.1: `reg [11:-4] w;` maps user index `i` to physical
//! bit offset `i - (-4)`. Range select `w[8:0]` maps to physical bits [12:4].

use xezim::simulate;

const SRC: &str = r#"
module top;
  reg [8:0] v;
  reg [11:-4] w;
  initial begin
    w = 16'hE020; // 16'b1110_0000_0010_0000
    v = w[8:0];   // physical bits 12:4 => 9'b000000010 = 2
    if (v == 9'd2) begin
      $display("TAG_PASS");
    end else begin
      $display("TAG_FAIL got=%0d expected=2", v);
    end
  end
endmodule
"#;

#[test]
fn test_negative_lsb_range_select() {
    let sim = simulate(SRC, 10_000).expect("simulation failed");
    assert!(
        sim.output
            .iter()
            .any(|line| line.message.contains("TAG_PASS")),
        "expected TAG_PASS in output, got: {:?}",
        sim.output
    );
}
