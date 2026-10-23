//! §11.4.13 `inside` over an INTERFACE port inside an instantiated module:
//! `bus.addr inside {[LO:HI]}`. The instance rewrite maps `bus.x` to the
//! connected interface everywhere else, but it did not descend into an
//! `inside` expression, so the operand kept the port name, read x, and the
//! test was always false (a bus slave then never returned read data).
//! Expected values cross-checked against the reference simulator.

use xezim::simulate;

const DESIGN: &str = r#"
interface bus_if;
  logic clk;
  logic [31:0] addr;
  logic [7:0] sel;
endinterface
module slave(interface bus);
  logic [7:0] lim = 8'd9;
  logic e1, e2, e3, e4, e5;
  always @(posedge bus.clk) begin
    e1 <= bus.addr inside {[32'h0100_0000:32'h0104_0000]};
    if (bus.addr inside {[32'h0100_0000:32'h0104_0000]}) e2 <= 1; else e2 <= 0;
    e3 <= bus.addr inside {32'h0100_0004, 32'h0};
    e4 <= lim inside {[bus.sel : 8'd10]};
    e5 <= bus.sel inside {[8'd0 : lim]};
  end
endmodule
module top;
  bus_if BUS();
  slave DUT(.bus(BUS));
  initial begin
    BUS.clk = 0;
    BUS.addr = 32'h0100_0004;
    BUS.sel = 8'd3;
    #1 BUS.clk = 1; #1;
    $display("P1 e1=%b e2=%b e3=%b e4=%b e5=%b", DUT.e1, DUT.e2, DUT.e3, DUT.e4, DUT.e5);
    BUS.clk = 0; BUS.addr = 32'h0200_0000; BUS.sel = 8'd12;
    #1 BUS.clk = 1; #1;
    $display("P2 e1=%b e2=%b e3=%b e4=%b e5=%b", DUT.e1, DUT.e2, DUT.e3, DUT.e4, DUT.e5);
  end
endmodule
"#;

#[test]
fn inside_reads_interface_port_members() {
    let sim = simulate(DESIGN, 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with('P'))
        .collect();
    assert_eq!(
        got,
        ["P1 e1=1 e2=1 e3=1 e4=1 e5=1", "P2 e1=0 e2=0 e3=0 e4=0 e5=0"]
    );
}
