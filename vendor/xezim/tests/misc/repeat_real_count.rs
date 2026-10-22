//! §12.7.2/§6.12.2: a real `repeat` count converts to an integer by rounding
//! (ties away from zero). The count was read as its raw IEEE-754 bits, so
//! `repeat (10.4)` spun ~2^62 times (ivtest `br967` timed out). Both the
//! interpreted loop and a loop whose body waits (compiled as a counted loop
//! in a process) are covered. Values cross-checked against the reference
//! simulator.

#[test]
fn real_repeat_counts_round_to_integers() {
    let sim = xezim::simulate(
        r#"
module test;
  integer n1 = 0, n2 = 0, n3 = 0, n4 = 0;
  real r = 3.6;
  bit clk = 0;
  always #5 clk = ~clk;
  initial begin
    repeat (10.4) n1 = n1 + 1;
    repeat (2.5) n2 = n2 + 1;
    repeat (r) n3 = n3 + 1;
    repeat (1.6) begin @(posedge clk); n4 = n4 + 1; end
    $display("R|%0d %0d %0d %0d %0t", n1, n2, n3, n4, $time);
    $finish;
  end
endmodule
"#,
        1000,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(o.iter().any(|l| l == "R|10 3 4 2 15"), "{o:?}");
}
