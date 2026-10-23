//! §6.21: a block-scoped local that SHADOWS an enclosing local is scoped to
//! its block and vanishes on exit — it must not clobber the outer same-named
//! variable (the xezim frame model flattens `begin...end` and blocking loop
//! bodies, so a shadowing block-local can otherwise overwrite the enclosing
//! local's shared frame slot and never be restored).
//!
//! The canonical case is UVM's `uvm_reg::do_write`: a task-scoped
//! `uvm_reg_cb_iter cbs` is shadowed inside a `foreach (m_fields[i])` body by
//! a block-local `uvm_reg_field_cb_iter cbs`, and after the loop the register
//! reads its own (now-clobbered) iterator, so the register callback never
//! fires. These tests isolate the scoping rule in pure SystemVerilog.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    messages(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect()
}

fn messages(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

const COMMON: &str = r#"
class It;
  string tag; int cnt;
  function new(string t); tag = t; cnt = 0; endfunction
  function int more(); cnt++; return cnt; endfunction
endclass
"#;

/// A sequential `for` loop whose body re-declares a block-local named like a
/// task-local: after the loop the task-local must still hold its original
/// value (UVM `do_write`'s `foreach`, but observed with no blocking body).
#[test]
fn seq_for_body_local_redeclaring_task_local_is_scoped() {
    let src = format!(
        "{COMMON}{}",
        r#"
task automatic do_work();
  begin
    It cbs = new("REGISTER");
    int i;
    for (i = 0; i < 2; i++) begin
      It cbs = new("FIELD");
      void'(cbs.more());
    end
    $display("T|after field loop, cbs.tag=%s (expect REGISTER)", cbs.tag);
  end
endtask

module top;
  initial begin
    do_work();
  end
endmodule
"#
    );
    assert_eq!(
        messages(&src),
        ["T|after field loop, cbs.tag=REGISTER (expect REGISTER)"]
    );
}

/// A nested plain `begin...end` whose local shadows the enclosing block-local:
/// the inner value must not leak out.
#[test]
fn nested_block_local_does_not_leak_to_enclosing() {
    let src = format!(
        "{COMMON}{}",
        r#"
task automatic t;
  begin
    string a = "OUTER";
    begin
      string a = "INNER";
    end
    $display("T|a=%s (expect OUTER)", a);
  end
endtask

module top;
  initial begin
    t();
  end
endmodule
"#
    );
    assert_eq!(messages(&src), ["T|a=OUTER (expect OUTER)"]);
}

/// A `foreach` with a BLOCKING body (`#1` forces the suspend-aware trampoline,
/// which flattens the body's `begin...end`) re-declaring a block-local named
/// like a task-local: after the loop the task-local is intact. This is the
/// exact shape of UVM's `uvm_reg::do_write` field loop.
#[test]
fn blocking_foreach_body_local_redeclaring_task_local_is_scoped() {
    let src = format!(
        "{COMMON}{}",
        r#"
int fields[2];

task automatic do_write();
  begin
    It cbs = new("REGISTER");
    int i;
    foreach (fields[i]) begin
      It cbs = new("FIELD");
      #1;                    // blocking body: run via the suspend-aware trampoline
      void'(cbs.more());
    end
    $display("T|after field loop, cbs.tag=%s (expect REGISTER)", cbs.tag);
  end
endtask

module top;
  initial begin
    do_write();
  end
endmodule
"#
    );
    assert_eq!(
        messages(&src),
        ["T|after field loop, cbs.tag=REGISTER (expect REGISTER)"]
    );
}

/// Synchronous blocks (nested, loop bodies, `disable`, early `return`,
/// recursion, strings): a block-local shadowing an enclosing local or a
/// module variable ends with its block, and a `static` block-local keeps its
/// own cell without clobbering the automatic it shadows (§6.21, §13.3.2).
#[test]
fn sync_block_locals_end_with_their_block() {
    let src = r#"
module top;
  int g = 100;
  function automatic int fnest();
    int x = 1;
    begin
      int x = 2;
      begin
        int x = 3;
        x++;
      end
      $display("T|fnest mid x=%0d (2)", x);
    end
    return x;
  endfunction

  task automatic tloops();
    int x = 10;
    for (int k = 0; k < 3; k++) begin
      int x = k * 7;
    end
    $display("T|for x=%0d", x);
    repeat (2) begin
      int x = 99;
    end
    $display("T|repeat x=%0d", x);
    begin
      int j = 0;
      while (j < 2) begin
        int x = 55;
        j++;
      end
    end
    $display("T|while x=%0d", x);
    begin
      int arr[3] = '{1,2,3};
      foreach (arr[i]) begin
        int x = arr[i];
      end
    end
    $display("T|foreach x=%0d", x);
    for (int k = 0; k < 3; k++) begin
      int x = 77;
      if (k == 1) break;
    end
    $display("T|break x=%0d", x);
    for (int k = 0; k < 3; k++) begin
      int x = 88;
      continue;
    end
    $display("T|continue x=%0d", x);
  endtask

  task automatic tdisable();
    int x = 5;
    begin : blk
      int x = 6;
      disable blk;
      x = 7;
    end
    $display("T|disable x=%0d", x);
  endtask

  function automatic int fret(int a);
    int x = 1;
    begin
      int x = 50;
      if (a) return x;
    end
    return x;
  endfunction

  function automatic int frec(int n);
    int x = n;
    if (n == 0) return 0;
    begin
      int x = 1000;
      x = frec(n - 1) + x;
    end
    return x + frec(n - 1);
  endfunction

  task automatic tstatic();
    int x = 3;
    begin
      static int x = 40;
      x++;
    end
    $display("T|static-inner outer x=%0d", x);
  endtask

  task automatic tmod();
    int y = 1;
    begin
      int g = 7;
      y = g;
    end
    $display("T|modsig g=%0d y=%0d", g, y);
  endtask

  task automatic tstr();
    string s = "outer";
    begin
      string s = "inner";
      s = {s, "!"};
    end
    $display("T|str s=%s", s);
  endtask

  task automatic tnoouter();
    begin
      int z = 4;
    end
    begin
      int z;
      $display("T|fresh z=%0d (0)", z);
    end
  endtask

  initial begin
    $display("T|fnest ret=%0d (1)", fnest());
    tloops();
    tdisable();
    $display("T|fret1=%0d fret0=%0d", fret(1), fret(0));
    $display("T|frec=%0d", frec(3));
    tstatic();
    tstatic();
    tmod();
    tstr();
    tnoouter();
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|fnest mid x=2 (2)",
            "T|fnest ret=1 (1)",
            "T|for x=10",
            "T|repeat x=10",
            "T|while x=10",
            "T|foreach x=10",
            "T|break x=10",
            "T|continue x=10",
            "T|disable x=5",
            "T|fret1=50 fret0=1",
            "T|frec=6",
            "T|static-inner outer x=3",
            "T|static-inner outer x=3",
            "T|modsig g=100 y=7",
            "T|str s=outer",
            "T|fresh z=0 (0)",
        ]
    );
}
/// Blocking blocks: a block with `#1`, blocking `repeat`/`for`/`while`/
/// `foreach` bodies (with `break`, nesting and `return`) and `fork` inside a
/// shadowing block restore the enclosing local when the block ends.
#[test]
fn blocking_block_locals_end_with_their_block() {
    let src = r#"
module top;
  task automatic tdelay();
    int x = 1;
    begin
      int x = 2;
      #1;
      x = 3;
    end
    $display("T|delay-block x=%0d (1)", x);
  endtask
  task automatic trep();
    int x = 1;
    repeat (2) begin
      int x = 9;
      #1;
    end
    $display("T|repeat-block x=%0d (1)", x);
  endtask
  task automatic tfor();
    int x = 1;
    for (int k = 0; k < 2; k++) begin
      int x = 9;
      #1;
    end
    $display("T|for-block x=%0d (1)", x);
  endtask
  task automatic twhile();
    int x = 1; int j = 0;
    while (j < 2) begin
      int x = 9;
      #1; j++;
    end
    $display("T|while-block x=%0d (1)", x);
  endtask
  int arr[2];
  task automatic tforeach_break();
    int x = 1;
    foreach (arr[i]) begin
      int x = 9;
      #1;
      if (i == 0) break;
    end
    $display("T|foreach-break x=%0d (1)", x);
  endtask
  task automatic tforeach_nested();
    int x = 1;
    foreach (arr[i]) begin
      int x = 9;
      foreach (arr[j]) begin
        int x = 20;
        #1;
      end
      $display("T|foreach-nested mid x=%0d (9)", x);
    end
    $display("T|foreach-nested x=%0d (1)", x);
  endtask
  task automatic tforeach_ret();
    int x = 1;
    foreach (arr[i]) begin
      int x = 9;
      #1;
      return;
    end
  endtask
  task automatic tfork();
    int x = 1;
    begin
      int x = 2;
      fork
        begin #1; $display("T|fork child sees x=%0d (2)", x); end
      join_none
    end
    #2;
    $display("T|fork parent x=%0d (1)", x);
  endtask
  task automatic tfork_join();
    int x = 1;
    begin
      int x = 2;
      fork
        begin #1; x = 5; end
      join
      $display("T|fork-join inner x=%0d (5)", x);
    end
    $display("T|fork-join outer x=%0d (1)", x);
  endtask
  initial begin
    tdelay(); trep(); tfor(); twhile(); tforeach_break(); tforeach_nested();
    tforeach_ret(); $display("T|foreach-ret done");
    tfork(); tfork_join();
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|delay-block x=1 (1)",
            "T|repeat-block x=1 (1)",
            "T|for-block x=1 (1)",
            "T|while-block x=1 (1)",
            "T|foreach-break x=1 (1)",
            "T|foreach-nested mid x=9 (9)",
            "T|foreach-nested mid x=9 (9)",
            "T|foreach-nested x=1 (1)",
            "T|foreach-ret done",
            "T|fork child sees x=2 (2)",
            "T|fork parent x=1 (1)",
            "T|fork-join inner x=5 (5)",
            "T|fork-join outer x=1 (1)",
        ]
    );
}
/// In class methods: after a block whose local is named like a property
/// ends, the name is the property again (synchronous and blocking blocks).
#[test]
fn method_block_locals_do_not_hide_properties() {
    let src = r#"
class C;
  int p = 42;
  int q = 8;
  int arr[2];
  function int m1();
    int x = 1;
    begin
      int x = 2;
    end
    return x;
  endfunction
  function int m2();
    begin
      int p = 5;
      p++;
    end
    return p;
  endfunction
  task m3();
    int x = 1;
    foreach (arr[i]) begin
      int x = 3;
      #1;
    end
    $display("T|m3 x=%0d (1)", x);
  endtask
  task m4();
    begin
      int q = 77;
      #1;
    end
    $display("T|m4 q=%0d (8)", q);
  endtask
endclass
module top;
  initial begin
    C c = new;
    $display("T|m1=%0d (1)", c.m1());
    $display("T|m2=%0d (42)", c.m2());
    $display("T|p after=%0d (42)", c.p);
    c.m3();
    c.m4();
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|m1=1 (1)",
            "T|m2=42 (42)",
            "T|p after=42 (42)",
            "T|m3 x=1 (1)",
            "T|m4 q=8 (8)",
        ]
    );
}
/// Array and queue block-locals shadowing enclosing ones leave the outer
/// storage intact.
#[test]
fn array_and_queue_block_locals_are_scoped() {
    let src = r#"
module top;
  int fa[2];
  task automatic tarr();
    int a[3] = '{1,2,3};
    int q[$] = {7};
    begin
      int a[3] = '{9,9,9};
      int q[$] = {5,5,5};
      a[0] = 100;
    end
    $display("T|arr a=%0d,%0d,%0d q.size=%0d", a[0], a[1], a[2], q.size());
    foreach (fa[i]) begin
      int a[3] = '{4,4,4};
      #1;
    end
    $display("T|arr-foreach a=%0d,%0d,%0d", a[0], a[1], a[2]);
  endtask
  initial tarr();
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|arr a=1,2,3 q.size=1", "T|arr-foreach a=1,2,3",]
    );
}

/// Synchronous loops whose body declares a name the loop control reads (the
/// condition, the step, a `foreach` index), with `break`, `continue`,
/// `return` and recursion: every iteration's control sees the enclosing
/// variable, and so does the code after the loop.
#[test]
fn loop_control_sees_enclosing_names() {
    let src = r#"
module top;
  int arr[3] = '{1, 2, 3};
  function automatic int cond_shadow();
    int n = 3, cnt = 0;
    for (int k = 0; k < n; k++) begin
      int n = 100;
      cnt++;
    end
    return cnt * 1000 + n;
  endfunction
  function automatic int while_shadow();
    int n = 2, cnt = 0;
    while (n > 0) begin
      int n = 50;
      cnt++;
      if (cnt > 5) break;
    end
    return cnt;
  endfunction
  function automatic int step_shadow();
    int cnt = 0;
    for (int k = 0; k < 4; k = k + 1) begin
      int k2 = k * 10;
      cnt += k2;
    end
    return cnt;
  endfunction
  function automatic int foreach_shadow();
    int s = 0;
    int i = 77;
    foreach (arr[j]) begin
      int i = arr[j];
      s += i;
    end
    return s * 1000 + i;
  endfunction
  function automatic int ret_in_loop();
    int x = 1;
    repeat (3) begin
      int x = 9;
      if (x == 9) return x;
    end
    return x;
  endfunction
  function automatic int rec(int d);
    int x = d;
    if (d == 0) return 0;
    for (int k = 0; k < 2; k++) begin
      int x = 100;
      x += rec(d - 1);
    end
    return x;
  endfunction
  function automatic int dowhile_fresh();
    int tot = 0, k = 0;
    do begin
      int z;
      tot += z;
      z = 5;
      k++;
    end while (k < 3);
    return tot;
  endfunction
  function automatic int cont_loop();
    int x = 4, s = 0;
    for (int k = 0; k < 4; k++) begin
      int x = k;
      if (k == 1) continue;
      s += x;
    end
    return s * 100 + x;
  endfunction
  initial begin
    $display("T|cond_shadow=%0d", cond_shadow());
    $display("T|while_shadow=%0d", while_shadow());
    $display("T|step_shadow=%0d", step_shadow());
    $display("T|foreach_shadow=%0d", foreach_shadow());
    $display("T|ret_in_loop=%0d", ret_in_loop());
    $display("T|rec=%0d", rec(2));
    $display("T|dowhile_fresh=%0d", dowhile_fresh());
    $display("T|cont_loop=%0d", cont_loop());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|cond_shadow=3003",
            "T|while_shadow=6",
            "T|step_shadow=60",
            "T|foreach_shadow=6077",
            "T|ret_in_loop=9",
            "T|rec=2",
            "T|dowhile_fresh=0",
            "T|cont_loop=504",
        ]
    );
}
