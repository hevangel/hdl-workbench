// IEEE 1800-2023 §15.5 — a blocking construct reached through a STATIC
// call on a PARAMETERIZED class must still SUSPEND the calling process.
//
// Regression for the 11waitmod-class hang: `C#(T)::wait_modified(...)` is a
// static blocking task whose receiver parses as a `Specialization` node
// (`C#(T)::m`). The suspend-aware inliner did not recognize that shape, so
// the call ran SYNCHRONOUSLY: its internal `@ev.trigger` never parked the
// calling process, the watch `while(1)` loop spun at time 0 and never saw
// the delayed config updates, and the simulation hung. The identical
// non-parameterized call suspends correctly.
//
// A watch loop mirrors UVM wait_modified: it registers a waiter and calls
// the STATIC PARAMETERIZED blocking task, which parks on the waiter's
// member event. A SEPARATE producer advances a SHARED value and fires the
// waiter after a delay (once per step). On the broken path nothing inside
// the loop suspends, so all iterations run at time 0 before the producer's
// first `#5`, `value` never reaches the sentinel, and the whole run spins.
// On the fixed path each `->w.trigger` releases exactly one parked
// iteration and `value == 3` is observed.
//
// Reference: run on a golden simulator; every case must report `pass`.

`timescale 1ns/1ns

class waiter_t;
   event trigger;
endclass

// Parameterized static blocking hop (the `uvm_config_db#(T)::wait_modified`
// shape). `C#(T)::wait_modified` must SUSPEND the calling process while the
// call parks on the waiter's member event.
class cfg_db #(type T = int);
   static task automatic wait_modified(waiter_t w);
      @(w.trigger);
   endtask
endclass

module param_static_park;

   int value = 0;          // shared "config" the watch loop reads
   int steps = 0;          // how many times the loop body completed
   int n_errors = 0;

   waiter_t w = new();

   task automatic chk(string what, int got, int exp);
      if (got !== exp) begin
         n_errors++;
         $display("  FAIL  %-30s got=%0d  exp=%0d", what, got, exp);
      end
   endtask

   // --- producer: advance the shared value, then fire the waiter ---------
   initial begin
      repeat (3) begin
         #5;                           // delay so the parked loop can wake
         value++;
         ->w.trigger;                  // release ONE parked iteration
      end
   end

   // --- consumer: while(1) body is the parameterized static call ---------
   task automatic watch_field();
      while (1) begin
         cfg_db#(bit)::wait_modified(w);   // must SUSPEND/block here
         steps++;                           // one per fired waiter
         if (value >= 3) break;             // sentinel reached
      end
   endtask

   initial begin
      fork
         watch_field();
      join_none
      // watchdog: if nothing suspends the loop spins hot at time 0
      #35;
      $display("TEST param_static_park");
      $display("  steps=%0d value=%0d", steps, value);
      chk("steps", steps, 3);            // exactly one wakeup per fire
      chk("value", value, 3);
      $display("TEST param_static_park: 2 checks, %0d errors -> %s",
               n_errors, (n_errors == 0) ? "PASS" : "FAIL");
      $finish;
   end
endmodule