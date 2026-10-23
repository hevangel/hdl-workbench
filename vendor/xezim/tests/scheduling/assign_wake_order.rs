//! Same-step wake order of processes parked on a clock and on nets derived
//! from it. §4.7 leaves the order open; xezim follows the reference
//! simulator, and every expectation below was measured there:
//!
//! * waiters on the procedurally written clock run first;
//! * a continuous assignment's update is its own active event (§10.3), so
//!   waiters on its target run after them, one step per assignment;
//! * an interface's input port connected to that clock comes later still:
//!   waiters on `vif.PCLK` run after waiters on an interface variable
//!   driven by `assign intf.CLK = PCLK`.
//!
//! The UVM uart example (modem_poll_test) races a modem item against a
//! status-register read on exactly the interface-port / assigned-variable
//! pair, and scored hundreds of errors while the two monitors woke in the
//! opposite order.

use xezim::simulate;

fn lines(src: &str, max_time: u64) -> Vec<String> {
    simulate(src, max_time)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

/// A consumer fed from a waiter on the clock runs before one fed from a
/// waiter on `assign CLK2 = PCLK`, at every edge both fire.
#[test]
fn assigned_clock_waiter_runs_after_source_waiter() {
    let src = r#"
module tb;
  logic PCLK = 0;
  wire CLK2;
  assign CLK2 = PCLK;
  always #1 PCLK = ~PCLK;
  mailbox #(int) mb_a = new(), mb_b = new();
  initial begin
    fork
      forever begin int v; mb_a.get(v); $display("%0t A", $time); end
      forever begin int v; mb_b.get(v); $display("%0t B", $time); end
    join_none
  end
  initial forever begin @(posedge CLK2); @(posedge CLK2); mb_b.put(1); end
  initial forever begin @(posedge PCLK); mb_a.put(1); end
  initial #8 $finish;
endmodule
"#;
    assert_eq!(
        lines(src, 100),
        ["1 A", "3 A", "3 B", "5 A", "7 A", "7 B"],
        "reference order"
    );
}

/// One step per assignment: clock, then `C2 = PCLK & EN`, then `C3 = ~C2`,
/// whatever order the waiters armed in.
#[test]
fn assignment_chain_wakes_in_depth_order() {
    let src = r#"
module tb;
  logic PCLK = 0, EN = 1;
  wire C2, C3;
  assign C2 = PCLK & EN;
  assign C3 = ~C2;
  always #1 PCLK = ~PCLK;
  initial forever begin @(negedge C3); $display("%0t C3", $time); end
  initial forever begin @(posedge C2); $display("%0t C2", $time); end
  initial forever begin @(posedge PCLK); $display("%0t PCLK", $time); end
  initial #4 $finish;
endmodule
"#;
    assert_eq!(
        lines(src, 100),
        ["1 PCLK", "1 C2", "1 C3", "3 PCLK", "3 C2", "3 C3"],
        "reference order"
    );
}

/// The uart testbench shape: class monitors on a virtual interface, one on
/// the interface's clock PORT, one on an interface variable the top drives
/// with `assign`. The top's own clock waiter first, then the assigned
/// variable's, then the port's — in either arming order.
#[test]
fn interface_port_waiter_runs_after_assigned_interface_variable() {
    let src = r#"
interface mif(input PCLK);
  logic CLK;
endinterface
class mon;
  virtual mif vif;
  string nm;
  bit use_port;
  function new(string n, virtual mif v, bit p); nm = n; vif = v; use_port = p; endfunction
  task run();
    forever begin
      if (use_port) @(posedge vif.PCLK); else @(posedge vif.CLK);
      $display("%0t %s", $time, nm);
    end
  endtask
endclass
module tb;
  logic PCLK;
  mif M(.PCLK(PCLK));
  mon a, b, c, d;
  initial begin PCLK = 0; repeat (4) #1 PCLK = ~PCLK; forever #1 PCLK = ~PCLK; end
  assign M.CLK = PCLK;
  initial begin
    a = new("port1", M, 1);
    b = new("clk1", M, 0);
    fork a.run(); b.run(); join_none
  end
  initial forever begin @(posedge PCLK); $display("%0t tb", $time); end
  initial begin
    c = new("clk2", M, 0);
    d = new("port2", M, 1);
    fork c.run(); d.run(); join_none
  end
  initial #6 $finish;
endmodule
"#;
    let out = lines(src, 100);
    for t in ["1", "3", "5"] {
        let at: Vec<&str> = out
            .iter()
            .filter_map(|l| l.strip_prefix(&format!("{t} ")))
            .collect();
        assert_eq!(at.len(), 5, "t={t}: {out:?}");
        assert_eq!(at[0], "tb", "t={t}: {at:?}");
        assert!(
            at[1].starts_with("clk") && at[2].starts_with("clk"),
            "t={t}: {at:?}"
        );
        assert!(
            at[3].starts_with("port") && at[4].starts_with("port"),
            "t={t}: {at:?}"
        );
    }
}
