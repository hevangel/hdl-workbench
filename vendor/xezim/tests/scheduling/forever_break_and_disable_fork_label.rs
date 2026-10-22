//! Round-2 audit of the "two implementations" surface: constructs that have a
//! synchronous path (`exec_statement`) and a suspend-aware one
//! (`run_process_stmts`) must agree. Three defects, all reference-validated.
//!
//! 1. **`forever` ignored loop control (synchronous path).** Its loop checked
//!    no flag at all, so a `break` left the body no-opping for the rest of the
//!    100k cap and — the real damage — the flag SURVIVED the loop, silently
//!    skipping every statement after it, `$finish` included. Only reachable
//!    when the `forever` is nested inside a block; a top-level one is handled
//!    by the suspend-aware arm and always worked.
//!
//! 2. **`exec_forever_sched` dropped the continuation chain.** It took the
//!    current frame's remainder as a flat slice, so for a `forever` nested in
//!    a `begin...end` everything after the enclosing block was lost when the
//!    loop exited. Now splices via `ProcCont::pushed` like every other arm.
//!
//! 3. **§9.6.2 `disable <label>` could not name a fork child's block.**
//!    `disable_labels` was populated only for initial blocks at start-up, so
//!    `fork begin : blk ... end join_none` + `disable blk` found no target,
//!    fell through to the self-unwind path, and left the child running — a
//!    later `wait fork` then blocked forever. `fork : name` worked, which is
//!    what hid it.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// `break` must exit a `forever` and let the statements after it run, whether
/// or not the loop is nested and whether or not its body suspends.
#[test]
fn break_exits_forever_and_tail_runs() {
    let src = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  int bare = 0, nested = 0, susp = 0, tail = 0;
  initial begin
    forever begin bare++; if (bare >= 5) break; end
    begin forever begin nested++; if (nested >= 5) break; end end
    begin forever begin @(posedge clk); susp++; if (susp >= 5) break; end end
    tail = 1;                 // continuation behind the block must survive
  end
endmodule
"#;
    let sim = simulate(src, 20_000).expect("simulate failed");
    assert_eq!(u(&sim, "bare"), 5, "top-level forever");
    assert_eq!(
        u(&sim, "nested"),
        5,
        "forever nested in a block (synchronous)"
    );
    assert_eq!(u(&sim, "susp"), 5, "forever nested in a block (suspending)");
    assert_eq!(
        u(&sim, "tail"),
        1,
        "statements after the enclosing block still run"
    );
}

/// §9.6.2: disabling a fork child by its own block label terminates it, so a
/// following `wait fork` completes.
#[test]
fn disable_labeled_fork_child_terminates_it() {
    let src = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  int ticks = 0, after_dis = 0, kept = 0, done = 0;
  initial begin
    fork
      begin : dblk forever begin @(posedge clk); ticks++; end end
    join_none
    repeat (4) @(posedge clk);
    disable dblk;
    after_dis = ticks;
    repeat (10) @(posedge clk);
    kept = ticks - after_dis;   // must be 0: the child is gone
    wait fork;                  // must not block on the disabled child
    done = 1;
  end
endmodule
"#;
    let sim = simulate(src, 40_000).expect("simulate failed");
    assert_eq!(u(&sim, "kept"), 0, "disabled child must stop advancing");
    assert_eq!(
        u(&sim, "done"),
        1,
        "wait fork must not block on a disabled child"
    );
}
