// Regression test: an OUT-OF-CLASS method definition (`task C::run`) must
// NOT be inserted into the module's free-task namespace under the bare name.
//
// LRM §8.25: a method declared `extern` inside a class is given its body
// out of class via the `Class::method` syntax. The elaborator was inserting
// these out-of-class definitions into the module's `tasks`/`functions` maps
// under the bare method name, so a later out-of-class definition silently
// overrode a same-named free task/function. (In real UVM this clobbered
// `task run_test` in uvm_globals.svh with uvm_root's out-of-class
// `task uvm_root::run_test`, so a bare `run_test()` dispatched the class
// body without a bound `this`.)
//
// The fix: skip the `tasks`/`functions` insert when the declaration's name
// has a scope (`Class::method`), at every elaboration entry point
// (module-with-defs, elaborate_items, package import).

use std::process::Command;

#[test]
fn out_of_class_method_does_not_override_free_task() {
    // ORDER MATTERS: the free task is declared FIRST and the out-of-class
    // method body SECOND, so without the fix the method's insert() under
    // bare name "run" clobbers the free task. (Mirrors UVM's declaration
    // order: uvm_globals.svh before uvm_root.svh.)
    let src = r#"module top;
  class C;
    int id;
    function new(int i);
      id = i;
    endfunction
    extern task run;          // prototype (LRM 8.25)
  endclass

  // Free task, SAME bare name. Declared FIRST.
  task run;
    $display("FREE_RUN");
  endtask

  // Out-of-class method body, declared SECOND. Must not clobber the free task.
  task C::run;
    $display("CLASS_RUN id=%0d", id);
  endtask

  initial begin
    C c;
    c = new(42);
    run();          // must call the FREE task  -> FREE_RUN
    c.run();        // must call the METHOD     -> CLASS_RUN id=42
    if (1) $display("TAG_PASS"); else $display("TAG_FAIL");
  end
endmodule
"#;

    let dir = std::env::temp_dir().join(format!("xezim_ooc_method_shadow_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv_path = dir.join("ooc_method_shadow.sv");
    std::fs::write(&sv_path, src).unwrap();

    let bin = env!("CARGO_BIN_EXE_xezim");
    let out = Command::new(bin)
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(sv_path.to_str().unwrap())
        .output()
        .expect("failed to run xezim");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The bare run() must hit the FREE task, and c.run() the method (with
    // a properly bound `this`, so id prints 42, not empty/uninitialized).
    assert!(
        stdout.contains("FREE_RUN") && stdout.contains("CLASS_RUN id=42"),
        "out-of-class method body overrode the free task, or `this` was unbound.\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("TAG_PASS") && !stdout.contains("TAG_FAIL") && !stdout.contains("FAIL"),
        "unexpected FAIL in output.\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

#[test]
fn out_of_class_function_does_not_override_free_function() {
    // Same bug for `function Class::fn` — verify the function arm of the fix.
    let src = r#"module top;
  class C;
    int id;
    function new(int i);
      id = i;
    endfunction
    extern function int peek;
  endclass

  // Free function, SAME bare name. Declared FIRST.
  function int peek;
    return 7;
  endfunction

  // Out-of-class function body, declared SECOND.
  function int C::peek;
    return id;
  endfunction

  initial begin
    C c;
    c = new(99);
    // Bare peek() must return the free function's 7 (not the method's 99).
    if (peek() == 7 && c.peek() == 99)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL free=%0d method=%0d", peek(), c.peek());
  end
endmodule
"#;

    let dir = std::env::temp_dir().join(format!("xezim_ooc_func_shadow_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv_path = dir.join("ooc_func_shadow.sv");
    std::fs::write(&sv_path, src).unwrap();

    let bin = env!("CARGO_BIN_EXE_xezim");
    let out = Command::new(bin)
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(sv_path.to_str().unwrap())
        .output()
        .expect("failed to run xezim");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("TAG_PASS") && !stdout.contains("TAG_FAIL"),
        "out-of-class function body overrode the free function.\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
