//! §11.4.2/§11.4.4/§11.8: arithmetic and relational operators on operands
//! wider than 128 bits keep every bit. The value model computed `+ - * / %
//! **` in `u128` and compared only the low 64 bits, so `logic signed
//! [199:0] a = -1` held 2^128-1, a signed 4096-bit `-1` (UVM's
//! `uvm_bitstream_t`) printed as 340282366920938463463374607431768211455,
//! and `(1 << 130) > (1 << 129)` was false. Expected output cross-checked
//! against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
module tb;
  logic signed [199:0] a, b, r;
  logic [199:0] ua, ub, ur;
  logic signed [99:0] n100;
  logic signed [127:0] s128;
  logic [4095:0] big;
  logic signed [4095:0] sbig;
  initial begin
    a = -1;               $display("neg1   %h", a);
    a = -200'sd5;         $display("neg5   %0d", a);
    a = 200'sd1 << 150; b = 200'sd3 << 140;
    r = a + b;            $display("add    %h", r);
    r = a - b;            $display("sub    %h", r);
    r = b - a;            $display("subn   %0d", r);
    r = (200'sd1 << 70) * (200'sd3 << 70); $display("mul    %h", r);
    r = -(200'sd7 << 130) / 200'sd3; $display("div    %0d", r);
    r = -(200'sd7 << 130) % 200'sd3; $display("mod    %0d", r);
    ua = (200'd1 << 199) + 200'd12345; ub = 200'd1000;
    ur = ua / ub;         $display("udiv   %0d", ur);
    ur = ua % ub;         $display("umod   %0d", ur);
    $display("lt     %0d %0d %0d", a < b, b < a, (-a) < b);
    $display("ult    %0d %0d", ua < ub, ub < ua);
    $display("le     %0d %0d", a <= a, (a+1) <= a);
    n100 = -3; s128 = n100 + 128'sd0; $display("sext   %0d", s128);
    s128 = -128'sd9 * 128'sd3; $display("m128   %0d", s128);
    r = 200'sd2 ** 150;   $display("pow    %h", r);
    r = -a;               $display("negv   %h", r);
    big = 0; big = ~big;  $display("bigd   %0d", big[255:0]);
    sbig = -1;            $display("sbig   %0d", sbig);
    sbig = sbig * 3;      $display("sbig3  %0d", sbig);
    ur = 200'd5; ur = ur - 200'd6; $display("uwrap  %h", ur);
    $display("cmpgt  %0d", (200'd1 << 130) > (200'd1 << 129));
  end
endmodule
"#;

#[test]
fn arithmetic_and_compare_past_128_bits() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(
        msgs,
        [
            "neg1   ffffffffffffffffffffffffffffffffffffffffffffffffff",
            "neg5   -5",
            "add    00000000000040300000000000000000000000000000000000",
            "sub    0000000000003fd00000000000000000000000000000000000",
            "subn   -1423066302981235389219248022273373568600375296",
            "mul    00000000000000300000000000000000000000000000000000",
            "div    -3175968757928758992324829669363169973589",
            "mod    -1",
            "udiv   803469022129495137770981046170581301261101496891396417663",
            "umod   33",
            "lt     0 1 1",
            "ult    0 1",
            "le     1 0",
            "sext   -3",
            "m128   -27",
            "pow    00000000000040000000000000000000000000000000000000",
            "negv   ffffffffffffc0000000000000000000000000000000000000",
            "bigd   115792089237316195423570985008687907853269984665640564039457584007913129639935",
            "sbig   -1",
            "sbig3  -3",
            "uwrap  ffffffffffffffffffffffffffffffffffffffffffffffffff",
            "cmpgt  1",
        ],
    );
}
