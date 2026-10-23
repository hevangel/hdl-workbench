//! A callee local that shares its bare name with the caller's PASSED associative-array
//! actual must not destroy the copy-back of an `output`/`inout` formal.
//!
//! §6.21 automatic locals and the simulator's flat, global-by-bare-name collection
//! tables collide here: when the callee declares `int A[string]` with the same name as
//! the caller's actual `A`, the declaration takes a snapshot of the caller's collection
//! into a queue frame and clears the bare-name registration. On return the frame restore
//! wipes every bare-name element it saved — which, if the `output` writeback had already
//! copied the formal's data onto `A`, silently discarded it.
//!
//! Queues already deferred their writeback until after the restore
//! (`stage_queue_param` in `pop_and_restore_queue_frame`); associative-array (and
//! fixed-array) formals did not. This pins that the output of a callee local named like
//! the actual survives for assoc arrays, across function and task call sites and
//! `output`/`inout` directions.
//!
//! Verified byte-for-byte against reference simulators.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn output_assoc_survives_callee_local_same_name() {
    // Function call site.
    let sim = simulate(
        r#"
module top;
  function automatic void fill(output int p[string]);
    int A[string];
    p["x"] = 42;
  endfunction
  initial begin
    int A[string];
    fill(A);
    $display("X=%0d", A["x"]);
    if (A["x"]==42) $display("TAG_PASS"); else $display("TAG_FAIL");
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"X=42".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");

    // Task call site.
    let sim = simulate(
        r#"
module top;
  task automatic fill(output int p[string]);
    int A[string];
    p["x"] = 42;
  endtask
  initial begin
    int A[string];
    fill(A);
    $display("X=%0d", A["x"]);
    if (A["x"]==42) $display("TAG_PASS"); else $display("TAG_FAIL");
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"X=42".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}

#[test]
fn inout_assoc_survives_callee_local_same_name() {
    let sim = simulate(
        r#"
module top;
  task automatic fill(inout int p[string]);
    int A[string];
    p["x"] = p["y"] + 1;   // reads the caller's key, adds 1
    p["z"] = 9;            // fresh key
  endtask
  initial begin
    int A[string];
    A["y"] = 41;
    fill(A);
    $display("X=%0d Y=%0d Z=%0d", A["x"], A["y"], A["z"]);
    if (A["x"]==42 && A["y"]==41 && A["z"]==9) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"X=42 Y=41 Z=9".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}

#[test]
fn different_or_no_callee_local_does_not_break_output() {
    // Different-name callee local.
    let sim = simulate(
        r#"
module top;
  function automatic void fill(output int p[string]);
    int B[string];
    p["x"] = 42;
  endfunction
  initial begin
    int A[string];
    fill(A);
    $display("X=%0d", A["x"]);
    if (A["x"]==42) $display("TAG_PASS"); else $display("TAG_FAIL");
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"X=42".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");

    // No callee local of any kind.
    let sim = simulate(
        r#"
module top;
  function automatic void fill(output int p[string]);
    p["x"] = 42;
  endfunction
  initial begin
    int A[string];
    fill(A);
    $display("X=%0d", A["x"]);
    if (A["x"]==42) $display("TAG_PASS"); else $display("TAG_FAIL");
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"X=42".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}
