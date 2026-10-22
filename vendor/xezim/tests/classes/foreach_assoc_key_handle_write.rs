//! §12.7.3 + §7.8: `foreach` over a HANDLE-KEYED associative array binds
//! the loop variable to the KEY's CLASS; a class-property WRITE through that
//! loop variable (`succ.m_state = …`) must commit to the heap object, not be
//! silently dropped as a plain hierarchical write.
//!
//! This is the UVM phase-scheduler shape: `uvm_phase::execute_phase` walks
//! `protected bit m_successors[uvm_phase]` and marks each successor
//! `succ.m_state = UVM_PHASE_SCHEDULED` (UVM 1.2 uvm_phase.svh:1646-1661).
//! Two execution paths must both bind the loop var's class:
//!   * the plain interpreter path (`exec_stmt_foreach` §12.7.3 registration),
//!   * the ForeachTail TRAMPOLINE path — a loop whose body blocks (`#0`) is
//!     converted into a continuation that bypasses `exec_stmt_foreach`
//!     entirely; iterations after the first suspension re-enter through the
//!     flattened process stream, where the binding recorded at
//!     materialization time must survive the park/restore cycle
//!     (`snapshot_process_context` keeps `local_type_stack`).
//! Reference-validated byte-for-byte (reference simulator prints the same
//! T| lines).

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// Plain (non-blocking-body) foreach: each successor write commits.
#[test]
fn handle_key_member_write_commits() {
    let src = r#"
package P;
  typedef enum int unsigned { DORMANT = 1, SCHEDULED = 2, STARTED = 8 } pstate;
  class phase;
    pstate m_state;
    protected bit m_successors[phase];
    function new(); m_state = DORMANT; endfunction
    function void add_succ(phase s); m_successors[s] = 1; endfunction
    function void exec();
      foreach (m_successors[succ]) begin
        if (succ.m_state < SCHEDULED) begin
          succ.m_state = SCHEDULED;
          $display("T|sched|%0d", succ.m_state);
        end
      end
    endfunction
  endclass
endpackage

module top;
  import P::*;
  phase r, a, b;
  initial begin
    r = new; a = new; b = new;
    r.add_succ(a);
    r.add_succ(b);
    r.exec();
    if (a.m_state == SCHEDULED && b.m_state == SCHEDULED)
      $display("T|PASS");
    else
      $display("T|FAIL a=%0d b=%0d", a.m_state, b.m_state);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"T|sched|2".to_string()), "{o:?}");
    assert!(o.contains(&"T|PASS".to_string()), "{o:?}");
}

/// Blocking-body foreach (`#0` mid-body, the UVM 1.2 execute_phase shape):
/// the loop is unrolled through the ForeachTail trampoline after the first
/// suspension; the loop-var class binding recorded at materialization must
/// ride the process-context snapshot so EVERY iteration's write commits —
/// the write of iteration N happens after N-1 park/restore cycles.
#[test]
fn handle_key_member_write_blocking_body() {
    let src = r#"
package P;
  typedef enum int unsigned { DORMANT = 1, SCHEDULED = 2, STARTED = 8 } pstate;
  class phase;
    pstate m_state;
    protected bit m_successors[phase];
    function new(); m_state = DORMANT; endfunction
    function void add_succ(phase s); m_successors[s] = 1; endfunction
    task exec();
      foreach (m_successors[succ]) begin
        if (succ.m_state < SCHEDULED) begin
          succ.m_state = SCHEDULED;
          $display("T|sched|%0d", succ.m_state);
        end
        #0;
      end
    endtask
  endclass
endpackage

module top;
  import P::*;
  phase r, a, b, c;
  initial begin
    r = new; a = new; b = new; c = new;
    r.add_succ(a);
    r.add_succ(b);
    r.add_succ(c);
    fork r.exec(); join_none
    #5;
    if (a.m_state == SCHEDULED && b.m_state == SCHEDULED && c.m_state == SCHEDULED)
      $display("T|PASS");
    else
      $display("T|FAIL a=%0d b=%0d c=%0d", a.m_state, b.m_state, c.m_state);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert_eq!(
        o.iter().filter(|l| l.starts_with("T|sched|")).count(),
        3,
        "{o:?}"
    );
    assert!(o.contains(&"T|PASS".to_string()), "{o:?}");
}

/// The assoc array is declared in a BASE class; the foreach runs in the
/// derived class's task with a blocking body — the key-class lookup must
/// walk the `extends` chain (`ElaboratedClass::assoc_key_types` of an
/// ancestor), and the binding must survive the trampoline re-entries.
#[test]
fn handle_key_member_write_assoc_in_base_class() {
    let src = r#"
package P;
  typedef enum int unsigned { DORMANT = 1, SCHEDULED = 2, STARTED = 8 } pstate;
  class phase_base;
    pstate m_state;
    protected bit m_successors[phase_base];
    function new(); m_state = DORMANT; endfunction
    function void add_succ(phase_base s); m_successors[s] = 1; endfunction
  endclass

  class phase_ext extends phase_base;
    task exec();
      foreach (m_successors[succ]) begin
        if (succ.m_state < SCHEDULED) begin
          succ.m_state = SCHEDULED;
          $display("T|sched|%0d", succ.m_state);
        end
        #0;
      end
    endtask
  endclass
endpackage

module top;
  import P::*;
  phase_ext r, a;
  initial begin
    r = new; a = new;
    r.add_succ(a);
    fork r.exec(); join_none
    #5;
    if (a.m_state == SCHEDULED) $display("T|PASS");
    else $display("T|FAIL a=%0d", a.m_state);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"T|sched|2".to_string()), "{o:?}");
    assert!(o.contains(&"T|PASS".to_string()), "{o:?}");
}
