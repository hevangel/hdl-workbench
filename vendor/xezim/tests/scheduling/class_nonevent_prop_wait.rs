//! GitHub #231: an event control `@prop` / `@(h.prop)` on a NON-event class
//! property is a value-change wait (§9.4.2). It used to be parked as if the
//! property were a named event, so only `->` could wake it and an ordinary
//! write never did. A bit property must react to a value change.
//! Wake times were validated against a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

const SRC: &str = r#"
`timescale 1ns/1ns
module tb;
  int t_task = -1, t_direct = -1, t_second = -1, t_same = -1, t_ev = -1, t_mod = -1;
  bit m;

  class change_cell;
    bit flag;
    int v;
    event pulse;
    task await_flag();  @flag;  endtask
    task await_pulse();  @pulse;  endtask
    task await_value();   @(v); endtask
    function void trigger(); flag = 1; endfunction
  endclass

  change_cell a = new();
  change_cell b = new();
  change_cell c = new();

  initial begin
    fork
      begin a.await_flag();  t_task   = $time; end
      begin @(b.flag);      t_direct = $time; end
      begin @(b.flag);      t_second = $time; end
      begin c.await_value();   t_same   = $time; end
      begin a.await_pulse();  t_ev     = $time; end
      begin @m;           t_mod    = $time; end
    join_none
    #2;
    a.trigger();
    b.flag = 1;
    m = 1;
    #1 c.v = 0;     // same value: must NOT wake @(v)
    #1 c.v = 5;     // change at t=4
    #1 ->a.pulse;      // named event still wakes on ->, at t=5
    #1 $finish;
  end
endmodule
"#;

#[test]
fn nonevent_property_wait_wakes_on_write() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    assert_eq!(
        u(&sim, "t_task"),
        2,
        "@flag in a task, written by a function"
    );
    assert_eq!(u(&sim, "t_direct"), 2, "@(b.flag), direct write");
    assert_eq!(u(&sim, "t_second"), 2, "second waiter on the same property");
    assert_eq!(u(&sim, "t_mod"), 2, "@m module variable (control)");
}

#[test]
fn nonevent_property_wait_ignores_same_value_write() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    assert_eq!(u(&sim, "t_same"), 4, "only a changed value wakes @(v)");
}

#[test]
fn named_event_property_still_wakes_on_trigger() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    assert_eq!(u(&sim, "t_ev"), 5, "@pulse on a real event property");
}
