// Regression test: edge-sensitive waits on member chains rooted at `this`
// inside class methods — `@(posedge this.vif.clk)`, `@(posedge bus.clk)`
// (implicit-this shorthand), and chains through a nested handle
// (`this.drv.bus.clk`).
//
// Before the fix, `member_chain_as_flat_ident` had no arm for
// `ExprKind::This`, so a `this.`-rooted chain collected an EMPTY sensitivity
// list — the event control became a no-op and the method spun at time 0
// re-executing the wait. Separately, `event_to_sens` resolved dotted names
// only through the static hierarchy: a chain that hops through a class-handle
// property (`this.drv.bus.clk`) or a virtual-interface binding produced a
// name with no signal id and parked forever. The fix walks the handle chain
// (`resolve_sens_name_via_handles`) and late-binds the sensitivity to the
// bound interface signal.
//
// Verified against reference-simulator behavior.

use std::process::Command;

fn xezim() -> String {
    env!("CARGO_BIN_EXE_xezim").to_string()
}

fn run(src: &str, tag: &str) -> String {
    let path = format!("/tmp/tces_{tag}.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(xezim())
        .args(["--simulate", "-s", "top", &path])
        .output()
        .expect("xezim failed to start");
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// `@(posedge this.vif.clk)` in a class task must wake on each interface
/// clock edge, not spin or park forever.
#[test]
fn explicit_this_vif_posedge() {
    let out = run(
        r#"
interface clk_if(input logic clk);
endinterface

class watcher;
  virtual clk_if vif;
  int edges;
  task watch();
    repeat (3) begin
      @(posedge this.vif.clk);
      edges++;
      $display("edge %0d at %0t", edges, $time);
    end
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  clk_if u_if(clk);

  initial begin
    watcher w = new;
    w.vif = u_if;
    w.watch();
    $display("done edges=%0d at %0t", w.edges, $time);
    $finish;
  end
endmodule
"#,
        "explicit",
    );
    assert!(out.contains("edge 1 at 5"), "missing edge 1: {out}");
    assert!(out.contains("edge 3 at 25"), "missing edge 3: {out}");
    assert!(out.contains("done edges=3 at 25"), "missing done: {out}");
}

/// Implicit-this shorthand: `@(posedge vif.clk)` (no `this.` prefix) inside a
/// class task must resolve through the instance's virtual-interface binding.
#[test]
fn implicit_this_vif_posedge() {
    let out = run(
        r#"
interface clk_if(input logic clk);
endinterface

class watcher;
  virtual clk_if vif;
  task watch();
    @(posedge vif.clk);
    $display("woke at %0t", $time);
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  clk_if u_if(clk);

  initial begin
    watcher w = new;
    w.vif = u_if;
    w.watch();
    $finish;
  end
endmodule
"#,
        "implicit",
    );
    assert!(out.contains("woke at 5"), "missing wake: {out}");
}

/// A chain hopping through a nested class-handle property —
/// `@(posedge this.drv.vif.clk)` — must follow the handle chain to the bound
/// interface signal.
#[test]
fn nested_handle_chain_posedge() {
    let out = run(
        r#"
interface clk_if(input logic clk);
endinterface

class driver;
  virtual clk_if vif;
endclass

class env;
  driver drv;
  function new();
    drv = new;
  endfunction
  task watch();
    repeat (2) @(posedge this.drv.vif.clk);
    $display("nested woke at %0t", $time);
  endtask
endclass

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  clk_if u_if(clk);

  initial begin
    env e = new;
    e.drv.vif = u_if;
    e.watch();
    $finish;
  end
endmodule
"#,
        "nested",
    );
    assert!(
        out.contains("nested woke at 15"),
        "missing nested wake: {out}"
    );
}
