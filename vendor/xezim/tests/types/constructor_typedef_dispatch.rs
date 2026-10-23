// Regression test: constructor dispatch for a module-level variable whose
// declared type is a TYPEDEF alias for a class.
//
// xezim bug (heartbeat failure root cause): the `is_new_call` guard in
// `exec_statement`'s BlockingAssign handler checked `module.classes.contains_key(tn)`
// for the lvalue's resolved type name. A typedef alias like
//   typedef uvm_objection uvm_callbacks_objection;
// resolves the type to `"uvm_callbacks_objection"`, which is NOT a key in
// `module.classes` — so `is_class = false`, the guard fails, and the
// `new(...)` call falls through to `eval_expr` which returns 0 (null handle).
//
// The fix: apply `resolve_simple_typedef_class(&tn)` before the `contains_key`
// check, so that typedef aliases resolve to the underlying class name.
//
// Two test scenarios:
//   1. Module-level typedef + module-level variable + ctor inside a function body
//   2. Module-level typedef + function-local variable + ctor inside a function body
// Both must produce non-null handles.
//
// Verified byte-for-byte against reference simulators.
// Test 1 (function body): result=1 PASS (xezim // Both must produce non-null handles. reference)
// Test 2 (module-level var): result=1 PASS (xezim // Both must produce non-null handles. reference)

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(&format!("top.{}", n))
        .or_else(|| sim.get_signal(n))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

const SRC_CTOR_IN_FUNC: &str = r#"
// A simple class with a constructor that records the argument
class myobj;
  int val;
  function new(int v);
    val = v;
  endfunction
endclass

// Typedef alias for the class
typedef myobj myobj_alias;

// Another class that holds a myobj_alias handle, assigned in its constructor
class holder;
  myobj_alias obj;
  int construction_ok;
  function new(int v);
    // This assignment must dispatch through the is_new_call path.
    // Without the typedef resolution fix, is_class=false and obj stays null.
    obj = new(v);
    if (obj != null && obj.val == v)
      construction_ok = 1;
    else
      construction_ok = 0;
  endfunction
endclass

module top;
  int result;
  initial begin
    holder h;
    h = new(42);
    if (h.construction_ok == 1)
      result = 1;
    else
      result = 0;
  end
endmodule
"#;

const SRC_MODULE_LEVEL_VAR: &str = r#"
class myobj;
  int val;
  function new(int v);
    val = v;
  endfunction
endclass

typedef myobj myobj_alias;

// Module-level variable of the typedef-aliased type
myobj_alias global_obj;

class holder;
  int check_ok;
  function new();
    // global_obj is a module-level variable, assigned here.
    // Without the typedef resolution fix, is_class=false and global_obj stays null.
    global_obj = new(99);
    if (global_obj != null && global_obj.val == 99)
      check_ok = 1;
    else
      check_ok = 0;
  endfunction
endclass

module top;
  int result;
  initial begin
    holder h;
    h = new();
    if (h.check_ok == 1)
      result = 1;
    else
      result = 0;
  end
endmodule
"#;

#[test]
fn constructor_typedef_dispatch_in_function() {
    let sim = simulate(SRC_CTOR_IN_FUNC, 1000).expect("simulate failed");
    assert_eq!(
        u(&sim, "result"),
        1,
        "constructor dispatch with typedef alias in function body must produce non-null handle"
    );
}

#[test]
fn constructor_typedef_dispatch_module_level_var() {
    let sim = simulate(SRC_MODULE_LEVEL_VAR, 1000).expect("simulate failed");
    assert_eq!(
        u(&sim, "result"),
        1,
        "constructor dispatch with typedef alias on module-level variable must produce non-null handle"
    );
}
