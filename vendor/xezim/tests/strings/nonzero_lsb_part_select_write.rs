//! Regression test for part-select READ *and* WRITE on packed vectors declared
//! with a non-zero lower bound (IEEE 1800-2023 §7.4.1 / §11.5.1).
//!
//! `logic [7:4] v;` is a 4-bit vector whose user indices run 7..4; user index
//! `i` maps to physical storage bit `i - lo_b` (lo_b = 4). The read path was
//! fixed earlier; the WRITE path (`w[3:2] = ...`) still used the raw indices and
//! landed one bit high (`w[3:2]` wrote physical bits [3:2] of a 3-bit word
//! instead of [2:1]), so `logic [3:1] w; w[3:2] = 2'b11` produced 3'b100 (= 4)
//! instead of the reference 3'b110 (= 6).

use xezim::simulate;

const SRC: &str = r#"
module top;
  logic [7:4] v;   // 4-bit vector, lower bound 4
  logic [3:1] w;   // 3-bit vector, lower bound 1
  initial begin
    v = 4'b0010;          // v[5]=1 -> v[6:5] reads as 2'b01 = 1
    if (v[6:5] != 3'd1) begin
      $display("TAG_FAIL read v[6:5]=%0d expected 1", v[6:5]);
    end else begin
      w = 3'b000;
      w[3:2] = 2'b11;     // sets w[3],w[2] -> physical 3'b110 = 6
      if (w == 3'd6)
        $display("TAG_PASS");
      else
        $display("TAG_FAIL write w=%0d expected 6", w);
    end
  end
endmodule
"#;

#[test]
fn test_nonzero_lsb_part_select_write() {
    let sim = simulate(SRC, 10_000).expect("simulation failed");
    assert!(
        sim.output
            .iter()
            .any(|line| line.message.contains("TAG_PASS")),
        "expected TAG_PASS in output, got: {:?}",
        sim.output
    );
}
