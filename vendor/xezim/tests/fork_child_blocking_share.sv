//----------------------------------------------------------------------
// Regression test: a fork...join_none child's write to a TASK-LOCAL
// automatic variable must be LIVE-SHARED with the suspended parent —
// visible to it immediately, not only when the child finishes.
//
// This is the exact idiom UVM 1800.2-2020.3.1 uses in the sequencer
// watchdog `uvm_sequencer_param_base::m_safe_select_item`:
//
//     process select_process;                 // task-local automatic
//     fork
//        begin
//           select_process = process::self(); // child writes parent local
//           forever begin ... process_id.await(); ... end   // child blocks
//        end
//     join_none
//     wait (select_process != null);          // parent waits on the write
//
// The child sets `select_process` and then BLOCKS FOREVER in its `await`
// loop — so a merge-on-child-completion model can never deliver the value
// to the parent (the child never completes). The parent's `wait(... != null)`
// parks forever unless the write is shared live. Without the fix this stalls
// the sequencer rendezvous and the run-phase objection never drops.
//
// This test distills the idiom: the child writes a task-local automatic and
// then spins in a `forever #1` loop (never finishing), while the parent waits
// on the write.
//
//   xezim --simulate -s top fork_child_blocking_share.sv
//
// Before fix: parent parks forever → watchdog prints FAIL at t=1000.
// After fix:  child's write is live-shared → parent wakes → prints PASS.
//----------------------------------------------------------------------

module top;

   bit done;

   task automatic handshake();
      int local_flag;          // TASK-LOCAL (not a module signal!)
      begin
         local_flag = 0;

         fork
            begin
               local_flag = 42;    // child writes the parent's local
               $display("[C] child wrote local_flag = %0d", local_flag);
               forever begin
                  #1;              // child blocks forever (UVM watchdog)
               end
            end
         join_none

         // Parent waits for the child's (never-finishing) write. Requires
         // truthful live-sharing. Before the fix this deadlocks.
         wait(local_flag != 0);
         $display("[P] parent woke — local_flag = %0d", local_flag);
         done = 1;
      end
   endtask

   initial begin
      handshake();
      #2;
      $display("PASS: fork-child live-shared write wakes a parked wait");
      $finish;
   end

   // Watchdog: if the wait deadlocks, fail visibly instead of hanging.
   initial begin
      #1000;
      $display("FAIL: wait(local_flag != 0) never woke — no live share");
      $finish;
   end

endmodule