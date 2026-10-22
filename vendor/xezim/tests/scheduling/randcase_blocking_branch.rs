//! §18.16: a `randcase` branch that blocks (a task with `fork ... join` and
//! `wait`, e.g. a UVM `seq.start(sqr)`) must suspend the process. The branch
//! ran on the synchronous path, returned at its first wait, and the loop
//! around the randcase started the next call while the first was still
//! running (UVM: "Sequence ... already started" at time 0). Cross-checked
//! against the reference simulator.

use xezim::simulate;

#[test]
fn randcase_branch_task_suspends() {
    let src = r#"
module tb;
  int busy = 0, overlap = 0, done = 0;
  bit go = 0;
  always #3 go = ~go;
  task automatic work();
    if (busy) overlap++;
    busy = 1;
    fork
      begin #5; end
    join
    wait (go);
    busy = 0;
    done++;
  endtask
  initial begin
    repeat (6) begin
      randcase
        1: work();
        1: work();
      endcase
    end
    $display("t=%0t done=%0d overlap=%0d", $time, done, overlap);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 1_000).expect("simulate failed");
    let out: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(out, ["t=33 done=6 overlap=0"]);
}
