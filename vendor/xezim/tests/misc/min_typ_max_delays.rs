//! §A.8.3: a min:typ:max expression takes its typical value — in a
//! parameter, a gate delay, an intra-assignment delay and a procedural delay.
//! Expected lines are the reference simulator's.

use xezim::simulate;

#[test]
fn min_typ_max_uses_the_typical_value() {
    let src = r#"
module top;
  parameter P = (1:2:3);
  reg a;
  reg [3:0] v;
  wire q;
  buf #(1:4:9) g (q, a);
  initial begin
    a = 0; v = 0;
    #10 a = 1;
    v = #(2:5:7) 4'd3;
    $display("V|%0t v=%0d P=%0d", $time, v, P);
    #(1:2:3) $display("T|%0t", $time);
  end
  always @(q) $display("Q|%0t q=%b", $time, q);
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.contains('|'))
        .collect();
    assert_eq!(got, ["Q|4 q=0", "Q|14 q=1", "V|15 v=3 P=2", "T|17"]);
}
