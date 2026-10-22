//! §25.5.4 / §25.9 — a task of an interface instance, called inside a
//! `forever` loop through a virtual interface or a hierarchical path, has no
//! class receiver. The forever runner's null-receiver guard evaluated the
//! receiver name as an object handle, found 0 (an interface is not an
//! object) and retired the loop: silently inside a class method, with a
//! bogus "null receiver" error at module level. An AVIP monitor proxy's
//! `forever begin vif.wait_for_idle_state(); ... end` never ran once.
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
fn vif_task_in_class_forever_loop_runs() {
    let o = lines(
        r#"
interface mon_if(input pclk, input areset);
  task wait_for_reset();
    @(negedge areset);
    @(posedge areset);
    $display("%0t reset done", $time);
  endtask
  task sample_idle_state();
    @(posedge pclk);
    $display("%0t sample_idle", $time);
  endtask
  task wait_for_idle_state();
    @(posedge pclk);
    $display("%0t wait_idle", $time);
  endtask
endinterface
package p;
  class proxy;
    virtual mon_if vif;
    task run();
      vif.wait_for_reset();
      vif.sample_idle_state();
      forever begin
        vif.wait_for_idle_state();
        $display("%0t loop", $time);
        @(posedge vif.pclk);
      end
    endtask
  endclass
endpackage
module top;
  import p::*;
  bit clk, rst;
  initial begin clk = 0; forever #10 clk = ~clk; end
  initial begin
    rst = 1;
    repeat (2) @(posedge clk);
    rst = 0;
    repeat (2) @(posedge clk);
    rst = 1;
  end
  mon_if m(.pclk(clk), .areset(rst));
  initial begin
    proxy px = new;
    px.vif = m;
    px.run();
  end
  initial #200 $finish;
endmodule
"#,
    );
    let want = [
        "70 reset done",
        "90 sample_idle",
        "110 wait_idle",
        "110 loop",
        "150 wait_idle",
        "150 loop",
        "190 wait_idle",
        "190 loop",
    ];
    let got: Vec<&String> = o
        .iter()
        .filter(|l| l.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .collect();
    assert_eq!(got, want.iter().collect::<Vec<_>>(), "{o:?}");
}

#[test]
fn hierarchical_interface_task_in_module_forever_loop_runs() {
    let o = lines(
        r#"
interface mon_if(input pclk);
  task w();
    @(posedge pclk);
    $display("%0t w", $time);
  endtask
endinterface
module top;
  bit clk;
  initial begin clk = 0; forever #10 clk = ~clk; end
  mon_if m(.pclk(clk));
  initial begin
    #75;
    forever begin
      m.w();
      $display("%0t loop", $time);
    end
  end
  initial #120 $finish;
endmodule
"#,
    );
    assert!(!o.iter().any(|l| l.contains("null receiver")), "{o:?}");
    for want in ["90 w", "90 loop", "110 w", "110 loop"] {
        assert!(o.iter().any(|l| l == want), "missing {want:?}: {o:?}");
    }
}
