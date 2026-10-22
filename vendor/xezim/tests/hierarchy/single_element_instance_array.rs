//! §23.3.3 — a one-element array of instances (`bus_if b[1]`) names its
//! element `b[0]`, like any other array. The expansion skipped arrays of
//! size one and registered a plain `b`, so `b[0]` bound to nothing: a module
//! port connected to it read x, and an edge wait on a member passed through
//! it completed at once. An AVIP with `NO_OF_SLAVES = 1` wires its slave
//! agent to `intf_s[0]`, and the slave driver's reset/clock waits never
//! blocked.
//!
//! Expected output cross-checked against the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn one_element_interface_array_element_binds() {
    let o = lines(
        r#"
interface drv_bfm(input bit pclk, input bit preset_n);
  task wait_rst();
    $display("%0t enter preset_n=%b", $time, preset_n);
    @(negedge preset_n);
    $display("%0t negedge", $time);
  endtask
endinterface
interface bus_if(input bit pclk, input bit preset_n);
  logic psel;
endinterface
module agent_bfm(bus_if intf);
  drv_bfm d(.pclk(intf.pclk), .preset_n(intf.preset_n));
endmodule
module top;
  bit clk, rst;
  initial begin clk = 0; forever #10 clk = ~clk; end
  initial begin rst = 1; #15 rst = 0; #20 rst = 1; end
  bus_if intf_s[1](clk, rst);
  agent_bfm a(intf_s[0]);
  initial begin
    #1;
    a.d.wait_rst();
    $display("%0t done", $time);
  end
  initial #100 $finish;
endmodule
"#,
    );
    for want in ["1 enter preset_n=1", "15 negedge", "15 done"] {
        assert!(o.iter().any(|l| l == want), "missing {want:?}: {o:?}");
    }
}
