//! Writes through the name of a local that hides a module, package or
//! `$unit` variable (IEEE 1800-2023 §23.9, §6.21, §13.3, §13.4, §8.10).
//!
//! A subroutine's locals and formals, a block's variables and a loop's
//! variable hide every outer variable of the same name, so a bit-, part- or
//! indexed part-select of one (§11.5.1) writes the local and nothing else.
//! So does a property of a class in its methods. The design covers blocking
//! and nonblocking selects, concatenation targets, compound and increment
//! forms, output arguments bound to a select, member selects of a packed
//! struct, package and `$unit` names, compiled function bodies, and the
//! static block variables of `always` and `initial` processes (§6.21).
//!
//! Two kinds of write still went wrong once plain selects reached the
//! local. A nonblocking select of a frame local (`a[3:0] <= v`, §10.4.2)
//! was queued and committed after the frame was gone, so it was lost or
//! wrote the module variable. A part-select of a member of a packed struct
//! local (`ps.hi[1:0] = v`) wrote the module's `ps`.
//!
//! The design checks itself. Every expected value follows the LRM, and
//! Verilator 5.038 (`--binary --timing`) passes all 63 checks. It runs with
//! the default method tiering, with every method and function compiled on
//! its first call, and with none compiled.

use std::process::Command;

const SRC: &str = r##"// $unit and package variables that subroutine locals shadow.
logic [7:0] uv = 8'h21;

package p;
  logic [7:0] pv = 8'h31;
endpackage

module sub;
  logic [7:0] sv = 8'h91;
  logic [7:0] got;
  task automatic t;
    logic [7:0] sv;
    sv = 8'h00;
    sv[3:0] = 4'h6;
    got = sv;
  endtask
endmodule

module top;
  import p::*;
  int errors = 0;
  task check(input string what, input logic [31:0] got, input logic [31:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", what, got, want);
    end else $display("PASS %s", what);
  endtask

  // Module variables named like the locals below. Every write through a
  // local's name must leave these alone.
  logic [7:0] a = 8'h11, b = 8'h12, c = 8'h13, d = 8'h14, e = 8'h15;
  logic [15:0] w = 16'h1617;
  logic [7:0] m [0:3];
  logic [8:1] r = 8'h18;
  logic [7:0] f = 8'h19, k = 8'h1A, s = 8'h1B, n = 8'h1C, x = 8'h1D, y = 8'h1E;
  logic [7:0] g = 8'h1F, h = 8'h20, cnt = 8'h23;
  integer i = 76;
  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } pair_t;
  pair_t ps = 8'h14;
  logic clk = 0;
  logic [7:0] r1, r2;

  // Bit-, part- and indexed part-selects, compound and increment forms.
  logic [7:0] o_part, o_bit, o_comp, o_inc, o_rd;
  logic [15:0] o_up, o_down;
  task automatic forms;
    logic [7:0] a;
    logic [15:0] w;
    a = 8'h00;
    a[3:0] = 4'h5;
    o_part = a;
    a[7] = 1'b1;
    o_bit = a;
    w = 16'h0000;
    for (int j = 0; j < 2; j++) w[j*8 +: 8] = 8'hA0 + j;
    o_up = w;
    w[15 -: 4] = 4'h3;
    o_down = w;
    a[3:0] += 4'h1;
    o_comp = a;
    a[7:4]++;
    o_inc = a;
    o_rd = a[7:4];
  endtask

  // A static task's locals, and nonblocking writes to them (§10.4.2).
  logic [7:0] o_static, o_nba, o_bnba, o_wnba;
  task stat;
    logic [7:0] b;
    b = 8'h00;
    b[3:0] = 4'h5;
    b[7:6] = 2'b11;
    o_static = b;
    b[5:4] <= 2'b10;
    #1 o_nba = b;
    begin
      logic [7:0] c;
      c = 8'h00;
      c[3:0] <= 4'hA;
      #1 o_bnba = c;
      c <= 8'h3C;
      #1 o_wnba = c;
    end
  endtask

  // An automatic task may schedule NBAs to explicitly static storage (§10.4.2).
  logic [7:0] o_anba, o_abit;
  task automatic auto_nba;
    static logic [7:0] a;
    a = 8'h00;
    a[3:0] <= 4'h5;
    #1 o_anba = a;
    a[7] <= 1'b1;
    #1 o_abit = a;
  endtask

  // Array elements and nested selects of an unpacked local, and a vector
  // local named like a module array.
  logic [7:0] o_e1, o_e2, o_e3, o_vec;
  task automatic arrays;
    logic [7:0] m [0:3];
    foreach (m[j]) m[j] = 8'h00;
    m[1] = 8'h12;
    m[2][3:0] = 4'h7;
    m[3][7] = 1'b1;
    o_e1 = m[1]; o_e2 = m[2]; o_e3 = m[3];
  endtask
  task automatic vec_named_like_array;
    logic [7:0] m;
    m = 8'h00;
    m[5:2] = 4'hF;
    o_vec = m;
  endtask

  // A concatenation of selects (§11.4.12).
  logic [7:0] o_ca, o_cb;
  task automatic concat;
    logic [7:0] c, d;
    c = 8'h00;
    d = 8'h00;
    {c[3:0], d} = 12'h9C3;
    {c[7:4], d[1:0]} = 6'b1011_10;
    o_ca = c; o_cb = d;
  endtask

  // Formals, a function's local, and a local declared with another range
  // than the module variable it hides.
  function automatic logic [7:0] f_auto(input logic [7:0] v);
    logic [7:0] f;
    f = v;
    f[0] = 1'b1;
    f[7:6] = 2'b01;
    return f;
  endfunction
  function logic [7:0] f_static(input logic [7:0] v);
    logic [7:0] f;
    f = v;
    f[7:4] = 4'hF;
    return f;
  endfunction
  function automatic logic [7:0] f_formal(input logic [7:0] k);
    k[0] = 1'b0;
    k[7:4] = 4'h3;
    return k;
  endfunction
  task automatic t_out(output logic [7:0] k);
    k = 8'h00;
    k[3:0] = 4'h5;
  endtask
  function automatic logic [7:0] f_range;
    logic [7:0] r;
    r = 8'h00;
    r[3:0] = 4'hA;
    return r;
  endfunction

  // Block and loop variables.
  logic [7:0] o_blk, o_loop;
  task automatic blocks;
    begin
      logic [7:0] e;
      e = 8'h00;
      e[5:4] = 2'b11;
      o_blk = e;
    end
    o_loop = 0;
    for (logic [7:0] g = 0; g < 8'h40; g = g + 8'h10) begin
      g[0] = 1'b1;
      o_loop = o_loop + 1;
    end
  endtask

  // Package and $unit variables.
  logic [7:0] o_pkg, o_unit;
  task automatic scopes;
    logic [7:0] pv, uv;
    pv = 8'h00;
    pv[3:0] = 4'h4;
    uv = 8'h00;
    uv[7:4] = 4'h4;
    o_pkg = pv; o_unit = uv;
  endtask

  // A packed struct local.
  logic [7:0] o_ps, o_ps2;
  task automatic structs;
    pair_t ps;
    ps = 8'h00;
    ps.lo = 4'h7;
    ps.hi[1:0] = 2'b10;
    o_ps = ps;
    ps.hi[2 +: 2] = 2'b11;
    o_ps2 = ps;
  endtask

  // An output argument bound to a select of a local.
  task automatic put4(output logic [3:0] v);
    v = 4'h9;
  endtask
  logic [7:0] o_arg;
  task automatic out_arg;
    logic [7:0] x;
    x = 8'h00;
    put4(x[7:4]);
    o_arg = x;
  endtask

  // Class methods: a method's local hides a property and the module
  // variable, and a property hides the module variable (§8.10).
  class C;
    logic [7:0] y = 8'h81;
    function logic [7:0] local_sel();
      logic [7:0] y;
      y = 8'h00;
      y[3:0] = 4'h6;
      return y;
    endfunction
    function void prop_sel();
      y[7:4] = 4'h9;
    endfunction
    task prop_nba();
      y[1:0] <= 2'b10;
      #1;
    endtask
    task prop_nba_whole();
      y <= 8'h5A;
      #1;
    endtask
    static function logic [7:0] static_sel();
      logic [7:0] y;
      y = 8'h00;
      y[1] = 1'b1;
      return y;
    endfunction
  endclass

  // Locals of the blocks of `always` and `initial` processes are static
  // (§6.21) and keep their values from one activation to the next.
  always @(posedge clk) begin : blk
    logic [7:0] cnt;
    if (cnt === 8'bx) cnt = 8'h00;
    cnt = cnt + 1;
    cnt[7] = 1'b1;
    r1 = cnt;
  end
  always @(posedge clk) begin
    logic [7:0] h;
    h = 8'h00;
    h[3:0] = 4'h5;
    h[7] = 1'b1;
    r2 = h;
  end

  sub u();

  initial begin
    C o;
    logic [7:0] v;
    foreach (m[j]) m[j] = 8'hEE;
    repeat (3) begin
      #1 clk = 1;
      #1 clk = 0;
    end
    check("always block variable", r1, 8'h83);
    check("unnamed always block variable", r2, 8'h85);
    forms;
    check("part-select", o_part, 8'h05);
    check("bit-select", o_bit, 8'h85);
    check("indexed part-select +:", o_up, 16'hA1A0);
    check("indexed part-select -:", o_down, 16'h31A0);
    check("compound assignment", o_comp, 8'h86);
    check("increment", o_inc, 8'h96);
    check("select read", o_rd, 8'h09);
    stat;
    check("static task local", o_static, 8'hC5);
    check("static nonblocking select", o_nba, 8'hE5);
    check("block nonblocking select", o_bnba, 8'h0A);
    check("block nonblocking write", o_wnba, 8'h3C);
    auto_nba;
    check("explicit static nonblocking part-select", o_anba, 8'h05);
    check("explicit static nonblocking bit-select", o_abit, 8'h85);
    arrays;
    check("array element", o_e1, 8'h12);
    check("array element part-select", o_e2, 8'h07);
    check("array element bit-select", o_e3, 8'h80);
    vec_named_like_array;
    check("vector named like an array", o_vec, 8'h3C);
    concat;
    check("concatenation: first", o_ca, 8'hB9);
    check("concatenation: second", o_cb, 8'hC2);
    check("automatic function local", f_auto(8'h12), 8'h53);
    check("static function local", f_static(8'h12), 8'hF2);
    check("function input", f_formal(8'hFF), 8'h3E);
    t_out(v);
    check("task output", v, 8'h05);
    check("local with another range", f_range(), 8'h0A);
    blocks;
    check("block variable", o_blk, 8'h30);
    check("loop variable", o_loop, 4);
    begin
      logic [7:0] d;
      d = 8'h00;
      d[1:0] = 2'b11;
      check("initial block variable", d, 8'h03);
      #1;
      d[7:6] = 2'b10;
      check("initial block variable across a delay", d, 8'h83);
    end
    for (int i = 0; i < 6; i++) i[0] = 1'b1;
    scopes;
    check("local named like a package variable", o_pkg, 8'h04);
    check("local named like a $unit variable", o_unit, 8'h40);
    structs;
    check("packed struct local", o_ps, 8'h27);
    check("packed struct member indexed part-select", o_ps2, 8'hE7);
    out_arg;
    check("output argument to a select", o_arg, 8'h90);
    o = new;
    check("method local", o.local_sel(), 8'h06);
    check("property after method local", o.y, 8'h81);
    o.prop_sel();
    check("property select", o.y, 8'h91);
    o.prop_nba();
    check("property nonblocking select", o.y, 8'h92);
    o.prop_nba_whole();
    check("property nonblocking write", o.y, 8'h5A);
    check("static method local", C::static_sel(), 8'h02);
    u.t;
    check("instance task local", u.got, 8'h06);
    // The module, package and $unit variables are untouched.
    check("module a", a, 8'h11);
    check("module b", b, 8'h12);
    check("module c", c, 8'h13);
    check("module d", d, 8'h14);
    check("module e", e, 8'h15);
    check("module w", w, 16'h1617);
    check("module m[2]", m[2], 8'hEE);
    check("module m[3]", m[3], 8'hEE);
    check("module r", r, 8'h18);
    check("module f", f, 8'h19);
    check("module k", k, 8'h1A);
    check("module x", x, 8'h1D);
    check("module y", y, 8'h1E);
    check("module g", g, 8'h1F);
    check("module h", h, 8'h20);
    check("module cnt", cnt, 8'h23);
    check("module i", i, 76);
    check("module ps", ps, 8'h14);
    check("package pv", pv, 8'h31);
    check("$unit uv", uv, 8'h21);
    check("instance sv", u.sv, 8'h91);
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
"##;

fn run_source(source: &str, env: &[(&str, &str)]) -> String {
    // Unique per call: tests run as parallel threads of one process.
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_local_shadow_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("top.sv");
    std::fs::write(&path, source).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "top",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .envs(env.iter().copied())
        .output()
        .expect("run xezim");
    assert!(
        out.status.success(),
        "simulator failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);
    text
}

fn check(env: &[(&str, &str)]) {
    let text = run_source(SRC, env);
    assert!(
        text.contains("ALL PASS") && !text.contains("FAIL"),
        "writes through a local that hides a module variable ({:?}):\n{}",
        env,
        text
    );
}

#[test]
fn local_shadow_select_writes() {
    check(&[]);
}

#[test]
fn local_shadow_select_writes_compiled_first_call() {
    check(&[("XEZIM_METHOD_TIER", "0")]);
}

#[test]
fn local_shadow_select_writes_interpreted() {
    check(&[("XEZIM_COMPILE_METHODS", "0")]);
}

// IEEE 1800-2023 §6.21, §7.4.1, §10.4.2: live storage and region ordering.
fn local_select_nba_neighbors_check(env: &[(&str, &str)]) {
    let text = run_source(include_str!("../issue_cases/pr368_nba_neighbors.sv"), env);
    assert!(
        text.contains("ALL PASS") && !text.contains("FAIL"),
        "{text}"
    );
}
#[test]
fn local_select_nba_neighbors() {
    local_select_nba_neighbors_check(&[]);
}
#[test]
fn local_select_nba_neighbors_eager() {
    local_select_nba_neighbors_check(&[("XEZIM_METHOD_TIER", "0")]);
}
#[test]
fn local_select_nba_neighbors_interpreted() {
    local_select_nba_neighbors_check(&[("XEZIM_COMPILE_METHODS", "0")]);
}
