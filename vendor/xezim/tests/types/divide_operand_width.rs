//! §11.6.1 Table 11-21: `/` and `%` size BOTH operands to the larger of their
//! widths and the context. The compiled path took only the dividend's width
//! for the operands' context, so in `ring[x % (n < 32 ? n : 32)]` (5-bit `x`,
//! a self-determined index) the ternary divisor compiled at 5 bits, 32 became
//! 0, and the modulo read x. `x % 32` and `x % n` were already right.
//! Expected values are the reference simulator's.

use xezim::simulate;

#[test]
fn a_ternary_divisor_is_not_narrowed_to_the_dividend() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [4:0] x = 5'd20;
  int n = 40;
  logic [7:0] ring [0:31];
  logic [7:0] a1, a2, a3, a4, a5;
  initial for (int k = 0; k < 32; k++) ring[k] = 8'(k + 100);
  always @(posedge clk) begin
    a1 = ring[x % (n < 32 ? n : 32)];
    a2 = ring[x % 32];
    a3 = ring[x % n];
    a4 = ring[(n < 32 ? n : 32) - 12];
    a5 = 8'((x + 5'd20) % (n < 32 ? n : 64));
  end
  initial begin
    #1 clk = 1; #1;
    $display("T| a1=%0d a2=%0d a3=%0d a4=%0d a5=%0d", a1, a2, a3, a4, a5);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("design must run");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(got, ["T| a1=120 a2=120 a3=120 a4=120 a5=40"]);
}
