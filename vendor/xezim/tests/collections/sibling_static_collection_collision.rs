//! §8.9 — a `static` collection (queue / assoc array) is one copy PER CLASS,
//! shared across instances/subclasses but NOT across distinct classes. Two
//! sibling classes declaring a same-named `static … settings[$]` must get
//! separate stores. xezim materialized every static collection under its bare
//! name, so the two siblings shared one cell: a module-scope read of either
//! (`A.settings[k]`, `.size()`) or an in-method bare access collided. In UVM
//! this made the four sibling `uvm_cmdline_*` classes' `settings` queues share
//! storage, so a verbosity `+UVM_VERBOSITY=` setting leaked into the
//! `uvm_set_action=/uvm_set_severity=/uvm_set_verbosity=` checkers, emitting
//! spurious "never took effect" INVLCMDARGS warnings that inflated the warning
//! severity count and broke self-checking report tests. Verified byte-for-byte
//! against a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal {} not found", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}
/// Two sibling classes declaring a same-named static queue must keep
/// independent stores; writes and index reads must not bleed across classes.
#[test]
fn sibling_classes_same_named_static_queue_independent() {
    let src = r#"
module top;
  class base; endclass
  class A extends base;
    static int settings[$];
  endclass
  class B extends base;
    static int settings[$];
  endclass
  int ra, rb, a0, b0;
  initial begin
    A::settings.push_back(1);
    B::settings.push_back(2);
    ra = A::settings.size();
    rb = B::settings.size();
    a0 = A::settings[0];
    b0 = B::settings[0];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "ra"), 1, "A.settings holds its own single element");
    assert_eq!(u(&sim, "rb"), 1, "B.settings holds its own single element");
    assert_eq!(u(&sim, "a0"), 1, "A.settings[0] is A's write");
    assert_eq!(u(&sim, "b0"), 2, "B.settings[0] is B's write");
}

/// Control: a static collection declared by exactly ONE class keeps its
/// bare-name store (sibling separation must not split single-class storage).
#[test]
fn single_declaring_class_static_queue_unchanged() {
    let src = r#"
module top;
  class S;
    static int log[$];
  endclass
  int n, first;
  initial begin
    S::log.push_back(9);
    n = S::log.size();
    first = S::log[0];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "n"), 1);
    assert_eq!(u(&sim, "first"), 9);
}

/// A same-named static collection shared through an INHERITED position still
/// resolves to the DECLARING class's single store (subclasses share the base
/// static — §8.9), across both A and B accesses.
#[test]
fn inherited_static_queue_shared_through_subclasses() {
    let src = r#"
module top;
  class S;
    static int cache[$];
  endclass
  class A extends S; endclass
  class B extends S; endclass
  int n, first;
  initial begin
    A::cache.push_back(3);
    B::cache.push_back(4);
    n = A::cache.size();   // same store as B's push
    first = A::cache[0];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    // `cache` is declared in S alone (no sibling collision), so bare-name
    // storage is shared by both subclasses.
    assert_eq!(u(&sim, "n"), 2);
    assert_eq!(u(&sim, "first"), 3);
}

/// A static collection DECLARED in a PARAMETERIZED base and iterated through a
/// specialized subclass with `foreach (this_type::coll[i])` must not be
/// mistaken for a sibling collision. The subclass inherits one cell with the
/// base — routing it to a per-class key (empty, unmaterialised) made the
/// `foreach` over the EMPTY collection produce a spurious NULL element, which
/// corrupted the sequence adapter's `sequences` array
/// (`init_sequence_library` → `this_type::m_typewide_sequences`).
#[test]
fn base_static_queue_foreach_through_specialized_subclass_not_null() {
    let src = r#"
module top;
  class base #(type DT = int);
    static DT items[$];
  endclass
  class sub extends base #(int);
    typedef sub this_type;
    function int init();
      int n;
      foreach (this_type::items[i])
        n += 1;
      return n;
    endfunction
  endclass
  sub s;
  int count;
  initial begin
    s = new();
    count = s.init();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    // `items` is declared only in the parameterized base; `sub` inherits the
    // single (empty) store. A false sibling collision would route
    // `this_type::items` to a per-class key and iterate a spurious NULL over
    // the empty collection, so `count` must stay 0.
    assert_eq!(u(&sim, "count"), 0);
}
/// §8.9 — a static collection REDECLARED in a derived class (not merely
/// inherited) must get its OWN cell, separate from the base's. This is the
/// `uvm_sequence_library_utils` pattern: `simple_seq_lib_RST` redeclares
/// `static m_typewide_sequences[$]` and `simple_seq_lib` (a subclass) inherits
/// the base's — a `d::q` push must never bleed into `b::q` and vice-versa.
/// The reference simulator keeps `b::q` = [1,2] and `d::q` = [3].
#[test]
fn derived_redeclared_static_queue_is_separate_from_base() {
    let src = r#"
module top;
  class b;
    static int m_q[$];
  endclass
  class d extends b;
    static int m_q[$];
  endclass
  int rb, rd, b0, d0;
  initial begin
    b::m_q.push_back(1);
    b::m_q.push_back(2);
    d::m_q.push_back(3);
    rb = b::m_q.size();
    rd = d::m_q.size();
    b0 = b::m_q[0];
    d0 = d::m_q[0];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "rb"), 2, "base keeps only its own two writes");
    assert_eq!(u(&sim, "rd"), 1, "derived holds only its own one write");
    assert_eq!(u(&sim, "b0"), 1, "base[0] is the base's first write");
    assert_eq!(u(&sim, "d0"), 3, "derived[0] is the derived's write");
}

/// Same as above but with the base accessed through an intermediate subclass
/// that does NOT redeclare (inherited-only): the single base cell is shared,
/// so `a::q` reads still see the base's writes, while a redeclaring `d` stays
/// separate.
#[test]
fn inherited_versus_redeclared_static_queue() {
    let src = r#"
module top;
  class base;
    static int q[$];
  endclass
  class a extends base; endclass      // inherits base::q
  class d extends a;
    static int q[$];                  // redeclares — own cell
  endclass
  int nar, nrd, ar;
  initial begin
    a::q.push_back(1);                // goes to base::q (inherited)
    d::q.push_back(2);                // goes to d::q (own)
    nar = a::q.size();                // 1 (base cell)
    nrd = d::q.size();                // 1 (own cell)
    ar = a::q[0];                     // 1
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "nar"), 1, "inherited a::q sees only the base write");
    assert_eq!(u(&sim, "nrd"), 1, "redeclared d::q holds its own write");
    assert_eq!(u(&sim, "ar"), 1, "a::q[0] is the base's write");
}

/// §8.x — a base-class method reading a qualified `this_type::static` must
/// resolve `this_type` to the DEFINING class (the base), not the receiver's
/// dynamic (derived) type, so a static-collection member declared in the base
/// and re-declared in a derived class stays separate per-class. This is the
/// `uvm_sequence_library` `init_sequence_library()` pattern: the base `new`
/// runs the base method reading `this_type::typewide` (the base cell), while
/// the derived `new` runs the derived method reading `Derived::typewide` (its
/// own cell). The reference simulator's derived instance ends up holding BOTH the base's
/// [1,2] and its own [3] = 3 elements.
#[test]
fn base_method_this_type_static_resolves_to_defining_class() {
    let src = r#"
module top;
  class item; int v; endclass
  class base;
    item sequences[$];
    static item typewide[$];
    typedef base this_type;
    function new();
      init_sequence_library();
    endfunction
    function void init_sequence_library();
      foreach (this_type::typewide[i])
        sequences.push_back(this_type::typewide[i]);
    endfunction
    function int n();
      return sequences.size();
    endfunction
  endclass
  class derived extends base;
    static item typewide[$];
    typedef derived this_type;
    function new();
      super.new();
      init_sequence_library();
    endfunction
    function void init_sequence_library();
      foreach (derived::typewide[i])
        sequences.push_back(derived::typewide[i]);
    endfunction
  endclass
  item b1, b2, r3;
  derived d;
  int n;
  initial begin
    b1 = new(); b1.v = 1;
    b2 = new(); b2.v = 2;
    r3 = new(); r3.v = 3;
    base::typewide.push_back(b1);
    base::typewide.push_back(b2);
    derived::typewide.push_back(r3);
    d = new();
    n = d.n();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "n"),
        3,
        "base init (this_type -> base [1,2]) + derived init (-> [3]) = 3"
    );
}
