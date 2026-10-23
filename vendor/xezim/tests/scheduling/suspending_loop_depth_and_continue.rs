//! Two defects in suspend-aware `for`/`while`, both reference-validated.
//!
//! 1. **Unbounded continuation chain → stack overflow.** A loop whose body
//!    suspends re-pushes its continuation from the LAST statement of its own
//!    frame, and `ProcCont::pushed` wrapped that already-exhausted frame in a
//!    fresh link every time. The chain grew one link per iteration, and the
//!    derived (recursive) `Drop` of that list aborted the process:
//!    `for (int i=0;i<2000;i++) @(posedge clk);` crashed with
//!    "has overflowed its stack". `repeat`/`forever` escaped only because they
//!    have a counted-waiter path. Fixed by splicing the tail directly when the
//!    resumed frame has nothing left, plus an iterative `Drop`.
//!
//! 2. **§12.7.2 `continue` skipped the for-step → infinite loop.** The
//!    suspend-aware path lowers `for` to `while (cond) { body; step; }`, so a
//!    `continue` skipped the step along with the rest of the body; the index
//!    never advanced and the simulation hung (no output, ran to max-time).
//!    A `LoopStep` barrier between body and step now consumes a pending
//!    `continue` while leaving `break`/`return` to exit the loop.
//!
//! Note both are invisible to a suite of short tests: nothing here loops more
//! than a few times, and `continue` in a *non*-suspending loop was always fine.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Deep suspending loops must complete (and count correctly) rather than
/// overflowing the stack. 2000 was the measured `for` threshold, 3000 the
/// `while` one — run past both.
#[test]
fn deep_suspending_loops_do_not_overflow() {
    let src = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  int fcnt = 0, wcnt = 0, tail = 0;
  initial begin
    for (int i = 0; i < 3500; i++) begin @(posedge clk); fcnt++; end
    begin
      int j = 0;
      while (j < 3500) begin @(posedge clk); wcnt++; j++; end
    end
    tail = 1;          // statements after the loop must still run
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 200_000).expect("simulate failed");
    assert_eq!(u(&sim, "fcnt"), 3500, "deep suspending for-loop");
    assert_eq!(u(&sim, "wcnt"), 3500, "deep suspending while-loop");
    assert_eq!(u(&sim, "tail"), 1, "loop tail still runs");
}

/// §12.7.2 in a loop whose body suspends: `continue` skips the rest of the
/// body, the step still runs, and `break` still exits without the step.
#[test]
fn continue_in_suspending_for_runs_the_step() {
    let src = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  int cnt = 0, brk = 0, nest = 0, tcnt = 0, sync = 0;
  task automatic step(); @(posedge clk); tcnt++; endtask
  initial begin
    // continue: body skipped, step still advances i
    for (int i = 0; i < 20; i++) begin @(posedge clk); if (i % 2) continue; cnt++; end
    // break still exits
    for (int i = 0; i < 20; i++) begin @(posedge clk); if (i == 15) break; if (i % 2) continue; brk++; end
    // nested loops: continue binds to the innermost
    for (int a = 0; a < 5; a++)
      for (int b = 0; b < 4; b++) begin @(posedge clk); if (b == 2) continue; nest++; end
    // body blocks via a task call rather than a direct event control
    for (int i = 0; i < 10; i++) begin step(); if (i % 2) continue; end
    // the non-suspending path was always correct — keep it covered
    for (int i = 0; i < 20; i++) begin if (i % 2) continue; sync++; end
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100_000).expect("simulate failed");
    assert_eq!(u(&sim, "cnt"), 10, "continue must not skip the for-step");
    assert_eq!(u(&sim, "brk"), 8, "break exits at i==15");
    assert_eq!(u(&sim, "nest"), 15, "continue binds to the inner loop");
    assert_eq!(u(&sim, "tcnt"), 10, "blocking-task body iterates fully");
    assert_eq!(u(&sim, "sync"), 10, "non-suspending loop unchanged");
}
