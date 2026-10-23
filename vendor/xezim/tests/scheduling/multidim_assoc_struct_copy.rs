//! §7.8.1 multidimensional ASSOCIATIVE arrays of STRUCT (`T m[K1][K2][K3]`)
//! when a sub-array is selected by a class-handle key and iterated / read by
//! member (`m[rhs].first(lhs)`, `m[lhs][rhs][pol].state`).
//!
//! UVM's copier/comparer recursion guards (`m_recur_states[rhs][lhs][policy]`
//! and `m_recur_states[lhs][rhs][rec].state`) rely on this exact shape to
//! break the deep-clone of a CYCLIC object graph. Runtime gaps this relies on:
//!   - `is_associative_array` didn't recognize a nested sub-array receiver
//!     `base[key]` (only the bare `base`), so `.first()`/`.exists()` on that
//!     sub-array reported "not found"; UVM's copier cycle guard then never
//!     re-used the in-progress clone -> `clone()` of a cyclic graph recursed
//!     forever (stack overflow).
//!   - A class-member MULTIDIM assoc-of-struct cell write `cp.m[a][b][0] =
//!     '{...}` stored only an existence marker (1-D worked, the 3-D nested
//!     case was dropped by `coll_assoc_struct_elem`), so `.state` read back
//!     NEVER and the comparer guard could not detect an in-progress compare.

use xezim::simulate;

fn read_int(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn multidim_assoc_class_member_struct_copy() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  typedef enum {NEVER, STARTED, FINISHED} state_e;
  typedef struct { state_e state; int ret_val; } s_t;

  class node;
    node next;
    string name;
    function new(string nm = "node"); this.name = nm; endfunction
  endclass

  class guard_t;
    s_t m[node][node][int];

    function void set(node lhs, node rhs, int pol, state_e st);
      m[lhs][rhs][pol] = '{st, 5};
    endfunction

    function int is_started(node lhs, node rhs, int pol);
      if (!m.exists(lhs)) return 0;
      if (!m[lhs].exists(rhs)) return 0;
      if (!m[lhs][rhs].exists(pol)) return 0;
      return (m[lhs][rhs][pol].state == STARTED) ? 1 : 0;
    endfunction

    function int first_of(node src, inout node out);
      if (m[src].first(out)) return 1;
      return 0;
    endfunction
  endclass

  guard_t g;
  int member_ok = 0;
  int exists_ok = 0;
  int cycle_ok = 0;

  function automatic node guarded_clone(node src);
    node h = null;
    if (g.first_of(src, h))
      return h;                 // reuse -> cycle broken
    h = new(src.name);
    g.set(h, src, 0, STARTED);
    if (src.next != null)
      h.next = guarded_clone(src.next);
    return h;
  endfunction

  initial begin
    node a = new("a"), b = new("b"), ca;
    g = new;
    a.next = b; b.next = a;               // two-node cycle
    g.set(a, b, 0, STARTED);              // struct cell store
    member_ok = g.is_started(a, b, 0);
    exists_ok = g.m[a][b].exists(0);
    ca = guarded_clone(a);
    cycle_ok = (ca != null && ca.next != null && ca.next.next == ca) ? 1 : 0;
    $display("member_ok=%0d exists_ok=%0d cycle_ok=%0d", member_ok, exists_ok, cycle_ok);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 2000).expect("simulate failed");
    assert_eq!(
        read_int(&sim, "member_ok"),
        1,
        "multidim class-member assoc struct cell must retain its .state member"
    );
    assert_eq!(
        read_int(&sim, "exists_ok"),
        1,
        "nested assoc sub-array .exists(pol) must find the stored cell"
    );
    assert_eq!(
        read_int(&sim, "cycle_ok"),
        1,
        "guarded deep-clone of a cyclic graph must terminate and rewire the cycle"
    );
}
