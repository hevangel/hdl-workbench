//! §3.14.3: a bare `#N` counts the ENCLOSING SCOPE's timeunit, wherever it is
//! written. The elaborator pre-scales every delay from the scope's timeunit to
//! the global tick, and the executor consumes tick-denominated delays — so a
//! delay the pre-scaling pass never visits is silently interpreted as raw
//! TICKS.
//!
//! That pass walked always/initial/final blocks, module task bodies,
//! continuous-assign and gate delays, and every generate form — but never
//! walked a CLASS. So `#40` inside a class method was 40 ticks instead of 40
//! timeunits: in a `1ns/1ps` scope it ran 1000x too short, and the process
//! resumed essentially immediately. The identical `#40` in a module task was
//! correct, which is what made it look like class delays were ignored
//! altogether rather than mis-scaled. (The module-task arm of that pass
//! already carries a comment about the very same bug being fixed for tasks.)
//!
//! An INTERFACE had the same defect, for a second reason: interfaces never
//! entered the `eff_ts` timescale map at all (that walk visited
//! `Description::Module` only), so routing them through the pass scaled by the
//! tick and was an identity. Interfaces and programs are now walked too — the
//! preprocessor already recorded them — so a BFM's `#` delays count the
//! interface's timeunit like any other scope.

use xezim::simulate;

/// The `NOTE:` lines a run printed, in order.
fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

/// A module task and a class method containing the SAME `#40` must advance
/// simulation time by the same amount. An independent `#1` ticker witnesses
/// the advance, so this cannot pass on a `$time` formatting quirk alone.
#[test]
fn class_method_delay_counts_timeunits_like_a_module_task() {
    let src = r#"
`timescale 1ns/1ps
module top;
  int ticks = 0;
  always #1 ticks++;
  task ModuleWait(); #40; $display("NOTE: module %0t %0d", $time, ticks); endtask
  class Holder;
    task MethodWait(); #40; $display("NOTE: class %0t %0d", $time, ticks); endtask
  endclass
  Holder h;
  initial begin
    h = new();
    ModuleWait();
    h.MethodWait();
    $display("NOTE: end %0t %0d", $time, ticks);
    $finish;
  end
endmodule
"#;
    // 1 ns == 1000 ticks, so each `#40` is 40 ns. The buggy path made the class
    // delay 40 TICKS (40 ps), leaving time parked at 40 ns.
    assert_eq!(
        notes(src),
        vec![
            "NOTE: module 40000 39",
            "NOTE: class 80000 79",
            "NOTE: end 80000 79",
        ]
    );
}

/// The delay must survive every way a class method can be reached: directly,
/// through `fork`, and nested through another class method.
#[test]
fn class_method_delay_holds_through_fork_and_nesting() {
    let src = r#"
`timescale 1ns/1ps
module top;
  class Holder;
    task Leaf();  #40; $display("NOTE: leaf %0t", $time); endtask
    task Outer(); Leaf(); $display("NOTE: outer %0t", $time); endtask
  endclass
  Holder h;
  initial begin
    h = new();
    fork h.Leaf(); join
    h.Outer();
    $display("NOTE: end %0t", $time);
    $finish;
  end
endmodule
"#;
    assert_eq!(
        notes(src),
        vec![
            "NOTE: leaf 40000",
            "NOTE: leaf 80000",
            "NOTE: outer 80000",
            "NOTE: end 80000",
        ]
    );
}

/// An interface task's `#` delay counts the interface's timeunit too — this is
/// the shape a bus BFM uses to pace its drive routine.
#[test]
fn interface_task_delay_counts_timeunits() {
    let src = r#"
`timescale 1ns/1ps
interface probe_if;
  task Pace(); #40; endtask
endinterface
module top;
  probe_if u_if();
  initial begin
    u_if.Pace();
    $display("NOTE: iface %0t", $time);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: iface 40000"]);
}

/// With no finer precision anywhere the scaling is an identity — the fix must
/// not disturb a design whose timeunit already equals the tick.
#[test]
fn identity_when_timeunit_equals_the_tick() {
    let src = r#"
`timescale 1ns/1ns
module top;
  class Holder;
    task MethodWait(); #7; $display("NOTE: %0t", $time); endtask
  endclass
  Holder h;
  initial begin
    h = new();
    h.MethodWait();
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: 7"]);
}
