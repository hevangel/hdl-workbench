//! Class-handle preservation through function/method return paths.
//!
//! Two distinct bugs that together broke UVM's registry/factory and
//! report-message machinery. Both silently dropped a class HANDLE so that a
//! later `obj.method()` dispatched onto a null/wrong object.
//!
//! 1. **Implicit-return assignment of a class handle** —
//!    `funcname = new(args)` inside a class-returning `function` lost the
//!    handle. The implicit return variable (named after the function) was
//!    sized correctly, but its class type was never registered in
//!    `var_class_types`, so the `new(args)` construction could not resolve
//!    the target class and the handle was dropped. UVM's
//!    `uvm_report_message::new_report_message()` uses exactly this idiom:
//!    `new_report_message = new(name);`. Fixed by registering the return
//!    variable's class type in `exec_function_call`.
//!
//!    (The mirrored method-call path `exec_method_in_class_hierarchy`
//!    already did this registration; the function-call path did not.)
//!
//! 2. **Class-scoped typedef static call loses its return value** —
//!    `Class::Typedef::static_fn()` (e.g. UVM's
//!    `base_class::type_id::get()`) returned null. The static-call
//!    dispatcher only handled `pkg::Class::method()` where the middle
//!    segment names a *class*. When the middle segment is a *typedef alias*
//!    to a class (`typedef Holder type_id;`) or to a parameterized
//!    specialization (`typedef registry#(T) type_id;`), the dispatcher
//!    could not follow the alias, so the call ran but its return value was
//!    discarded. Fixed by adding `resolve_class_member_typedef_class` /
//!    `resolve_class_member_typedef_spec` to the dispatcher.

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

// ── 1. implicit-return `funcname = new(args)` keeps the handle ────────
// A package-scoped class `C` and two class-returning functions: `mk_direct`
// assigns the implicit return var directly (`mk_direct = new(x)`), `mk_local`
// routes through a local (`t = new(x); mk_local = t`). At HEAD both lose the
// handle (`d == null`); with the fix the handle survives and the field is
// readable.
const IMPLICIT_RETURN_SRC: &str = r#"
package pkg;
  class C;
    int v;
    function new(int v = 0); this.v = v; endfunction
  endclass
  // Direct: assign the implicit return variable named after the function.
  function C mk_direct(int x);
    mk_direct = new(x);
  endfunction
  // Indirect: construct into a local, then assign the return var from it.
  function C mk_local(int x);
    C t;
    t = new(x);
    mk_local = t;
  endfunction
endpackage
module top;
  import pkg::*;
  initial begin
    C d, l;
    d = mk_direct(11);
    l = mk_local(22);
    if (d == null)       $display("TAG_FAIL direct=null");
    else if (d.v != 11)  $display("TAG_FAIL direct.v=%0d", d.v);
    else if (l == null)  $display("TAG_FAIL local=null");
    else if (l.v != 22)  $display("TAG_FAIL local.v=%0d", l.v);
    else                 $display("TAG_PASS");
  end
endmodule
"#;

#[test]
fn implicit_return_var_keeps_class_handle() {
    let sim = simulate(IMPLICIT_RETURN_SRC, 100).expect("simulate failed");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "TAG_PASS"),
        "`funcname = new(args)` must preserve the class handle in the \
         implicit return variable; got {:?}",
        msgs
    );
}

// ── 2a. non-parameterized `Class::Typedef::static_fn()` return ────────
// `Base::type_id` is a typedef alias (`typedef Holder type_id;`); calling
// `Base::type_id::get()` must resolve the alias to `Holder` and return the
// handle. At HEAD the return value is dropped (h == null).
const SCOPED_TYPEDEF_NONPARAM_SRC: &str = r#"
package pkg;
  class Holder;
    int tag;
    static function Holder get();
      static Holder me;
      if (me == null) begin me = new; me.tag = 777; end
      return me;
    endfunction
  endclass
  class Base;
    typedef Holder type_id;   // non-parameterized typedef alias to a class
  endclass
endpackage
module top;
  import pkg::*;
  initial begin
    Base::type_id h = Base::type_id::get();
    if (h == null)       $display("TAG_FAIL h=null");
    else if (h.tag != 777) $display("TAG_FAIL h.tag=%0d", h.tag);
    else                 $display("TAG_PASS");
  end
endmodule
"#;

#[test]
fn scoped_typedef_static_call_nonparam() {
    let sim = simulate(SCOPED_TYPEDEF_NONPARAM_SRC, 100).expect("simulate failed");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "TAG_PASS"),
        "`Class::Typedef::static_fn()` (non-parameterized alias) must return \
         the handle; got {:?}",
        msgs
    );
}

// ── 2b. parameterized `Class::Typedef::static_fn()` return ────────────
// `Base::type_id` is now a typedef of a *parameterized* specialization
// (`typedef reg#(T) type_id;`). The dispatcher must follow the alias to the
// specialization and return the handle. This mirrors UVM's
// `typedef uvm_object_registry#(T,\"T\") type_id;` inside a class `T`.
const SCOPED_TYPEDEF_PARAM_SRC: &str = r#"
package pkg;
  // A parameterized registry that hands out a singleton handle.
  class registry #(type T = int);
    static registry#(T) me;
    static function registry#(T) get();
      if (me == null) me = new;
      return me;
    endfunction
    int serial;
  endclass
  class Base;
    // Parameterized typedef alias — like UVM's `type_id`.
    typedef registry#(Base) type_id;
  endclass
endpackage
module top;
  import pkg::*;
  initial begin
    Base::type_id h = Base::type_id::get();
    if (h == null) $display("TAG_FAIL h=null");
    else           $display("TAG_PASS");
  end
endmodule
"#;

#[test]
fn scoped_typedef_static_call_param() {
    let sim = simulate(SCOPED_TYPEDEF_PARAM_SRC, 100).expect("simulate failed");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "TAG_PASS"),
        "`Class::Typedef::static_fn()` (parameterized alias) must return \
         the handle; got {:?}",
        msgs
    );
}
