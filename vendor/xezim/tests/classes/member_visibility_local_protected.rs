//! IEEE 1800-2023 §8.2 member visibility: `local` and `protected` class
//! members may not be accessed outside the declaring class (`local`) or
//! outside the declaring class and its subclasses (`protected`). xezim now
//! rejects these at elaboration time (the reference simulators report
//! `Illegal access to local/protected member`), so:
//!   * a module-scope `t.secret` where `secret` is `local` is a compile
//!     error, even when the member is inherited through `extends` — exactly
//!     the `t.events` case from the UVM library;
//!   * a subclass may read a `protected` member of `this` (legal);
//!   * public members stay accessible from everywhere, and unqualified
//!     accesses inside the declaring class remain legal.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal {n} not found"))
        .to_u64()
        .unwrap_or_else(|| panic!("{n} not u64-able (x/z?)"))
}

// --- Illegal: module-scope access to a `local` member inherited via
// `extends` (structurally identical to `t.events` in the UVM base classes).
#[test]
fn module_scope_local_member_is_rejected() {
    let src = r#"
module top;
  class Tr;
    local int secret_p;        // `local` in the base class
    int pub;
  endclass
  class MyTr extends Tr; endclass
  initial begin
    MyTr t;
    t = new();
    t.secret_p = 3;            // illegal: `local` member, module scope
    $display("unreached");
  end
endmodule
"#;
    let err = match simulate(src, 20) {
        Ok(_) => panic!("expected elaboration error"),
        Err(e) => e,
    };
    assert!(
        err.contains("Illegal access to local member 'secret_p'"),
        "expected local-member rejection, got: {err}"
    );
}

// --- Illegal: module-scope access to a `protected` member.
#[test]
fn module_scope_protected_member_is_rejected() {
    let src = r#"
module top;
  class Tr;
    protected int seed_count;
  endclass
  initial begin
    Tr t;
    t = new();
    t.seed_count = 5;          // illegal: module scope is never inside a class
    $display("unreached");
  end
endmodule
"#;
    let err = match simulate(src, 20) {
        Ok(_) => panic!("expected elaboration error"),
        Err(e) => e,
    };
    assert!(
        err.contains("Illegal access to protected member 'seed_count'"),
        "expected protected-member rejection, got: {err}"
    );
}

// --- Illegal: a `local` member of a base is not visible in a subclass,
// and a non-subclass class cannot reach another instance's `protected`.
#[test]
fn base_local_hidden_from_subclass() {
    let src = r#"
module top;
  class Tr;
    local int secret_p;
  endclass
  class MyTr extends Tr;
    function int read_secret();
      return this.secret_p;    // illegal: `local` is only visible in Tr
    endfunction
  endclass
  int r;
  initial begin
    MyTr t;
    t = new();
    r = t.read_secret();
  end
endmodule
"#;
    let err = match simulate(src, 20) {
        Ok(_) => panic!("expected elaboration error"),
        Err(e) => e,
    };
    assert!(
        err.contains("Illegal access to local member 'secret_p'"),
        "expected base-local-hidden-from-subclass rejection, got: {err}"
    );
}

// --- Legal: subclass reads `protected` via `this`; public and unqualified
// `local`/`protected` accesses inside the declaring class stay valid.
#[test]
fn subclass_protected_this_and_public_pass() {
    let src = r#"
module top;
  class Tr;
    protected int p;
    local int priv;
    int pub_in;
    function new(); p = 1; priv = 2; pub_in = 3; endfunction
    task knock_priv(int v); priv = v; endtask   // unqualified `local` write
    function int read_priv(); return priv; endfunction
  endclass
  class MyTr extends Tr;
    function int read_prot();
      return this.p;          // legal: protected `this` access in a subclass
    endfunction
  endclass
  int result;
  int r;
  initial begin
    MyTr t;
    t = new();
    t.pub_in = 10;            // legal public access from module scope
    r = t.read_prot();        // legal subclass protected read
    r = r + t.read_priv();    // legal (via subclass-inherited method)
    result = (r === 3) ? 1 : 0;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("legal visibility program unexpectedly failed");
    assert_eq!(u(&sim, "result"), 1);
}
