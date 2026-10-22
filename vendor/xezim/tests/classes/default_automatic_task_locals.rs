//! IEEE 1800-2023 §6.21 / §13.3: a module, interface or program declared
//! `automatic` makes its tasks automatic by default, so their local arrays
//! are per-invocation exactly as in an explicit `task automatic`; an
//! explicit `task static` there still shares its locals. Multi-dimensional
//! locals of an automatic task are per-invocation too. Before, only an
//! explicit `automatic` keyword isolated the locals, and a 2-D local was
//! always shared, so concurrent invocations read each other's elements.
//! Expected lines are the reference simulator's output.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn module_automatic_task_locals_are_per_invocation() {
    let out = t_lines(
        r#"
module automatic top;
  int results[2];
  task t(input int idx, output int r);
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
    $display("T| a2 r0=%0d r1=%0d", results[0], results[1]);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T| a2 r0=101 r1=202"], "{out:?}");
}

#[test]
fn program_automatic_and_package_task_locals() {
    let out = t_lines(
        r#"
package p;
  task automatic pt(input int idx, output int r);
    int loc[2];
    loc[0] = idx; #1; r = loc[0];
  endtask
endpackage
program automatic prg;
  int res[4];
  task t(input int idx, output int r);
    int loc[2];
    loc[0] = idx; #1; r = loc[0];
  endtask
  initial begin
    fork
      begin int r; t(1, r); res[0]=r; end
      begin int r; t(2, r); res[1]=r; end
      begin int r; p::pt(3, r); res[2]=r; end
      begin int r; p::pt(4, r); res[3]=r; end
    join
    $display("T| a7 %0d %0d %0d %0d", res[0], res[1], res[2], res[3]);
  end
endprogram
"#,
    );
    assert_eq!(out, ["T| a7 1 2 3 4"], "{out:?}");
}

/// An `interface automatic` task, a task in a generate block of a
/// `module automatic`, and a 2-D local.
#[test]
fn interface_generate_and_2d_locals() {
    let out = t_lines(
        r#"
interface automatic ifc;
  task t(input int idx, output int r);
    int loc[2];
    loc[0] = idx; #1; r = loc[0];
  endtask
endinterface
module automatic top;
  ifc i();
  int res[6];
  if (1) begin : g
    task t(input int idx, output int r);
      int loc[2];
      loc[0] = idx; #1; r = loc[0];
    endtask
  end
  task static ts(input int idx, output int r);
    int loc[2];
    loc[0] = idx; #1; r = loc[0];
  endtask
  task t2(input int idx, output int r);
    int m[2][2];
    m[1][0] = idx; #1; r = m[1][0];
  endtask
  initial begin
    fork
      begin int r; i.t(1, r); res[0] = r; end
      begin int r; i.t(2, r); res[1] = r; end
      begin int r; g.t(3, r); res[2] = r; end
      begin int r; g.t(4, r); res[3] = r; end
      begin int r; t2(5, r); res[4] = r; end
      begin int r; t2(6, r); res[5] = r; end
    join
    $display("T| e2 %0d %0d %0d %0d %0d %0d", res[0], res[1], res[2], res[3], res[4], res[5]);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T| e2 1 2 3 4 5 6"], "{out:?}");
}

#[test]
fn explicit_automatic_task_locals_of_every_kind() {
    let out = t_lines(
        r#"
module top;
  int res[8];
  task automatic td(input int idx, output int r); int d[]; d = new[2]; d[0] = idx; #1; r = d[0]; endtask
  task automatic tq(input int idx, output int r); int q[$]; q.push_back(idx); #1; r = q[0]; endtask
  task automatic ta(input int idx, output int r); int aa[int]; aa[5] = idx; #1; r = aa[5]; endtask
  task automatic tm(input int idx, output int r); int m[2][2]; m[1][1] = idx; #1; r = m[1][1]; endtask
  initial begin
    fork
      begin int r; td(1, r); res[0]=r; end
      begin int r; td(2, r); res[1]=r; end
      begin int r; tq(1, r); res[2]=r; end
      begin int r; tq(2, r); res[3]=r; end
      begin int r; ta(1, r); res[4]=r; end
      begin int r; ta(2, r); res[5]=r; end
      begin int r; tm(1, r); res[6]=r; end
      begin int r; tm(2, r); res[7]=r; end
    join
    $display("T| d2 dyn %0d %0d q %0d %0d aa %0d %0d 2d %0d %0d", res[0],res[1],res[2],res[3],res[4],res[5],res[6],res[7]);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T| d2 dyn 1 2 q 1 2 aa 1 2 2d 1 2"], "{out:?}");
}
