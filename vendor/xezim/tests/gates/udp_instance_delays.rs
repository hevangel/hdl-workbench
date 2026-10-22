//! §29.8: a UDP instance takes at most a rise and a fall delay. The reference
//! simulator rejects three and runs the two-delay instance with the same
//! output.

use xezim::simulate;

const UDP: &str = r#"
primitive mux2 (out, ctl, in0, in1);
  output out;
  input ctl, in0, in1;
  table
    0 0 ? : 0;
    0 1 ? : 1;
    1 ? 0 : 0;
    1 ? 1 : 1;
    x 0 0 : 0;
    x 1 1 : 1;
  endtable
endprimitive
"#;

#[test]
fn three_udp_delays() {
    let src = format!(
        "{UDP}module top;\n  reg ctl, in0, in1;\n  wire out;\n  \
         mux2 #(10, 20, 30) q(out, ctl, in0, in1);\nendmodule\n"
    );
    assert!(simulate(&src, 10).is_err());
}

#[test]
fn two_udp_delays() {
    let src = format!(
        "{UDP}{}",
        r#"module top;
  reg ctl = 0, in0 = 1, in1 = 0;
  wire out;
  mux2 #(10, 20) q(out, ctl, in0, in1);
  initial begin
    #5 $display("U|%b", out);
    #10 $display("U|%b", out);
  end
endmodule
"#
    );
    let sim = simulate(&src, 100).expect("two delays are legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("U|"))
        .collect();
    assert_eq!(lines, ["x", "1"], "{:?}", sim.output);
}
