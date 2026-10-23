//! Select writes to `static` subroutine locals (IEEE 1800-2023 §6.21,
//! §11.5.1, §13.3, §13.4).
//!
//! A variable declared in a static task or function, or declared `static`
//! in any subroutine, is one variable shared by every call (§6.21). Its
//! bit-, part- and indexed part-selects (§11.5.1) write part of that
//! variable and leave the other bits as they were.
//!
//! Unless it is allocated with the module, such a variable has a live
//! shared cell, which every read uses, and a copy in the call frame. A
//! whole-variable write updated both, but a select or member write changed
//! only the frame copy, so it was lost: a static function building a carry
//! chain with `tb[i] = ...` on a local that starts all X returned its input
//! unchanged, and `a[15:8] = 8'hA5` left `a` all X. A nonblocking write to
//! a whole member of a static packed struct (`s.lo <= v`, §10.4.2) was
//! dropped as well.
//!
//! Neighbours of the fix are pinned as well: a block variable that hides a
//! static of the same name keeps its reads and writes to itself, and so does
//! a queue named like another method's static; an index that writes the
//! same static is evaluated before the select is applied; and every index
//! runs once, in order, even after an x index that discards the write
//! (§11.4.1, §11.5.1).
//!
//! The design checks itself. Verilator 5.038 (`--binary --timing`) passes
//! 30 of its 36 checks. The other six follow the LRM text and say so at the
//! check:
//! - two `$sscanf` checks: Verilator cannot build a `$sscanf` output that
//!   is a select with a function call in its index;
//! - two x-index checks: Verilator is two-state and writes bit 0 for an
//!   x index, where §11.5.1 discards the write (one of them also uses the
//!   `$sscanf` form);
//! - two nonblocking checks: Verilator commits a nonblocking write to a
//!   task's local at once, where §10.4.2 schedules it in the NBA region.
//!
//! It runs with the default method tiering, with every method and function
//! compiled on its first call, and with none compiled, so the interpreted
//! and compiled bodies and the switch from one to the other are all covered.

use std::process::Command;

const SRC: &str = r##"module top;
  int errors = 0;
  task check(input string what, input logic [31:0] got, input logic [31:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", what, got, want);
    end else $display("PASS %s", what);
  endtask

  // A static function whose bit-select writes build a carry chain into a
  // local that starts all X.
  function [3:0] incr(input [3:0] s);
    reg [3:0] v, tb;
    integer i;
    begin
      tb[0] = 1;
      v = s;
      for (i = 1; i < 4; i = i + 1) tb[i] = v[i-1] & tb[i-1];
      for (i = 0; i < 4; i = i + 1) if (tb[i] == 1) v[i] = ~v[i];
      incr = v;
    end
  endfunction

  // Constant, indexed and bit-selects on a local that starts all X.
  logic [15:0] r_const, r_up, r_down, r_bit;
  task sel_forms;
    reg [15:0] a, b, c, d;
    integer i;
    begin
      a[15:8] = 8'hA5;
      i = 1;
      b[i*8 +: 8] = 8'h3C;
      c[i*8 + 7 -: 8] = 8'h5A;
      for (i = 0; i < 16; i = i + 2) d[i] = 1'b1;
      r_const = a; r_up = b; r_down = c; r_bit = d;
    end
  endtask

  // The testbench idiom: fill a vector one element at a time.
  localparam int W = 8, N = 4;
  logic [N*W-1:0] r_fill;
  task fill;
    reg [W-1:0] v;
    reg [N*W-1:0] acc;
    integer i;
    begin
      v = 8'b10101010;
      for (i = 0; i < N; i = i + 1) acc[i*W +: W] = v + i;
      r_fill = acc;
    end
  endtask

  // An explicit static in an automatic task, written whole first.
  logic [15:0] r_whole;
  task automatic whole_first;
    static reg [15:0] a;
    a = 0;
    a[7:0] = 8'hAB;
    r_whole = a;
  endtask

  // Partial writes carry from one call to the next (§6.21).
  function [7:0] count(input clear);
    reg [7:0] c;
    begin
      if (clear) c = 0;
      c[7:4] = c[7:4] + 1;
      count = c;
    end
  endfunction

  // Nested selects, an unpacked array of vectors, two static locals.
  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } pair_t;
  logic [15:0] r_nest;
  logic [7:0] r_arr0, r_arr1, r_member;
  task nested;
    reg [1:0][7:0] p;
    reg [7:0] m [0:1];
    begin
      p[1] = 8'h12;
      p[0] = 8'h34;
      m[0][3:0] = 4'h3;
      m[1][7:4] = 4'hC;
      r_nest = p; r_arr0 = m[0]; r_arr1 = m[1];
    end
  endtask
  task automatic member;
    static pair_t s;
    s.lo = 4'h3;
    s.hi[3:2] = 2'b01;
    s.hi[1:0] = 2'b11;
    r_member = s;
  endtask
  function [15:0] two(input [7:0] x);
    reg [7:0] lo, hi;
    begin
      lo[3:0] = x[3:0];
      lo[7:4] = x[7:4];
      hi[0] = 1'b1;
      hi[7:1] = 7'd0;
      two = {hi, lo};
    end
  endfunction

  // Other writes in place: increment, compound assignment, a concatenation
  // target and an output argument bound to a part-select.
  logic [7:0] r_inc, r_cat, r_out;
  task setnib(output [3:0] o); o = 4'h7; endtask
  task others;
    reg [7:0] a, c, o;
    reg [3:0] b;
    begin
      a[3:0] = 4'h1;
      a[3:0]++;
      a[7:4] = 4'h0;
      a[7:4] += 4'h3;
      {c[7:4], b} = 8'h5C;
      c[3:0] = b;
      setnib(o[7:4]);
      o[3:0] = 4'h1;
      r_inc = a; r_cat = c; r_out = o;
    end
  endtask

  // A nonblocking select write commits in the NBA region (§10.4.2).
  logic [7:0] r_nba;
  task nba;
    reg [7:0] a;
    begin
      a[3:0] <= 4'h9;
      a[7:4] <= 4'h6;
      #1 r_nba = a;
    end
  endtask

  // A blocking select write after a nonblocking write to the whole
  // variable, and a select write from a fork child.
  logic [7:0] r_after, r_fork;
  task after_nba;
    reg [7:0] a, b;
    begin
      a <= 8'h55;
      #1 a[0] = 1'b0;
      b[7:4] = 4'h2;
      fork
        #1 b[3:0] = 4'h7;
      join
      r_after = a; r_fork = b;
    end
  endtask

  // A block variable hides the subroutine's static of the same name, so
  // its writes stay out of the static (§6.21).
  logic [7:0] r_inner, r_outer, r_loop_inner, r_loop_outer;
  task automatic hide;
    static reg [7:0] a = 8'hF0;
    begin
      automatic reg [7:0] a = 8'h00;
      a[0] = 1'b1;
      r_inner = a;
    end
    r_outer = a;
  endtask
  task automatic hide_loop;
    static reg [7:0] b = 8'hF0;
    for (int k = 0; k < 2; k++) begin
      automatic reg [7:0] b = 8'h00;
      b[k] = 1'b1;
      r_loop_inner = b;
    end
    r_loop_outer = b;
  endtask
  // The same for a static with no initializer.
  logic [7:0] r_bare_inner, r_bare_outer;
  task automatic hide_bare;
    static reg [7:0] a;
    a = 8'hF0;
    begin
      automatic reg [7:0] a;
      a = 8'h00;
      a[0] = 1'b1;
      r_bare_inner = a;
    end
    r_bare_outer = a;
  endtask

  // An index whose evaluation writes the same static is evaluated before
  // the select is applied (§11.4.1).
  function automatic logic [7:0] rec(input int n);
    static reg [7:0] a = 8'h00;
    if (n == 1) begin
      a = 8'hF0;
      return 0;
    end
    a[rec(1)] = 1'b1;
    return a;
  endfunction
  // The same through `$sscanf`, which takes its output target as written,
  // on a static with no initializer (LRM only: Verilator cannot build it).
  function automatic logic [7:0] rec_scan(input int n);
    static reg [7:0] a;
    if (n == 1) begin
      a = 8'hF0;
      return 0;
    end
    a = 8'h00;
    void'($sscanf("1", "%b", a[rec_scan(1)]));
    return a;
  endfunction

  // A side-effecting index runs once, and an x index discards the write
  // (§11.4.1, §11.5.1; LRM only: Verilator writes bit 0 for an x index and
  // cannot build the `$sscanf` form).
  int idx_calls = 0;
  logic [7:0] r_pick;
  function automatic logic [31:0] pick(input bit unknown);
    idx_calls++;
    return unknown ? 32'bx : 32'd3;
  endfunction
  logic [7:0] r_scan, r_nba_idx;
  logic [15:0] r_calls;
  task automatic pick_task;
    static reg [7:0] a;
    a = 8'h00;
    a[pick(0)] = 1'b1;
    a[pick(1)] = 1'b1;
    r_pick = a;
  endtask
  task automatic pick_scan;
    static reg [7:0] a;
    a = 8'h00;
    idx_calls = 0;
    void'($sscanf("1", "%b", a[pick(1)]));
    r_calls[15:8] = idx_calls;
    r_scan = a;
  endtask
  task automatic pick_nba;
    static reg [7:0] a;
    int i;
    a = 8'h00;
    idx_calls = 0;
    a[pick(1)] <= 1'b1;
    i = 1;
    a[i] <= 1'b1;
    i = 5;
    #1 r_calls[7:0] = idx_calls;
    r_nba_idx = a;
  endtask

  // Nonblocking writes to locals of a static task that the task writes
  // whole first commit in the NBA region, not at once (§10.4.2; LRM only:
  // Verilator commits them at once).
  logic [7:0] r_early_b, r_late_b, r_early_c, r_late_c;
  task nba_frame;
    reg [7:0] b, c;
    begin
      b = 8'h00;
      c = 8'h00;
      b[2] <= 1'b1;
      c <= 8'h11;
      r_early_b = b;
      r_early_c = c;
      #1 r_late_b = b;
      r_late_c = c;
    end
  endtask

  // A nonblocking select write keeps the static's own type when the
  // task has returned to a caller with a two-state variable of the same
  // name before the NBA region (§6.11.2, §10.4.2).
  logic [7:0] r_type;
  task automatic xnba(input bit read, output logic [7:0] q);
    static logic [7:0] a;
    if (read) begin
      q = a;
      return;
    end
    a = 8'h00;
    a[0] <= 1'bx;
  endtask
  task automatic xcaller;
    bit [7:0] a;
    logic [7:0] q;
    a = 8'hFF;
    xnba(0, q);
    #1 xnba(1, q);
    r_type = q;
  endtask

  // A nonblocking member select of a static packed struct, read in the
  // same time slot, after it, and after a delayed write to a whole member
  // (§10.4.2; LRM only for the first read: Verilator commits the
  // nonblocking write at once).
  logic [7:0] r_m_now, r_m_next, r_m_late;
  task automatic member_nba;
    static pair_t s;
    s = 8'h00;
    s.hi[3:2] <= 2'b01;
    s.lo <= #2 4'h5;
    r_m_now = s;
    #1 r_m_next = s;
    #2 r_m_late = s;
  endtask

  // Delayed nonblocking writes wait for their own time slot (§9.4.5).
  logic [7:0] r_d_early, r_d_late, r_dw_early, r_dw_late;
  task automatic dnba;
    static logic [7:0] a, b;
    a = 8'h00;
    b = 8'h00;
    a[0] <= #5 1'b1;
    a[7:4] <= #5 4'hA;
    b <= #5 8'h33;
    #1 r_d_early = a;
    r_dw_early = b;
    #5 r_d_late = a;
    r_dw_late = b;
  endtask

  // Static locals of class methods.
  class C;
    function logic [7:0] m(input logic [3:0] x);
      static logic [7:0] q;
      q[3:0] = x;
      q[7:4] = 4'h4;
      return q;
    endfunction
    static function logic [7:0] sm(input logic [3:0] x);
      static logic [7:0] q;
      q[3:0] = x;
      q[7:4] = 4'h3;
      return q;
    endfunction
    function logic [7:0] tick(input logic clear);
      static logic [7:0] c;
      if (clear) c = 8'h0F;
      c[7:4] = c[7:4] + 1;
      return c;
    endfunction
    task shadow(output logic [7:0] i, output logic [7:0] o);
      static logic [7:0] a = 8'hF0;
      begin
        automatic logic [7:0] a = 8'h00;
        a[0] = 1'b1;
        i = a;
      end
      o = a;
    endtask
    // A queue named like the static `q` of `m` keeps its own writes, with
    // `$` naming its last element (§7.10.1).
    function int last();
      int q[$];
      q = '{1, 2, 3};
      void'($sscanf("7", "%d", q[$]));
      return q[2];
    endfunction
    // An x index discards the write, but the index after it still runs
    // (§11.4.1, §11.5.1; LRM only: Verilator cannot build it).
    function logic [15:0] pair();
      static logic [1:0][7:0] p;
      p = 16'h1234;
      void'($sscanf("1", "%b", p[pick(1)][pick(0)]));
      return p;
    endfunction
  endclass

  initial begin
    C o;
    logic [7:0] c1, c2, c3;
    logic [15:0] w;
    check("static function bit-selects", {incr(5), incr(7), incr(15)}, {20'd0, 4'd6, 4'd8, 4'd0});
    sel_forms;
    check("constant part-select", r_const, 16'hA5xx);
    check("indexed +: part-select", r_up, 16'h3Cxx);
    check("indexed -: part-select", r_down, 16'h5Axx);
    check("bit-select loop", r_bit, 16'bx1x1x1x1x1x1x1x1);
    fill;
    check("indexed part-select loop", r_fill, 32'hADACABAA);
    whole_first;
    check("explicit static written whole first", r_whole, 16'h00AB);
    c1 = count(1);
    c2 = count(0);
    c3 = count(0);
    check("partial writes persist across calls", {c1, c2, c3}, 24'h102030);
    nested;
    check("packed array element select", r_nest, 16'h1234);
    check("unpacked array element select", {r_arr0, r_arr1}, 16'bxxxx0011_1100xxxx);
    member;
    check("packed struct member", r_member, 8'h73);
    check("two static locals", two(8'hA5), 16'h01A5);
    others;
    check("increment and compound assignment", r_inc, 8'h32);
    check("concatenation target", r_cat, 8'h5C);
    check("output argument", r_out, 8'h71);
    nba;
    check("nonblocking part-select", r_nba, 8'h69);
    after_nba;
    check("select write after a nonblocking write", r_after, 8'h54);
    check("select write from a fork child", r_fork, 8'h27);
    hide;
    check("block variable hiding a static", {r_inner, r_outer}, 16'h01F0);
    hide_loop;
    check("loop block variable hiding a static", {r_loop_inner, r_loop_outer}, 16'h02F0);
    hide_bare;
    check("block variable hiding an uninitialized static", {r_bare_inner, r_bare_outer}, 16'h01F0);
    check("index that writes the same static", rec(0), 8'hF1);
    check("index that writes the same static, through $sscanf", rec_scan(0), 8'hF1);
    pick_task;
    check("index with a side effect runs once", {r_pick, idx_calls[7:0]}, 16'h0802);
    pick_scan;
    pick_nba;
    check("x index from a function, $sscanf and nonblocking", {r_calls, r_scan, r_nba_idx}, 32'h0101_0002);
    nba_frame;
    check("nonblocking writes commit in the NBA region", {r_early_b, r_late_b, r_early_c, r_late_c}, 32'h0004_0011);
    xcaller;
    check("nonblocking select write keeps the static's type", r_type, 8'b0000000x);
    dnba;
    check("delayed nonblocking writes", {r_d_early, r_d_late, r_dw_early, r_dw_late}, 32'h00A1_0033);
    member_nba;
    check("nonblocking member select", {r_m_now, r_m_next, r_m_late}, 24'h004045);
    o = new;
    check("class method static local", o.m(4'h5), 8'h45);
    check("static class method static local", C::sm(4'h5), 8'h35);
    o.shadow(c1, c2);
    check("method block variable hiding a static", {c1, c2}, 16'h01F0);
    check("queue named like a static", o.last(), 7);
    idx_calls = 0;
    w = o.pair();
    check("x index before another index call", {w, idx_calls[7:0]}, 24'h1234_02);
    // Enough calls for a compiled body to take over part way.
    c1 = count(1);
    repeat (4999) c2 = count(0);
    check("function after many calls", count(0), 8'h90);
    c1 = o.tick(1);
    repeat (4999) c2 = o.tick(0);
    check("method after many calls", o.tick(0), 8'h9F);
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
"##;

fn check(env: &[(&str, &str)]) {
    // Unique per call: tests run as parallel threads of one process.
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_static_select_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("top.sv");
    std::fs::write(&path, SRC).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top"])
        .arg(&path)
        .env_remove("XEZIM_METHOD_TIER")
        .env_remove("XEZIM_COMPILE_METHODS")
        .envs(env.iter().copied())
        .output()
        .expect("run xezim");
    std::fs::remove_dir_all(&dir).unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success() && text.contains("ALL PASS") && !text.contains("FAIL"),
        "select writes to static locals ({env:?}):\n{text}"
    );
}

#[test]
fn static_local_select_writes() {
    check(&[]);
}

#[test]
fn static_local_select_writes_compiled_first_call() {
    check(&[("XEZIM_METHOD_TIER", "0")]);
}

#[test]
fn static_local_select_writes_interpreted() {
    check(&[("XEZIM_COMPILE_METHODS", "0")]);
}
