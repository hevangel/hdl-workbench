//! §11.4.2: `++x` / `x--` yield a value of `x`'s type. The interpreter added
//! or subtracted an UNSIGNED 1, which made the result unsigned whatever `x`
//! was, so `int x = -5; (++x < 0)` was false and an associative element
//! written from such a result lost its sign.
//!
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn inc_dec_result_keeps_operand_signedness() {
    let src = r#"
module top;
  int x = -5;
  int y;
  byte b = -1;
  int aa[int];
  initial begin
    y = (++x < 0);
    $display("A %0d %0d", y, x);
    $display("B %0d %0d", (x-- < 0), (--b < 0));
    $display("C %0d %0d", b++, b);
    aa[3] = -7;
    $display("D %0d %0d", (++aa[3] < 0), aa[3]--);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["A 1 -4", "B 1 1", "C -2 -1", "D 1 -6"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
