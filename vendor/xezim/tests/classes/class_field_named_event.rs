// Regression test: class-field named events (`event m_event` declared inside
// a class) must block on `@field`/`@(field)` and wake on `->field` when both
// appear inside methods on the same `this`.
//
// Before the fix, a class `event` field had no backing signal in
// `signal_table`, so `@m_event` fell through to the delta-yield (NBA) branch
// and returned IMMEDIATELY at time 0 — it never blocked. This broke
// `uvm_event#(T)::wait_trigger()` / `uvm_event::trigger()` (the basis of
// `uvm_heartbeat` synchronization): a `start()`ed heartbeat died at t=0 after
// one spurious check, instead of monitoring across the event's trigger
// schedule.
//
// The fix adds a per-instance named-event waiter list (`instance_event_waiters`)
// keyed by `(this_handle, field_name)`. Both the `@field` park path and the
// `->field` trigger path resolve the same `(this, field)` identity, so a wait
// and a trigger inside methods on the same object match.
//
// Verified byte-for-byte against reference simulators.

use std::process::Command;

fn xezim() -> String {
    env!("CARGO_BIN_EXE_xezim").to_string()
}

fn run(src: &str, tag: &str) -> String {
    let path = format!("/tmp/cfev_{tag}.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(xezim())
        .args(["--simulate", "-s", "top", &path])
        .output()
        .expect("xezim failed to start");
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// A bare `@ev; ->ev;` pair inside class methods must block then resume at
/// the trigger time (not return immediately at t=0).
#[test]
fn class_field_event_blocks_and_resumes() {
    let out = run(
        r#"module top;
  class C;
    event ev;
    int woke_at;
    task t_wait();
      @ev;
      woke_at = $time;
    endtask
    task t_trigger();
      ->ev;
    endtask
  endclass
  C c;
  initial begin
    c = new();
    fork c.t_wait(); join_none
    #50;
    c.t_trigger();
    #10;
    if (c.woke_at == 50) $display("TAG_PASS");
    else $display("TAG_FAIL woke_at=%0d", c.woke_at);
    $finish;
  end
endmodule
"#,
        "basic",
    );
    assert!(
        out.contains("TAG_PASS"),
        "class-field @ev did not block until ->ev fired\n{}",
        out
    );
}

/// The parenthesized form `@(ev)` (parses as EventExpr, not Identifier) must
/// also block.  Without this the abort branch of uvm_heartbeat's
/// `fork ... join_any` returned immediately and fired `disable fork`,
/// killing the monitoring loop.
#[test]
fn class_field_event_paren_form_blocks() {
    let out = run(
        r#"module top;
  class C;
    event ev;
    int woke_at;
    task t_wait();
      @(ev);
      woke_at = $time;
    endtask
    task t_trigger();
      ->ev;
    endtask
  endclass
  C c;
  initial begin
    c = new();
    fork c.t_wait(); join_none
    #50;
    c.t_trigger();
    #10;
    if (c.woke_at == 50) $display("TAG_PASS");
    else $display("TAG_FAIL woke_at=%0d", c.woke_at);
    $finish;
  end
endmodule
"#,
        "paren",
    );
    assert!(
        out.contains("TAG_PASS"),
        "class-field @(ev) did not block\n{}",
        out
    );
}

/// A process parked on `@field` inside a `fork ... join_any` must count as
/// suspended (not finished), so the join does not fire prematurely and
/// `disable fork` does not kill it.  This mirrors uvm_heartbeat::m_hb_process
/// whose loop branch waits on `@m_event` while the abort branch waits on
/// `@m_stop_event`.
#[test]
fn join_any_sibling_does_not_kill_event_waiter() {
    let out = run(
        r#"module top;
  class C;
    event ev_loop;
    event ev_abort;
    int checks;
    task run();
      fork
        begin : loop
          for (int i = 0; i < 4; i++) begin
            @ev_loop;
            checks++;
          end
        end
        begin : abort
          @ev_abort;
        end
      join_any
      disable fork;
    endtask
    task fire_loop();  ->ev_loop; endtask
    task fire_abort(); ->ev_abort; endtask
  endclass
  C c;
  initial begin
    c = new();
    fork c.run(); join_none
    repeat(4) begin #10; c.fire_loop(); end
    #5;
    c.fire_abort();
    #1;
    if (c.checks == 4) $display("TAG_PASS");
    else $display("TAG_FAIL checks=%0d", c.checks);
    $finish;
  end
endmodule
"#,
        "joinany",
    );
    assert!(
        out.contains("TAG_PASS"),
        "join_any sibling killed the @field waiter prematurely\n{}",
        out
    );
}
