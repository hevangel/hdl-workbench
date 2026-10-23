//! class-perf: a function / method that declares an EMPTY local collection
//! (`chandle cached[$];`, `int q[$];`) used to lower that one declaration to
//! a `StmtFallback("VarDecl_local_coll")`, which tripped the fast-call
//! admission gate (`has_forbidden`). The whole enclosing body — e.g.
//! `uvm_re_match`, the hottest function in the 7352_regex regression — was
//! then forced onto the AST interpreter every call. That single declaration
//! is now a dedicated `Insn::InitLocalColl` (no initializer, single
//! QUEUE/unsized dim) so the body stays frame-free-eligible.
//!
//! This regression locks in the SEMANTICS of that dedicated opcode: the
//! collection must come up empty, must resolve under the per-call local
//! frame (a `@name#id` key that does not bleed across calls), and must work
//! from a free function, a class method, and a nested call chain — exactly
//! the `uvm_re_match` shape. Values are the ones used in the reference
//! simulator's run.
//!
//! NOTE: this is a semantics test. Fast-call *eligibility* is exercised by
//! the fact that the enclosing body is compiled at all (the old fallback was
//! permitted but slow); correctness byte-for-byte is the compat contract.

use xezim::simulate;

#[test]
fn empty_local_collection_decl_is_compiled() {
    // Compiled from the first call (tier 0) in a child process, then again
    // here at the default tier.
    let _ = super::compiled_method_test_env::eager();
    let src = r#"module top;

class cache;
  static cache m_inst;
  protected string keys[$];
  static function cache get_inst();
    if (m_inst == null) m_inst = new;
    return m_inst;
  endfunction
  function void put(string k); keys.push_back(k); endfunction
  function int count(); return keys.size(); endfunction
endclass

// the uvm_re_match shape: an empty local queue-collection declaration
// alongside a static-method call and collection methods.
function automatic int look(string k);
  cache c;
  string cached[$];
  int hit;
  c = cache::get_inst();
  cached.delete();     // explicit clear (decl is already empty)
  if (cached.size() == 0) begin
    c.put(k);
    hit = 0;
  end else begin
    hit = cached.size();
  end
  return hit + c.count() * 100;
endfunction

// nested chain: caller also declares an empty local collection.
function automatic int outer(string k);
  automatic cache c;
  automatic string seen[$];
  automatic int r;
  c = cache::get_inst();
  seen.delete();
  r = look(k);
  // per-call frame: out here, `seen` is still empty (no bleed).
  return r + seen.size() + c.count();
endfunction

initial begin
  automatic int a;
  automatic int b;
  automatic int same;
  // first call builds the cache list; second call sees a cache hit path.
  a = look("x");          // miss: hit=0, singleton cache count=1 -> 0 + 100 = 100
  b = look("x");          // miss again; cache now 2 -> 0 + 200 = 200
  same = outer("y");      // r=look("y")=300; seen.size()=0; c.count()=3 -> 303
  $display("A %0d", a);
  $display("B %0d", b);
  $display("S %0d", same);
  $display("END");
end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["A 100", "B 200", "S 303", "END"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
