//! class-perf Phase 4: the collection-return snapshot is DEFERRED at
//! `RetCollection` (only the flat `vm_ret_vals` channel is published) and
//! materialized lazily inside `assign_pending_ret_collection` only for the
//! destination path that cannot consume the flat fast channel directly.
//! These cover the two consumers:
//! * a destination that resolves to a writable base (local queue / dynamic
//!   array) uses the flat `vm_ret_vals` fast path — the deferred snapshot is
//!   never materialized;
//! * a destination that does NOT resolve (a module-scope named queue written
//!   from a method's collection return) forces the deferred cells to be
//!   built, then `copy_whole_queue` — exercising the lazy materialization.
//!
//! Values cross-checked byte-for-byte against the reference simulator.

use xezim::simulate;

#[test]
fn deferred_snapshot_fast_and_lazy_paths() {
    let _ = super::compiled_method_test_env::eager();
    let src = r#"module top;
typedef int q_t[$];

class store;
  typedef int q_t[$];
  protected int m[string];
  function q_t get(string k); if (m.exists(k)) return '{m[k]}; return '{}; endfunction
  function void put(string k, int x); m[k] = x; endfunction
endclass

int glob[$];            // module-scope queue: NOT a writable fast base

function automatic int lazy_dest();
  automatic store s = new;
  automatic int q[$];
  s.put("k", 42);
  q = s.get("k");       // fast channel (local collection dest)
  glob = s.get("k");    // forces lazy snapshot -> copy_whole_queue
  return q.size() + glob.size() + glob[0] - 42;
endfunction

initial begin
  $display("L %0d", lazy_dest());  // 1 + 1 + 42 - 42 = 2
  $display("G %0d %0d", glob.size(), glob[0]);
  $display("END");
end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["L 2", "G 1 42", "END"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
