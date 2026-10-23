
class runner;

  // Like uvm_phase::m_phase_hopper.
  static mailbox #(int) hopper = new();

  // Bookkeeping for the self-check.
  static int got_count;
  static int first_val;
  static int second_val;

  // Like uvm_phase::m_iterate_through_phases(): a STATIC task whose body
  // blocks on a mailbox.
  static task iterate();
    int phase;
    repeat (2) begin
      hopper.get(phase);                 // must block while box is empty
      if (got_count == 0) first_val = phase;
      else                second_val = phase;
      got_count++;
      $display("[t=%0t] execute_phase(%0d)", $time, phase);
    end
  endtask

  // Like uvm_phase::m_run_phases(): a STATIC task that calls the blocking
  // static task of the SAME class by its BARE name (no receiver, no `this`).
  static task run_phases();
    iterate();                           // <-- bare static-task call
    $display("[t=%0t] phase runner finished", $time);
  endtask

  static function string result();
    if (got_count == 2 && first_val == 42 && second_val == 43)
      return "TEST PASSED";
    return $sformatf("TEST FAILED: got_count=%0d first_val=%0d second_val=%0d (expected 2, 42, 43)",
                     got_count, first_val, second_val);
  endfunction

endclass

module tb;

  // Like uvm_root::run_test(): fork the phase runner, let it start.
  initial begin : run_test_proc
    fork begin
      runner::run_phases();              // like uvm_phase::m_run_phases()
    end
    join_none
    #0;                                  // let the phase runner start
  end

  // Stimulus: feed the hopper at t=10 and t=20, then self-check.
  initial begin : stimulus_proc
    #10 runner::hopper.put(42);
    #10 runner::hopper.put(43);
    #5;
    $display("%s", runner::result());
    $finish;
  end

endmodule

