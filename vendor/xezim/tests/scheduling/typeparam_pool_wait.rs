//! §8.20 + §6.20.3: constructing a class with a TYPE-PARAMETER-typed
//! assoc-array pool, and parking a `forever @` waiter on an element that the
//! pool lazily `new`s (UVM's `uvm_pool::get` → `uvm_object_string_pool#(T)`).
//!
//! A recorder-style pool use hangs at 100% CPU when the pool is built with
//! `T` bound to its DEFAULT (`uvm_object`) instead of the concrete element
//! (`uvm_event#(uvm_object)`): `pool[key] = new()`
//! then creates a plain `uvm_object` (no event member to park on), so the
//! `forever @(ev.m_event)` waiter never suspends and spins at time 0.
//!
//! Mirrors the pool's `get`/`wait_trigger`/`trigger` contract in miniature:
//! `pool.get(key)` caches a freshly-`new`ed element on first request, and a
//! `forever` waiter parked on that element wakes exactly once per trigger.

use xezim::simulate;

fn read_int(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn typeparam_pool_get_builds_and_parks_waiter() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  int wakes = 0;
  int null_flag = 0;

  // generic pool keyed by string; element type = the class TYPE PARAMETER
  class pool_t #(type T);
    T items[string];
    function T get(string k);
      if (!items.exists(k))
        items[k] = new;
      return items[k];
    endfunction
    function new; endfunction
  endclass

  // element whose waiter parks on a nested event member
  class ev_t;
    event mev;
    function void trigger; ->mev; endfunction
    task wait_t;
      @mev;
    endtask
  endclass

  pool_t #(ev_t) pool;
  ev_t b;

  initial begin
    pool = new;
    b = pool.get("b");
    null_flag = (b == null) ? 1 : 0;
    fork
      begin
        forever begin
          b.wait_t;
          wakes++;
        end
      end
      begin
        #5 b.trigger;
        #5 b.trigger;
      end
    join_none
    #30;
    $display("wakes=%0d null=%0d", wakes, null_flag);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 2000).expect("simulate failed");
    assert_eq!(
        read_int(&sim, "null_flag"),
        0,
        "pool.get must return a non-null element"
    );
    assert_eq!(
        read_int(&sim, "wakes"),
        2,
        "each ->mev wakes the forever waiter exactly once"
    );
}
