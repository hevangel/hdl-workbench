//! A scalar `bit` formal of a class method takes the formal's one-bit width
//! (§6.11, §13.5): bound to the literal `1` it kept 32 bits, so `%b` printed
//! 32 digits (#272's ENQUEUE line), and bound to `2` it read nonzero.
//! Expected values are the reference simulator's.

use xezim::simulate;

#[test]
fn scalar_bit_formals_fit_to_one_bit() {
    let src = r#"
module top;
  function automatic void mf(bit a, bit [3:0] b, logic [7:0] c, int d, byte e);
    $display("T|mf a=%b b=%b c=%b d=%0d e=%0d bw=%0d", a, b, c, d, e, $bits(a));
  endfunction
  task automatic mt(bit a, bit [3:0] b);
    $display("T|mt a=%b b=%b", a, b);
  endtask
  class C;
    function void cf(bit a, bit [3:0] b, logic [7:0] c, int d, byte e);
      $display("T|cf a=%b b=%b c=%b d=%0d e=%0d", a, b, c, d, e);
    endfunction
    task ct(bit a, bit [3:0] b);
      $display("T|ct a=%b b=%b", a, b);
    endtask
    static function void sf(bit a, bit [3:0] b);
      $display("T|sf a=%b b=%b", a, b);
    endfunction
  endclass
  C c;
  initial begin
    c = new;
    mf(1, 8'hfe, 4'h3, 16'hffff, 300);
    mt(3, 8'h1f);
    c.cf(1, 8'hfe, 4'h3, 16'hffff, 300);
    c.ct(3, 8'h1f);
    C::sf(2, 5'h1f);
    c.cf(1'b1, 4'hf, 8'h1, 5, 6);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|l| l.message.strip_prefix("T|"))
        .collect();
    assert_eq!(
        got,
        [
            "mf a=1 b=1110 c=00000011 d=65535 e=44 bw=1",
            "mt a=1 b=1111",
            "cf a=1 b=1110 c=00000011 d=65535 e=44",
            "ct a=1 b=1111",
            "sf a=0 b=1111",
            "cf a=1 b=1111 c=00000001 d=5 e=6",
        ]
    );
}
