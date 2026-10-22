//! §9.3.2/§26.3: a forked call to a blocking PACKAGE task (`pkg::t()`) runs
//! as its own process and suspends at its delays like any other task call.
//! The process runner inlined blocking tasks only for single-segment names,
//! so `pkg::t()` took the synchronous path, whose nested delay loop never
//! returned for a `forever` body: the parent's `#25 $finish` never ran and
//! the simulation went on to the time limit — from a class function that
//! forks (the UVM HDL-polling runner) or straight from an `initial` block.
//! Expected output cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
package pk;
  task automatic runner(input bit b = 0);
    forever #10 $display("tick %0t", $time);
  endtask
endpackage
module tb;
  class C;
    function bit create();
      fork
        pk::runner(0);
      join_none
      return 1;
    endfunction
  endclass
  initial begin
    C o;
    o = new;
    void'(o.create());
    fork
      pk::runner();
    join_none
    #25 $display("parent %0t", $time);
    $finish;
  end
endmodule
"#;

#[test]
fn forked_package_task_forever_lets_parent_finish() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(
        msgs,
        ["tick 10", "tick 10", "tick 20", "tick 20", "parent 25"],
    );
    assert_eq!(sim.time, 25);
}
