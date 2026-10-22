//! class-perf: a COMPILED free function must not disturb the caller's
//! locals. The compiled-free-function hook used to return early with an
//! epilogue that only mirrored the interpreter's stack pops, skipping the
//! interpreter's post-body formal-teardown sequence — most importantly
//! `pop_local_frame`. Every compiled free-fn call therefore leaked its
//! local frame onto `local_stack`, so the CALLER (e.g. a task whose string
//! formal was bound before the call) resolved its variables against the
//! stale callee frame and string formals came back as garbage (a single
//! stray byte instead of "simple_test"). This is the `run_test(name)` →
//! `uvm_dpi_get_next_arg` shape from the UVM run_test flow.

use xezim::simulate;

fn gate_on() -> bool {
    super::compiled_method_test_env::eager()
}

fn out(src: &str) -> String {
    let sim = simulate(src, 200).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

const FREE_FN_CALLER_LOCALS: &str = r#"
function string ff_next(int k);
  return "";
endfunction

task inner(string test_name);
  string x;
  x = ff_next(1);
  $display("INNER[%s]", test_name);
endtask

module top;
  initial begin
    inner("simple_test");
  end
endmodule
"#;

#[test]
fn compiled_free_fn_leaves_caller_string_formal_intact() {
    if !gate_on() {
        return;
    }
    let o = out(FREE_FN_CALLER_LOCALS);
    // The task's string formal must survive a compiled free-fn call made
    // between the binding and the use.
    assert!(
        o.contains("INNER[simple_test]"),
        "string formal corrupted by compiled free fn:\n{}",
        o
    );
}
