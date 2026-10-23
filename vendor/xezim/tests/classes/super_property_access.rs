//! §8.15 — `super.<property>`. Reference-validated.
//!
//! `super` was only ever handled as a METHOD-call receiver. As an expression
//! base it matched no arm, so `super.p` flattened to the phantom name
//! "super.p": reads returned 0 and writes were swallowed, with no diagnostic.
//! It failed even for a uniquely-named inherited property, so this was not
//! about shadowing — `this.p` and a bare `p` both worked, only the `super.`
//! spelling did not.
//!
//! `super` names the same object as `this`; only method lookup differs, and a
//! method call is a different expression shape, so `super.foo()` still
//! dispatches to the base.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn super_property_reads_and_writes_reach_the_object() {
    let src = r#"
class B;
  int s;
  int t;
  function new(); s = 100; t = 101; endfunction
  virtual function int who(); return 1; endfunction
endclass
class D extends B;
  virtual function int who(); return 2; endfunction
  function int getsup();      return super.s; endfunction
  function int getbare();     return s;       endfunction
  function int getthis();     return this.s;  endfunction
  function void setsup(int v); super.t = v;   endfunction
  function int call_super();  return super.who(); endfunction
  function int call_virtual(); return who();  endfunction
endclass
module tb;
  D d;
  int q1, q2, q3, q4, q5, q6;
  initial begin
    d = new();
    q1 = d.getsup();
    q2 = d.getbare();
    q3 = d.getthis();
    d.setsup(9);
    q4 = d.t;
    q5 = d.call_super();
    q6 = d.call_virtual();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "q1"), 100, "super.<prop> read");
    assert_eq!(u(&sim, "q2"), 100, "bare name still works");
    assert_eq!(u(&sim, "q3"), 100, "this.<prop> still works");
    assert_eq!(u(&sim, "q4"), 9, "super.<prop> write reaches the object");
    assert_eq!(
        u(&sim, "q5"),
        1,
        "super.<method>() still dispatches to the base"
    );
    assert_eq!(
        u(&sim, "q6"),
        2,
        "an unqualified call still dispatches virtually"
    );
}

/// §18.5.7 — a `.size()` constraint nested inside `if/else`. The size solver
/// only descended into Block/Soft/Inside/Expr items, so a size bound written
/// under a conditional never reached it: the queue got an arbitrary size and
/// `randomize()` still returned 1.
#[test]
fn queue_size_constraint_under_if_else() {
    let src = r#"
class Q;
  rand bit [7:0] q[$];
  bit sel;
  constraint c { if (sel) q.size() == 2; else q.size() == 4; }
endclass
module tb;
  Q a, b;
  int size_when_set, size_when_clear, ok;
  initial begin
    ok = 1;
    a = new(); a.sel = 1'b1;
    if (!a.randomize()) ok = 0;
    size_when_set = a.q.size();
    b = new(); b.sel = 1'b0;
    if (!b.randomize()) ok = 0;
    size_when_clear = b.q.size();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "ok"), 1, "both randomize calls succeed");
    assert_eq!(u(&sim, "size_when_set"), 2, "the then-branch size applies");
    assert_eq!(
        u(&sim, "size_when_clear"),
        4,
        "the else-branch size applies"
    );
}

/// A parenthesis-less `super.new;` is a CONSTRUCTOR call, not a property
/// reference — it parses as the same shape, and treating it as one recursed
/// into construction until the stack overflowed.
#[test]
fn super_new_without_parens_still_constructs() {
    let src = r#"
class B;
  int v;
  function new(); v = 7; endfunction
endclass
class C extends B;
  function new();
    super.new;
  endfunction
endclass
module tb;
  C c;
  int got;
  initial begin
    c = new();
    got = c.v;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "got"), 7);
}

/// §18.5.6 — a `.size()` constraint under an IMPLICATION. The parser's
/// expression grammar consumes `->` as the low-precedence LogImplies OPERATOR
/// before the constraint parser can see it, so `sel -> q.size() == 2` arrives
/// as one Expr item — never as ConstraintItem::Implication — and the size
/// bound under it was silently dropped. Worse, the resulting mismatch made the
/// solver force `sel` to 0 on every draw, so the guard never even varied.
#[test]
fn queue_size_constraint_under_implication() {
    let src = r#"
class Q3;
  rand bit [7:0] q[$];
  rand bit sel;
  constraint c { sel -> q.size() == 2; !sel -> q.size() == 4; }
endclass
module tb;
  Q3 x;
  int ok, saw_sel1, saw_sel0;
  initial begin
    ok = 1;
    x = new();
    for (int i = 0; i < 40; i++) begin
      if (!x.randomize()) ok = 0;
      if (x.sel) begin
        saw_sel1 = 1;
        if (x.q.size() != 2) ok = 0;
      end else begin
        saw_sel0 = 1;
        if (x.q.size() != 4) ok = 0;
      end
    end
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "ok"), 1, "the guarded size holds on every draw");
    assert_eq!(
        u(&sim, "saw_sel1"),
        1,
        "the guard actually varies (was pinned 0)"
    );
    assert_eq!(u(&sim, "saw_sel0"), 1);
}
