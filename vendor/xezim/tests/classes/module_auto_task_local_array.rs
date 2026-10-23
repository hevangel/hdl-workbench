//! §6.21 / §13.3 — a fixed-size array declared in a module-scope `automatic`
//! task is per-invocation: every concurrent invocation owns its storage, just
//! like a class-method local (compare `concurrent_method_local_arrays`).
//! Previously only class-method locals got that isolation; an `automatic`
//! task's `int loc[K]` was registered under its bare name, so two sequences
//! running the task concurrently shared one `loc[2]`. Each wrote `loc[0]`
//! before a blocking `#1`, then read it back after the resume — the second
//! invocation saw the first's element and produced the wrong result (the
//! reference simulator printed `1 101 202`; xezim printed `1 202 0`). The
//! expected lines are the reference simulator's output.

use xezim::simulate;

const SRC: &str = r#"
module top;
  int results[2];
  task automatic t(input int idx, output int r);
    int loc[2];
    loc[0] = idx;
    #1;
    loc[1] = loc[0] * 100;
    r = loc[0] + loc[1];
  endtask
  initial begin
    fork
      begin int r; t(1, r); results[0]=r; end
      begin int r; t(2, r); results[1]=r; end
    join
    $display("%0t r0=%0d r1=%0d", $time, results[0], results[1]);
  end
endmodule
"#;

#[test]
fn automatic_module_task_local_array_is_per_invocation() {
    let out: Vec<String> = simulate(SRC, 50)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    // Each invocation owns its own `loc[2]`: t(1) → loc[0]=1, loc[1]=100
    // (r0=101) and t(2) → loc[0]=2, loc[1]=200 (r1=202). If the array were
    // shared, one of the two would see the other's loc[0] after the #1.
    let want = ["1 r0=101 r1=202"];
    assert_eq!(out, want, "{out:?}");
}
