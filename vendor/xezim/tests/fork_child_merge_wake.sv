//----------------------------------------------------------------------
// Regression test: a fork...join_none child's write to the parent's
// TASK-LOCAL automatic is delivered back to the suspended parent via the
// fork-child merge path, not by a synchronous live-share, and must wake the
// parent's level-sensitive `wait` (§9.4.2).
//
// This mirrors the UVM sequencer `m_safe_select_item` rendezvous:
//
//     process select_process;
//     fork
//        begin
//           select_process = process::self();  // child writes parent local
//           forever begin ... process_id.await(); ... end
//        end
//     join_none
//     wait (select_process != null);   // parent parks on the child's write
//
// `handshake`: a #0 deferral between the fork and the write guarantees the
// parent runs on to `wait(...)` first.
//
// `edge_handshake`: the child is woken by a clock edge, so its write lands
// inside that edge's cascade. The parent must resume in the same active
// region, before the edge's NBA commits (it samples `q`, which a flop
// increments with `<=` on the same edge). Without the merge's wake the
// parent only resumed after the NBA region and saw the incremented value.
//
//   xezim --simulate -s top fork_child_merge_wake.sv
//   Reference simulator: PASS result=1 seen=0,1
//----------------------------------------------------------------------

module top;

   int result;
   logic clk = 0;
   int q = 0;

   always #5 clk = ~clk;
   always @(posedge clk) q <= q + 1;

   task automatic handshake();
      int local_flag;           // TASK-LOCAL (not a module signal!)
      begin
         local_flag = 0;

         fork
            begin
               #0;              // defer so the parent parks first
               local_flag = 1;  // then write the parent's automatic
            end
         join_none

         // Parent parks until the child's deferred write is propagated back
         // into this activation and the level-sensitive wait re-checks it.
         wait(local_flag != 0);
         result = result + 1;
      end
   endtask

   task automatic edge_handshake(output int seen);
      int flag = 0;
      fork
         begin
            @(posedge clk);
            flag = 1;           // written inside the edge's cascade
         end
      join_none
      wait (flag != 0);
      seen = q;                 // before this edge's `q <= q + 1` commits
   endtask

   initial begin
      int s1, s2;
      handshake();
      edge_handshake(s1);
      edge_handshake(s2);
      $display("PASS result=%0d seen=%0d,%0d", result, s1, s2);
      $finish;
   end

   // Watchdog: if a wait never wakes, fail visibly instead of hanging.
   initial begin
      #2000;
      $display("FAIL: a wait on a fork child's write never woke");
      $finish;
   end

endmodule
