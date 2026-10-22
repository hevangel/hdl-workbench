//! A clock generator whose target is an interface member (`BUS.clk`) must
//! toggle THAT signal. The `initial VAR = C; forever #d VAR = ~VAR;` and
//! `always #d VAR = ~VAR;` fast paths keyed the target on its last segment
//! alone, so `BUS.clk` bound to whichever signal ending in `.clk` came first
//! (another interface's `clk`) or to a top-level `clk`: the real clock never
//! moved and every `@(posedge BUS.clk)` hung. The expected lines were
//! cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn interface_member_clock_generators_drive_their_own_member() {
    let src = r#"
interface bus_if;
  logic clk;
endinterface
interface irq_if;
  logic clk;
  logic irq;
endinterface
module top;
  logic clk;
  bus_if BUS();
  bus_if BUS2();
  irq_if INT();
  initial begin
    BUS.clk = 0;
    forever begin
      #10 BUS.clk = ~BUS.clk;
    end
  end
  initial BUS2.clk = 1;
  always #7 BUS2.clk = ~BUS2.clk;
  initial begin
    repeat (3) @(posedge BUS.clk);
    $display("%0t bus posedge 3", $time);
    repeat (2) @(posedge BUS2.clk);
    $display("%0t bus2 posedge 2", $time);
    $display("%0t clk=%b int.clk=%b", $time, clk, INT.clk);
    $finish;
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        vec![
            "50 bus posedge 3".to_string(),
            "70 bus2 posedge 2".to_string(),
            "70 clk=x int.clk=x".to_string(),
        ],
        "{out:?}"
    );
}
