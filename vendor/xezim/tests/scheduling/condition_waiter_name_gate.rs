//! A parked `wait(cond)` inside a class method is re-checked only when a
//! name its condition reads is written (a property of its object, a static,
//! a property reached through a local handle, a local a fork child writes);
//! `wait(0)` is never re-checked. Every store write re-armed every such
//! waiter before, so a clock-counting property update each cycle resumed all
//! of UVM's housekeeping waits every cycle. Each waiter must still wake at
//! the write that makes its condition true. The expected lines were
//! cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class ph;
  bit m_premature_end;
  int n;
endclass
class hopper;
  int m_queue[$];
  int m_arb_size, m_lock_arb_size;
  static bit s_go;
  int busy;
  task get(); wait (m_queue.size() != 0); $display("%0t get %0d", $time, m_queue[0]); endtask
  task arb(); wait (m_arb_size != m_lock_arb_size); $display("%0t arb", $time); endtask
  task never(); wait (0); $display("%0t never woke", $time); endtask
  task jump(ph phase); wait (phase.m_premature_end); $display("%0t jump n=%0d", $time, phase.n); endtask
  task stat(); wait (s_go); $display("%0t static", $time); endtask
  task local_flag();
    bit done;
    fork
      begin #8 done = 1; end
    join_none
    wait (done);
    $display("%0t local", $time);
  endtask
endclass
module top;
  logic clk = 0;
  always #1 clk = ~clk;
  hopper h; ph p;
  initial begin
    h = new; p = new;
    fork h.get(); h.arb(); h.never(); h.jump(p); h.stat(); h.local_flag(); join_none
    repeat (3) @(posedge clk) h.busy++;
    p.n = 4;
    #2 p.m_premature_end = 1;
    #2 h.m_arb_size = 3;
    #2 h.m_queue.push_back(9);
    #2 hopper::s_go = 1;
    #20 $display("%0t end busy=%0d", $time, h.busy);
    $finish;
  end
endmodule
"#;

#[test]
fn gated_condition_waiters_wake_on_their_writes() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = [
        "7 jump n=4",
        "8 local",
        "9 arb",
        "11 get 9",
        "13 static",
        "33 end busy=3",
    ];
    assert_eq!(out, want, "{out:?}");
}
