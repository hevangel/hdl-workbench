fn output(src: &str) -> Vec<String> {
    let sim = xezim::simulate(src, 100).expect("simulate");
    sim.output
        .iter()
        .filter(|line| line.message.starts_with("STATE"))
        .map(|line| line.message.clone())
        .collect()
}

#[test]
fn shared_clock_and_nand_banks_track_every_enable() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  logic clk;
  logic e0, e1, e2, e3;
  wire g0, g1, g2, g3;
  wire n0, n1, n2, n3;

  assign g0 = clk & e0;
  assign g1 = clk & e1;
  assign g2 = clk & e2;
  assign g3 = clk & e3;

  assign n0 = ~(clk & e0);
  assign n1 = ~(clk & e1);
  assign n2 = ~(clk & e2);
  assign n3 = ~(clk & e3);

  initial begin
    clk = 0;
    e0 = 1;
    e1 = 0;
    e2 = 1'bx;
    e3 = 1;
    #1;
    $display("STATE0 g=%b%b%b%b n=%b%b%b%b", g0,g1,g2,g3,n0,n1,n2,n3);

    clk = 1;
    #1;
    $display("STATE1 g=%b%b%b%b n=%b%b%b%b", g0,g1,g2,g3,n0,n1,n2,n3);

    // A control transition must update its branch even though the common
    // clock input did not move.
    e1 = 1;
    e3 = 0;
    #1;
    $display("STATE2 g=%b%b%b%b n=%b%b%b%b", g0,g1,g2,g3,n0,n1,n2,n3);

    clk = 1'bx;
    e0 = 0;
    #1;
    $display("STATE3 g=%b%b%b%b n=%b%b%b%b", g0,g1,g2,g3,n0,n1,n2,n3);
    $finish;
  end
endmodule
"#;

    assert_eq!(
        output(src),
        [
            "STATE0 g=0000 n=1111",
            "STATE1 g=10x1 n=01x0",
            "STATE2 g=11x0 n=00x1",
            "STATE3 g=0xx0 n=1xx1",
        ]
    );
}
