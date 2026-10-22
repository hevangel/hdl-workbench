//! Nested selects of packed task and function locals (IEEE 1800-2023
//! §7.4.1, §7.2.1, §11.5.1).
//!
//! A select of a packed array element, or of a packed struct or union
//! member, names part of the variable: a write changes only those bits, and
//! a read returns them (§7.4.1, §11.5.1). The left bound of a packed
//! dimension labels its most significant element or bit, whatever the
//! direction, and the first member of a packed struct is its most
//! significant (§7.2.1).
//!
//! The select arms that store a nested select (`p[1][3:0] = v`,
//! `p[0][7] = b`, `a[i].f[3] = b`) of a multidimensional packed variable or
//! a packed array of structs look the variable up in the module signal table
//! only. A local of a task or function lives in its call frame, so the write
//! was dropped, while the same write to a module variable worked. A part- or
//! bit-select below a struct member read and wrote the member's raw bit
//! positions, so `e.f[4:1]` of a member declared `[8:1]` missed by one, and
//! an element of a packed array of a struct typedef declared ascending
//! (`e_t [0:1] b`) was taken from the wrong end.
//!
//! An enum's bits are laid out by its base type (§6.19), so a select below
//! an enum member or element declared `enum logic [8:1]` uses the base
//! type's labels too, not `[w-1:0]`.
//!
//! The design checks itself: 2-D, 3-D and 4-D locals with constant and
//! variable indices, `+:` and `-:` after an element select, element ranges
//! of a middle dimension, dimensions that are not `[n-1:0]`, struct and
//! union members, enum members and elements whose base type is not
//! `[n-1:0]`, packed arrays of structs, selects out of range and with x
//! indices, a 2-state local, an output formal, a return value, a block
//! variable, static subroutines, and a function called 5,000 times. Every
//! expected value is confirmed by the reference simulator, including X/Z and
//! partially out-of-range accesses. The design runs with default method
//! tiering, eager compilation, and interpreted methods.

use std::process::Command;

const SRC: &str = r##"module top;
  typedef struct packed { logic [3:0] a; logic [3:0] b; } s_t;
  typedef struct packed { logic [8:1] f; logic [0:3] g; } e_t;
  typedef union packed { logic [7:0] w; s_t s; } u_t;
  typedef struct packed { logic [1:0][3:0] m; e_t e; } c_t;
  typedef enum logic [8:1] { E8_0 = 8'h00, E8_5A = 8'h5A } e8_t;
  typedef logic [0:3] nib_t;
  typedef enum nib_t { EN_0 = 4'h0, EN_9 = 4'h9 } en_t;
  typedef struct packed { e8_t f; en_t g; } se_t;
  int errors = 0;
  task automatic check(input string what, input logic [63:0] got, input logic [63:0] want);
    if (got !== want) begin
      errors++;
      $display("FAIL %s: got %h, want %h", what, got, want);
    end else $display("PASS %s", what);
  endtask

  // The issue's case: 2-D, constant indices.
  task automatic two_d;
    logic [1:0][7:0] p;
    p = '0;
    p[1][3:0] = 4'h9;
    p[0][7] = 1'b1;
    check("2-D constant", p, 16'h0980);
    check("2-D constant read", {p[1][3:0], p[0][7]}, 5'b1001_1);
  endtask

  // 2-D, variable indices, and +: / -: after an element select.
  task automatic two_d_var;
    logic [1:0][7:0] p;
    logic [3:0][7:0] q;
    int i, j;
    p = '0;
    q = '0;
    i = 1;
    j = 2;
    p[i][j] = 1'b1;
    p[i-1][j+3 -: 2] = 2'b11;
    p[i][7:6] = 2'b10;
    check("2-D variable", p, 16'h8430);
    check("2-D variable read", {p[i][j], p[i-1][j+3 -: 2]}, 3'b111);
    q[2][j +: 4] = 4'hF;
    q[3][7 -: 3] = 3'b101;
    q[0][0 +: 8] = 8'h5A;
    check("+: and -:", q, 32'hA03C005A);
    check("+: read", q[2][j +: 4], 4'hF);
  endtask

  // 3-D and 4-D, including an element range of a middle dimension.
  task automatic deeper;
    logic [1:0][2:0][3:0] p;
    logic [1:0][1:0][1:0][3:0] q;
    int i;
    p = '0;
    i = 1;
    p[1][2][3] = 1'b1;
    p[0][1][2:1] = 2'b11;
    p[i][0] = 4'hC;
    p[0][i+1][i +: 2] = 2'b10;
    check("3-D", p, 24'h80C460);
    check("3-D read", {p[1][2][3], p[0][1][2:1], p[i][0], p[0][i+1][i +: 2]}, 9'b1_11_1100_10);
    p[1][2:1] = 8'hAB;
    p[0][0 +: 2] = 8'hCD;
    check("3-D element range", p, 24'hABC_4CD);
    check("3-D element range read", p[1][2:1], 8'hAB);
    q = '0;
    q[1][0][1][2] = 1'b1;
    q[0][1][0][3:1] = 3'b101;
    q[1][1][0 +: 1] = 4'h7;
    check("4-D", q, 32'h0740_0A00);
  endtask

  // Dimensions that are not [n-1:0] (§7.4.1: the left bound is the MSB).
  task automatic labels;
    logic [2:1][8:1] p;
    logic [0:1][0:7] q;
    p = '0;
    q = '0;
    p[2][4:1] = 4'h5;
    p[1][8] = 1'b1;
    q[0][0:3] = 4'hA;
    q[1][2 +: 3] = 3'b101;
    q[1][7] = 1'b1;
    check("non-zero-based", p, 16'h0580);
    check("ascending", q, 16'hA029);
    check("label reads", {p[2][4:1], p[1][8], q[0][0:3], q[1][2 +: 3]}, 12'b0101_1_1010_101);
  endtask

  // Packed struct and union members.
  task automatic members;
    s_t s;
    e_t e;
    u_t u;
    c_t c;
    s = '0;
    e = '0;
    u = '0;
    c = '0;
    s.a[3:2] = 2'b11;
    s.b[0] = 1'b1;
    check("struct member", s, 8'hC1);
    check("struct member read", {s.a[3:2], s.b[0]}, 3'b11_1);
    e.f[4:1] = 4'hB;
    e.g[0:1] = 2'b11;
    e.g[3] = 1'b1;
    check("member labels", e, 12'h0BD);
    check("member labels read", {e.f[4:1], e.g[0:1], e.g[3]}, 7'b1011_11_1);
    u.s.b[2:1] = 2'b11;
    u.w[7] = 1'b1;
    check("union member", u, 8'h86);
    c.m[1][3:0] = 4'h9;
    c.m[0][2] = 1'b1;
    c.e.f[8:5] = 4'h3;
    c.e.g[1 +: 2] = 2'b10;
    check("struct of packed array and struct", c, 20'h94_304);
    check("nested member read", {c.m[1][3:0], c.e.f[8:5], c.e.g[1 +: 2]}, 10'b1001_0011_10);
  endtask

  // Enum members and elements: the base type's range labels the bits
  // (§6.19, §7.4.1), through a typedef too; the default base is int.
  task automatic enum_members;
    se_t s;
    e8_t [1:0] a;
    struct packed { enum logic [8:1] { AE_0, AE_1 } f; enum { AH_0, AH_1 } h; } n;
    s = 12'h5A9;
    check("enum member read", {s.f[8:5], s.f[1], s.g[0:1], s.g[3]}, 8'b0101_0_10_1);
    s = '0;
    s.f[1] = 1'b1;
    s.f[8:7] = 2'b11;
    s.g[0] = 1'b1;
    check("enum members", s, 12'hC18);
    a = '0;
    a[1][1] = 1'b1;
    a[0][8 -: 2] = 2'b10;
    check("array of enums", a, 16'h0180);
    check("array of enums read", {a[1][2:1], a[0][8]}, 3'b01_1);
    n = '0;
    n.f[1] = 1'b1;
    n.h[31] = 1'b1;
    check("anonymous enum members", n, 40'h01_8000_0000);
    check("anonymous enum members read", {n.f[2:1], n.h[31:30]}, 4'b01_10);
  endtask

  // Packed arrays of packed structs.
  task automatic struct_arrays;
    s_t [2:0] a;
    e_t [0:1] b;
    int i;
    a = '0;
    b = '0;
    a[1].a = 4'h5;
    a[2].b[1:0] = 2'b10;
    i = 0;
    a[i].a[3] = 1'b1;
    a[i+1][1:0] = 2'b11;
    check("array of structs", a, 24'h02_53_80);
    check("array of structs read", {a[2].b[1:0], a[i].a[3], a[1][1:0]}, 5'b10_1_11);
    b[0].f[4:1] = 4'hB;
    b[1].g = 4'h3;
    b[1].g[0] = 1'b1;
    check("ascending array of structs", b, 24'h0B0_00B);
    check("ascending array of structs read", {b[0].f[4:1], b[1].g, b[1]}, 20'hB_B_00B);
  endtask

  // A select that names no element writes nothing and reads x (§11.5.1);
  // one partly out of range writes the bits that exist.
  task automatic out_of_range;
    logic [1:0][7:0] p;
    bit [1:0][7:0] t;
    int i;
    integer x;
    p = '0;
    i = 2;
    x = 'x;
    p[i][3:0] = 4'hF;
    p[x][3:0] = 4'hF;
    p[0][x] = 1'b1;
    check("no element", p, 16'h0000);
    p = '0;
    p[1][9 -: 4] = 4'hF;
    check("partly out of range", p, 16'hC000);
    check("out-of-range read", p[i][3:0], 4'hx);
    check("partly out-of-range read", p[1][9 -: 4], 4'bxx11);
    t = '0;
    check("2-state out-of-range read", t[i][3:0], 4'h0);
    t[1][3:0] = 4'bx01z;
    check("2-state stores 0 for x/z", t, 16'h0200);
  endtask

  // Output formal, return value and a block variable.
  task automatic out_arg(output logic [1:0][7:0] o);
    o = '0;
    o[1][3:0] = 4'h9;
    o[0][7] = 1'b1;
  endtask

  function automatic logic [1:0][7:0] ret(input logic [3:0] v);
    ret = '0;
    ret[1][3:0] = v;
    ret[0][7] = 1'b1;
  endfunction

  task automatic block;
    begin : inner
      logic [1:0][2:0][3:0] q;
      q = '0;
      q[0][1][2:1] = 2'b11;
      q[1][2][3] = 1'b1;
      check("block variable", q, 24'h800_060);
    end
  endtask

  // Static subroutines.
  task two_d_static;
    logic [1:0][7:0] p;
    p = '0;
    p[1][3:0] = 4'h9;
    p[0][7] = 1'b1;
    check("static task", p, 16'h0980);
  endtask

  function logic [15:0] fill_static(input logic [3:0] v);
    logic [3:0][3:0] p;
    p = '0;
    for (int k = 0; k < 4; k++) p[k][k] = v[k];
    return p;
  endfunction

  // Called often enough for compiled bodies to take over.
  function automatic logic [15:0] fill(input logic [3:0] v, input int k);
    logic [1:0][7:0] p;
    p = '0;
    p[1][3:0] = v;
    p[0][k] = 1'b1;
    return p;
  endfunction

  initial begin
    logic [1:0][7:0] o;
    int bad;
    two_d;
    two_d_var;
    deeper;
    labels;
    members;
    enum_members;
    struct_arrays;
    out_of_range;
    out_arg(o);
    check("output formal", o, 16'h0980);
    check("return value", ret(4'h9), 16'h0980);
    block;
    two_d_static;
    check("static function", fill_static(4'b1011), 16'h8021);
    bad = 0;
    for (int n = 0; n < 5000; n++)
      if (fill(n[3:0], n % 8) !== {4'h0, n[3:0], 8'h1 << (n % 8)}) bad++;
    check("many calls", bad, 0);
    if (errors == 0) $display("ALL PASS");
    $finish;
  end
endmodule
"##;

fn run_source(source: &str, env: &[(&str, &str)]) -> String {
    // Unique per call: tests run as parallel threads of one process.
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_packed_local_select_{}_{}",
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
        "nested selects of packed locals ({:?}):\n{}",
        env,
        text
    );
}

#[test]
fn packed_local_nested_selects() {
    check(&[]);
}

#[test]
fn packed_local_nested_selects_compiled_first_call() {
    check(&[("XEZIM_METHOD_TIER", "0")]);
}

#[test]
fn packed_local_nested_selects_interpreted() {
    check(&[("XEZIM_COMPILE_METHODS", "0")]);
}

// IEEE 1800-2023 §6.21, §7.4.1, §10.4.2: live storage and region ordering.
fn packed_local_static_neighbors_check(env: &[(&str, &str)]) {
    let text = run_source(
        include_str!("../issue_cases/pr369_static_neighbors.sv"),
        env,
    );
    assert!(
        text.contains("ALL PASS") && !text.contains("FAIL"),
        "{text}"
    );
}
#[test]
fn packed_local_static_neighbors() {
    packed_local_static_neighbors_check(&[]);
}
#[test]
fn packed_local_static_neighbors_eager() {
    packed_local_static_neighbors_check(&[("XEZIM_METHOD_TIER", "0")]);
}
#[test]
fn packed_local_static_neighbors_interpreted() {
    packed_local_static_neighbors_check(&[("XEZIM_COMPILE_METHODS", "0")]);
}
