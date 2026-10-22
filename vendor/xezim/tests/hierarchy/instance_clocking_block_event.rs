//! §14.3 — a clocking block declared inside a module or interface INSTANCE
//! is registered under its instance path (`m.cb`). A bare `@(cb)` in that
//! instance's own task or process must resolve through the executing scope.
//! It used to wait on a signal literally named `cb`: in an interface task
//! that returned at once, so a monitor BFM's `while (!cb.valid) @(cb);`
//! sampling loop spun at one timestamp forever.
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
fn interface_task_clocking_wait_blocks() {
    let o = lines(
        r#"
interface mon_bfm(input aclk, input v);
  clocking cb @(posedge aclk);
    input v;
  endclocking
  task sample();
    @(cb);
    $display("%0t first", $time);
  endtask
endinterface
module top;
  bit clk, v;
  initial begin clk = 0; forever #10 clk = ~clk; end
  mon_bfm m(.aclk(clk), .v(v));
  initial begin #35; m.sample(); m.sample(); m.sample(); end
  initial begin #100; $finish; end
endmodule
"#,
    );
    let got: Vec<&String> = o.iter().filter(|l| l.ends_with(" first")).collect();
    assert_eq!(got, ["50 first", "70 first", "90 first"], "{o:?}");
}

#[test]
fn vif_sampling_loop_steps_on_the_clock() {
    let o = lines(
        r#"
interface mon_bfm(input aclk, input aresetn, input v);
  clocking cb @(posedge aclk);
    default input #1step output #1step;
    input v;
  endclocking
  int n;
  task wait_rst();
    @(negedge aresetn);
    @(posedge aresetn);
  endtask
  task sample();
    @(cb);
    $display("%0t first", $time);
    while (cb.v !== 1) begin
      @(cb);
      n++;
      if (n < 5) $display("%0t loop", $time);
    end
  endtask
endinterface
class proxy;
  virtual mon_bfm vif;
  task run();
    vif.wait_rst();
    forever begin
      vif.sample();
    end
  endtask
endclass
module top;
  bit clk, rst, v;
  initial begin clk = 0; forever #10 clk = ~clk; end
  initial begin rst = 1; #15 rst = 0; #20 rst = 1; end
  mon_bfm m(.aclk(clk), .aresetn(rst), .v(v));
  initial begin
    proxy px = new;
    px.vif = m;
    px.run();
  end
  initial begin #100; $display("n=%0d", m.n); $finish; end
endmodule
"#,
    );
    for want in ["50 first", "70 loop", "90 loop", "n=2"] {
        assert!(o.iter().any(|l| l == want), "missing {want:?}: {o:?}");
    }
    assert!(!o.iter().any(|l| l == "35 first"), "{o:?}");
}
