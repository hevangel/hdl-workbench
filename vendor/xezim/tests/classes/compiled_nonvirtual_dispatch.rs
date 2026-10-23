//! IEEE 1800-2023 §8.20: a non-virtual method called through a class handle
//! binds to the handle's DECLARED class, not the object's class. Compiled
//! callers (a wait-free task from its first call, a function past the call
//! tier, a static task) used to dispatch on the object's dynamic class, so
//! `h.bump()` with `A h` holding a `B` ran `B::bump`.

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
fn nonvirtual_call_in_compiled_caller_uses_declared_class() {
    if !all_policies() {
        return;
    }
    let src = r#"
class A;
  int v;
  function void bump(); v += 1; endfunction
  task automatic tbump(); v += 1; endtask
endclass
class B extends A;
  function void bump(); v += 100; endfunction
  task automatic tbump(); v += 100; endtask
endclass
class Drv;
  task run_t(A h); h.tbump(); endtask
  task run_f(A h); h.bump(); endtask
  function void frun_f(A h); h.bump(); endfunction
endclass
module top;
  int r1, r2, r3;
  initial begin
    B b = new; A ab = b; Drv d = new;
    for (int i = 0; i < 3; i++) d.run_t(ab);
    r1 = b.v; b.v = 0;
    for (int i = 0; i < 3; i++) d.run_f(ab);
    r2 = b.v; b.v = 0;
    for (int i = 0; i < 1500; i++) d.frun_f(ab);
    r3 = b.v;
    $display("T|run_t=%0d run_f=%0d frun_f=%0d", r1, r2, r3);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|run_t=3 run_f=3 frun_f=1500"]);
}

#[test]
fn nonvirtual_task_through_static_task_and_virtual_callee() {
    if !all_policies() {
        return;
    }
    let src = r#"
class A;
  int v;
  virtual function int val(); return 1; endfunction
  task automatic put_v(); v += val(); endtask
  static task automatic st(A h); h.put_v(); endtask
endclass
class B extends A;
  virtual function int val(); return 10; endfunction
  task automatic put_v(); v += 100 * val(); endtask
endclass
module top;
  initial begin
    A a = new; B b = new; A ab = b;
    for (int i = 0; i < 1500; i++) begin a.put_v(); ab.put_v(); b.put_v(); A::st(ab); end
    $display("T|a.v=%0d b.v=%0d", a.v, b.v);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|a.v=1500 b.v=1530000"]);
}
