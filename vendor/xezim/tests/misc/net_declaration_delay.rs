//! §6.7/§10.3.1: a net declaration assignment with a net delay after the data
//! type (`wire [3:0] #2 w = x;`) drives the net through that delay. Expected
//! lines are the reference simulator's.

use xezim::simulate;

#[test]
fn net_declaration_assignment_is_delayed() {
    let src = r#"
module top;
  reg [3:0] x;
  wire [3:0] #2 w = x;
  wire #(3) v = x[0];
  initial begin
    x = 4'd1;
    #10 x = 4'd8;
  end
  always @(w or v) $display("W|%0t w=%0d v=%b", $time, w, v);
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with("W|"))
        .collect();
    assert_eq!(
        got,
        ["W|2 w=1 v=x", "W|3 w=1 v=1", "W|12 w=8 v=1", "W|13 w=8 v=0"]
    );
}
