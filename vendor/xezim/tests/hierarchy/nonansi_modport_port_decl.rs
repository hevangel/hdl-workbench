//! §25.5: NON-ANSI body port declaration with a modport-qualified interface
//! type — `counter_if.counter_mp c_data;` after a plain port list. The
//! parser died on the dot ("expected identifier, found Dot") while every
//! ANSI modport form worked; Verilator's t_interface_modport keeps this form
//! under an ifndef exactly because tools disagree here. Reference simulator
//! compiles and simulates it (value=10 for this stimulus).

use xezim::simulate;

const SRC: &str = r#"
interface counter_if;
  logic [3:0] value;
  logic reset;
  modport counter_mp (input reset, output value);
endinterface

module counter_nansi_m (clkm, c_data, i_value);
  input clkm;
  counter_if.counter_mp c_data;
  input logic [3:0] i_value;
  always @(posedge clkm) c_data.value <= c_data.reset ? i_value : c_data.value + 1;
endmodule

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  counter_if ci();
  counter_nansi_m u(clk, ci, 4'd7);
  initial begin
    ci.reset = 1;
    @(negedge clk); ci.reset = 0;
    repeat (3) @(negedge clk);
    $display("NOTE: value=%0d", ci.value);
    $finish;
  end
endmodule
"#;

#[test]
fn nonansi_modport_typed_port_declaration_parses() {
    let sim = simulate(SRC, 1_000_000).expect("simulate failed");
    let notes: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect();
    assert_eq!(notes, ["NOTE: value=10"]);
}
