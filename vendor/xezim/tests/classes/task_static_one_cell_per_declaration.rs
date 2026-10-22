//! IEEE 1800-2023 §13.3.2: a `static` variable of a task or function is one
//! variable per declaration, shared by every activation "regardless of the
//! number of concurrent activations". A package or module task called from a
//! class method used to key its statics by the CALLING class, so each class
//! got its own copy.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// Package and module tasks and a package function with a `static` counter,
/// called from two classes and from module code: one counter each.
#[test]
fn subroutine_static_shared_by_all_callers() {
    let src = r#"
package pk;
  task automatic ptask(output int r);
    static int cnt = 0;
    cnt++;
    r = cnt;
  endtask
  task automatic ptask_wait(output int r);
    static int cnt = 0;
    #1;
    cnt++;
    r = cnt;
  endtask
  function automatic int pfunc();
    static int cnt = 0;
    cnt++;
    return cnt;
  endfunction
endpackage
module top;
  import pk::*;
  task automatic mtask(output int r);
    static int cnt = 0;
    cnt++;
    r = cnt;
  endtask
  class A;
    task run(); int r; ptask(r); $display("T|A ptask=%0d", r);
      ptask_wait(r); $display("T|A ptask_wait=%0d", r);
      mtask(r); $display("T|A mtask=%0d", r);
      $display("T|A pfunc=%0d", pfunc()); endtask
  endclass
  class B;
    task run(); int r; ptask(r); $display("T|B ptask=%0d", r);
      ptask_wait(r); $display("T|B ptask_wait=%0d", r);
      mtask(r); $display("T|B mtask=%0d", r);
      $display("T|B pfunc=%0d", pfunc()); endtask
  endclass
  initial begin
    A a = new; B b = new; int r;
    a.run();
    b.run();
    ptask(r); $display("T|mod ptask=%0d", r);
    ptask_wait(r); $display("T|mod ptask_wait=%0d", r);
    mtask(r); $display("T|mod mtask=%0d", r);
    $display("T|mod pfunc=%0d", pfunc());
    a.run();
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|A ptask=1",
            "T|A ptask_wait=1",
            "T|A mtask=1",
            "T|A pfunc=1",
            "T|B ptask=2",
            "T|B ptask_wait=2",
            "T|B mtask=2",
            "T|B pfunc=2",
            "T|mod ptask=3",
            "T|mod ptask_wait=3",
            "T|mod mtask=3",
            "T|mod pfunc=3",
            "T|A ptask=4",
            "T|A ptask_wait=4",
            "T|A mtask=4",
            "T|A pfunc=4",
        ]
    );
}

/// A `wait` barrier on a `static` task variable, released from one class and
/// awaited from another (UVM's `uvm_wait_for_nba_region` shape).
#[test]
fn static_task_barrier_shared_across_classes() {
    let src = r#"
package pk;
  task automatic barrier(int who);
    static int flag = 0;
    if (who == 0) begin
      #5 flag = 1;
    end else begin
      wait (flag == 1);
    end
    $display("T|barrier who=%0d passed at %0t", who, $time);
  endtask
endpackage
module top;
  import pk::*;
  class A; task run(); barrier(0); endtask endclass
  class B; task run(); barrier(1); endtask endclass
  initial begin
    A a = new; B b = new;
    fork a.run(); b.run(); join
    $display("T|done %0t", $time);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|barrier who=0 passed at 5",
            "T|barrier who=1 passed at 5",
            "T|done 5",
        ]
    );
}
