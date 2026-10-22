`timescale 1ns/1ps
// Pure-SystemVerilog regression for a FORWARD-DECLARED class used as a
// static-method formal whose class-handle MEMBER is written inside a fork.
//
// `typedef class FWD;` is a forward declaration (`class_ref == Some(true)`),
// so `FWD` is ALSO registered as a typedef placeholder. A static method's
// formal of that type (`spawnit(FWD guard)`) must still be classified as a
// CLASS OBJECT — the handle's member write (`guard.m_guard_process = selfp`)
// must land on the heap object, so a later `clear()` (on the same object via
// a different call path) observes the watcher and can kill it.
//
// This is the `uvm_process_guard_base::m_process_guard` scenario: if the
// formal is misclassified as a plain typedef the member write is silently
// dropped in the forked child context, the guard keeps a null watcher, and a
// downstream sequence-cancel storms on (the SEQPRTZMB regression).

typedef class FWD;

class D;
  static function void spawnit(FWD guard);
    fork
      begin
        process selfp = process::self();
        guard.m_guard_process = selfp;   // class-handle member store
      end
    join_none
  endfunction
endclass : D

class FWD;
  process m_guard_process;
  function new();
    m_guard_process = null;
  endfunction
  function void clear();
    if (m_guard_process != null) begin
      m_guard_process.kill();
    end
  endfunction
endclass : FWD

module top;
  FWD g;
  initial begin
    g = new();
    D::spawnit(g);
    #1;
    g.clear();
    if (g.m_guard_process == null) $display("TAG_FAIL");
    else $display("TAG_PASS");
    $finish;
  end
endmodule