//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    xezim::simulate(src, 1_000_000)
        .expect("simulation failed")
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn exponent_type_and_width_are_self_determined() {
    let got = output(
        r###"module top;
 logic signed [7:0] e=-2,f=127;
 logic [7:0] u=0,v;
 logic [15:0] w;
 wire [7:0] c=u+(8'sd2**(e/f));
 initial begin
  #1;
  v=u+(8'sd2**(e/f));
  $display("T|exponent continuous %0d procedural %0d",c,v);
  w=16'd0+(16'd2**(8'd1<<8));
  $display("T|width exponent %0d",w);
  $finish;
 end
endmodule
"###,
    );
    assert!(
        got.contains("T|exponent continuous 1 procedural 1"),
        "{got}"
    );
    assert!(got.contains("T|width exponent 1"), "{got}");
}
