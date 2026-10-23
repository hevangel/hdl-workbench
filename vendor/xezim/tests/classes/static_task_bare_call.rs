//! §8.10, §13.3: inside a static method a bare call names a method of the
//! enclosing class. A blocking static task reached that way must suspend the
//! calling process like any task call (#313): its mailbox `get`, `#`, `@`,
//! `wait`, semaphore `get` and `fork...join` used to return at once. The
//! neighbours cover `Cls::` calls, nested chains, loops, inherited and
//! parameterized statics, static-to-instance calls and a `$unit` task the
//! class method shadows. Expected values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    let mut v: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    v.sort();
    v
}

const SRC: &str = r#"// #313 neighbours: bare calls between static/non-static tasks that block.
task automatic dup_t(output int r); #1 r = 111; endtask   // $unit task, shadowed in class
package pk;
  task automatic ptask(output int r); #4 r = 44; endtask
  class PC;
    static task run(); int v; ptask(v); $display("T|L pkg-bare t=%0t v=%0d", $time, v); endtask
  endclass
endpackage

class A;  // mailbox, bare + Cls:: + this-less
  static mailbox #(int) mb = new();
  static task it(output int v); mb.get(v); endtask
  static task bare(); int v; it(v); $display("T|A1 bare-mbx t=%0t v=%0d", $time, v); endtask
  static task scoped(); int v; A::it(v); $display("T|A2 scoped-mbx t=%0t v=%0d", $time, v); endtask
endclass
class B;  // delay / event / wait / semaphore
  static event ev; static int flag; static semaphore sem = new(0);
  static task d(); #5; endtask
  static task e(); @ev; endtask
  static task w(); wait (flag == 1); endtask
  static task s(); sem.get(1); endtask
  static task rd(); d(); $display("T|B1 delay t=%0t", $time); endtask
  static task re(); e(); $display("T|B2 event t=%0t", $time); endtask
  static task rw(); w(); $display("T|B3 wait t=%0t", $time); endtask
  static task rs(); s(); $display("T|B4 sem t=%0t", $time); endtask
endclass
class C;  // nested bare chain, fork/join, loop
  static task c3(); #2; endtask
  static task c2(); c3(); c3(); endtask
  static task c1(); c2(); $display("T|C1 nested t=%0t", $time); endtask
  static task fj(); fork #3; #6; join $display("T|C2 forkjoin t=%0t", $time); endtask
  static task fjr(); fj(); $display("T|C3 after-fj t=%0t", $time); endtask
  static task step(); #3; endtask
  static task loop(); repeat (3) step(); $display("T|C4 loop t=%0t", $time); endtask
endclass
class D;  // static -> non-static through a handle, non-static -> static bare
  int id;
  task inst_t(); #7; $display("T|D1 inst t=%0t id=%0d", $time, id); endtask
  static task st(); D h = new(); h.id = 9; h.inst_t(); $display("T|D2 st-after t=%0t", $time); endtask
  static task sdel(); #4; endtask
  task ns(); sdel(); $display("T|D3 nonstatic->static t=%0t", $time); endtask
endclass
class P #(int W = 2);  // parameterized
  static task pw(); #(W); endtask
  static task run(); pw(); $display("T|P1 param W=%0d t=%0t", W, $time); endtask
endclass
class Base; static task bt(); #6; endtask endclass
class Der extends Base;
  static task run(); bt(); $display("T|E1 inherited t=%0t", $time); endtask
endclass
class F;  // function vs task, shadowing of a $unit task
  static function int f(int x); return x * 2; endfunction
  static task tf(); int r; r = f(21); #1; $display("T|F1 func t=%0t r=%0d", $time, r); endtask
  static task dup_t(output int r); #8 r = 222; endtask
  static task sh(); int r; dup_t(r); $display("T|F2 shadow t=%0t r=%0d", $time, r); endtask
endclass

module tb;
  initial begin
    fork
      A::bare(); A::scoped(); B::rd(); B::re(); B::rw(); B::rs();
      C::c1(); C::fjr(); C::loop(); D::st();
      begin D h = new(); h.ns(); end
      P#(5)::run(); Der::run(); F::tf(); F::sh(); pk::PC::run();
    join_none
    #10 A::mb.put(42); A::mb.put(43);
    #2 -> B::ev;
    #3 B::flag = 1;
    #5 B::sem.put(1);
    #30 $display("T|END t=%0t", $time);
    $finish;
  end
endmodule
"#;

#[test]
fn static_task_bare_calls_block() {
    assert_eq!(
        t_lines(SRC),
        [
            "T|A1 bare-mbx t=10 v=42",
            "T|A2 scoped-mbx t=10 v=43",
            "T|B1 delay t=5",
            "T|B2 event t=12",
            "T|B3 wait t=15",
            "T|B4 sem t=20",
            "T|C1 nested t=4",
            "T|C2 forkjoin t=6",
            "T|C3 after-fj t=6",
            "T|C4 loop t=9",
            "T|D1 inst t=7 id=9",
            "T|D2 st-after t=7",
            "T|D3 nonstatic->static t=4",
            "T|E1 inherited t=6",
            "T|END t=50",
            "T|F1 func t=1 r=42",
            "T|F2 shadow t=8 r=222",
            "T|L pkg-bare t=4 v=44",
            "T|P1 param W=5 t=5",
        ]
    );
}
