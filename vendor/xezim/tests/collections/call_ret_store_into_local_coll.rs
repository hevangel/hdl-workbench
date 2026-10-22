//! class-perf: storing a collection-returning CALL into a bare local
//! collection inside a method / free-function body —
//! `q_local = cache.get(k)`, `da_local = obj.collect(i)`.
//!
//! Previously this shape declined the WHOLE body when compiled
//! (`blocking_target` in a method/free-fn body, whose lowering forbids the
//! AST fallback), so `uvm_re_match`'s cache branch fell to the AST
//! interpreter (~the dominant hot cost in the 7352_regex regression). Now
//! the compiler emits a dedicated `AssignPendingColl` insn that copies the
//! callee's `pending_ret_collection` snapshot whole into the named local
//! collection — a compiled store with no AST round-trip.
//!
//! Coverage (values cross-checked byte-for-byte against the reference
//! simulator):
//! * a class method returning a queue stored to a queue local (`q = s.get`);
//! * the empty-return `'{ }` yields size 0 / null-equivalent;
//! * a free-function wrapper (the uvm_re_match shape) stores the same way;
//! * a dynamic-array local collection receives the same call store;
//! * specialization: an adjacent non-collection local (`n = s.count()`)
//!   stays a normal scalar store (no accidental pending consumption).

use xezim::simulate;

#[test]
fn collection_ret_call_store_into_local_collection() {
    // Compiled from the first call (tier 0) in a child process, then again
    // here at the default tier.
    let _ = super::compiled_method_test_env::eager();
    let src = r#"module top;

class store;
  typedef int q_t[$];
  protected int m[string];

  function q_t get(string k); if (m.exists(k)) return '{m[k]}; return '{}; endfunction
  function int count(); return 3; endfunction
  function void put(string k, int x); m[k] = x; endfunction

  function int gather();
    // queue local gets the collection-returning call result
    automatic q_t q;
    automatic int n;
    q = this.get("a");
    n = q.size();
    // missing key -> empty return -> size 0
    q = get("nope");
    return n + q.size();
  endfunction
endclass

// free-function wrapper: the uvm_re_match shape (`cached = cache.get(re); ...`)
function automatic int wrap_queue();
  automatic store s = new;
  automatic store::q_t q;
  automatic int n;
  automatic int c;
  s.put("k", 42);
  q = s.get("k");
  n = q.size();
  q = s.get("absent");
  // a scalar local stays a normal store right after the coll stores
  c = s.count();
  return n + q.size() * 10 + c;
endfunction

function automatic int wrap_dyn();
  automatic store s = new;
  automatic int da[];
  automatic int n;
  s.put("k", 7);
  da = s.get("k");
  n = da.size();
  return n + da[0] + (da.size() == 0 ? 0 : da.size());
endfunction

initial begin
  automatic store s = new;
  s.put("a", 11); s.put("k", 5);
  $display("G %0d", s.gather());         // 1 (from "a") + 0 = 1
  $display("W %0d", wrap_queue());      // 1 (from "k") + 0 + 3 = 4
  $display("D %0d", wrap_dyn());        // size 1, da[0]==7 -> 1+7+1 = 9
  $display("END");
end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["G 1", "W 4", "D 9", "END"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
