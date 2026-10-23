//! A class task compiles to bytecode only when nothing it runs can suspend
//! the process (IEEE 1800-2023 §9.4 timing controls, §9.3.2 fork/join,
//! §15.3/§15.4 semaphore and mailbox waits, §13 task callees doing any of
//! these). The analysis fails closed: a callee reached through an
//! interface-class handle or a mailbox handle, a `randsequence` production,
//! or a call cycle that reaches a delay keeps the task on the interpreter,
//! which suspends it correctly.

use std::process::Command;
use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

fn all_policies() -> bool {
    super::compiled_method_test_env::policies(&[("1", "0"), ("1", "1000"), ("0", "1000")])
}

#[test]
fn interface_class_callee_that_waits_keeps_task_interpreted() {
    if !all_policies() {
        return;
    }
    let src = r#"
module top;
  event ev; bit flag;
  interface class I;
    pure virtual task t();
  endclass
  class OnEv implements I;
    virtual task t(); @ev; endtask
  endclass
  class OnFlag implements I;
    virtual task t(); wait (flag); endtask
  endclass
  class OnMbx implements I;
    mailbox #(int) m; int x;
    virtual task t(); m.get(x); endtask
  endclass
  class Runner;
    I h; int n;
    task go(); h.t(); n++; $display("T|go done at %0t n=%0d", $time, n); endtask
  endclass
  Runner r1, r2, r3; OnEv a; OnFlag b; OnMbx c;
  initial begin
    r1 = new(); r2 = new(); r3 = new(); a = new(); b = new(); c = new(); c.m = new();
    r1.h = a; r2.h = b; r3.h = c;
    fork r1.go(); r2.go(); r3.go(); join_none
    #3 ->ev;
    #3 flag = 1;
    #3 c.m.put(5);
    #3 $display("T|end at %0t x=%0d", $time, c.x);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|go done at 3 n=1",
            "T|go done at 6 n=1",
            "T|go done at 9 n=1",
            "T|end at 12 x=5"
        ]
    );
}

#[test]
fn mailbox_wait_in_helper_with_user_get_method_keeps_task_interpreted() {
    if !all_policies() {
        return;
    }
    let src = r#"
module top;
  class Other;
    function int get(); return 1; endfunction
    function int peek(); return 2; endfunction
  endclass
  class C;
    mailbox #(int) m;
    int x, y;
    task helper(); m.get(x); endtask
    task run(); helper(); y = x + 1; $display("T|run x=%0d y=%0d at %0t", x, y, $time); endtask
    task helper2(); m.peek(x); endtask
    task run2(); helper2(); $display("T|run2 x=%0d at %0t", x, $time); endtask
  endclass
  C c; Other o;
  initial begin
    c = new(); o = new(); c.m = new();
    fork c.run(); join_none
    #5 c.m.put(7);
    #1 fork c.run2(); join_none
    #5 c.m.put(8);
    #5 $display("T|end %0d", o.get());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|run x=7 y=8 at 5", "T|run2 x=8 at 11", "T|end 1"]
    );
}

const CYCLE_AND_RANDSEQ: &str = r#"
module top;
  class C;
    int depth, n;
    task e(); a(); endtask
    task a(); b(); #1; endtask
    task b(); nn(); endtask
    task nn(); if (depth < 1) begin depth++; a(); end endtask
    task m(); b(); $display("T|m done at %0t depth=%0d", $time, depth); endtask
    task rs(); randsequence (main) main : first second; first : { #3; }; second : { n += 10; }; endsequence endtask
    task go(); rs(); n++; $display("T|go done at %0t n=%0d", $time, n); endtask
  endclass
  C c;
  initial begin
    c = new();
    c.e();
    $display("T|e done at %0t", $time);
    c.depth = 0;
    c.m();
    c.go();
    $display("T|end at %0t", $time);
  end
endmodule
"#;

#[test]
fn call_cycle_and_randsequence_waits_are_seen() {
    if !all_policies() {
        return;
    }
    assert_eq!(
        t_lines(CYCLE_AND_RANDSEQ),
        [
            "T|e done at 2",
            "T|m done at 3 depth=1",
            "T|go done at 6 n=11",
            "T|end at 6"
        ]
    );
}

/// The verdicts themselves: every task above reaches a delay, so none may be
/// planned for compilation, whatever order the analysis meets them in (`e`
/// first walks the `a -> b -> nn -> a` cycle; `b` and `m` must not keep the
/// tentative "wait-free" assumed for `a` inside it).
#[test]
fn tasks_reaching_a_delay_are_declined() {
    let d = std::env::temp_dir().join(format!("xezim_task_wait_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("top.sv"), CYCLE_AND_RANDSEQ).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top", "top.sv"])
        .current_dir(&d)
        .env("XEZIM_COMPILE_METHODS", "1")
        .env("XEZIM_METHOD_TIER", "0")
        .env("XEZIM_FALLBACK_SITES", "1")
        .env_remove("XEZIM_METHOD_CACHE")
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&d);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{text}");
    for task in ["e", "a", "b", "nn", "m", "rs", "go"] {
        assert!(
            text.contains(&format!(
                "plan-decline reason=task-suspends scope=C.{task}\n"
            )),
            "C.{task} was not declined:\n{text}"
        );
    }
}

/// The interpreter's own "does this call block" memo had the same cycle
/// flaw: after `go_a` walked `a -> b -> nn -> a`, `nn` stayed cached as
/// non-blocking, so `go_n` ran it synchronously and its `@ev` did not wait.
#[test]
fn call_cycle_reaching_an_event_wait_suspends_the_caller() {
    if !all_policies() {
        return;
    }
    let src = r#"
class C;
  event ev; int k, n;
  task a(); b(); @ev; endtask
  task b(); nn(); endtask
  task nn(); if (k > 0) begin k--; a(); end endtask
  task go_a(); a(); $display("T|a done at %0t", $time); endtask
  task go_n(); k = 1; nn(); n++; $display("T|nn done n=%0d at %0t", n, $time); endtask
endclass
module top;
  C c = new;
  initial begin
    c.go_a();
    c.go_n();
    $display("T|end at %0t", $time);
    $finish;
  end
  always #3 ->c.ev;
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|a done at 3", "T|nn done n=1 at 6", "T|end at 6"]
    );
}
