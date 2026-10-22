//! §18.4/§18.5.9: rand handles held in arrays of objects, and members of
//! rand sub-objects wider than 64 bits or inside packed structs, join the
//! joint solve of the enclosing object (#255). Expected values come from
//! the reference simulator.

use xezim::simulate;

fn run(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

/// Fixed, dynamic and queue arrays of rand objects: a `foreach` over the
/// object array, parent items over element members, a null element skipped
/// and a non-rand handle array read as state.
#[test]
fn randomize_rand_object_arrays() {
    let out = run(OBJ_ARRAYS);
    assert_eq!(
        out,
        [
            "fixed_arr bad=0",
            "dyn_arr bad=0",
            "null_elem bad=0 x0=29 x2=27",
            "pair_only ok=1 x0=30 x1=20 y0=31",
        ],
        "{out:?}"
    );
}

/// Nested fields of a wide packed-struct member of a sub-object, elements of
/// a packed-array field (a random index included), and a field of a narrow
/// struct member, constrained from the parent; the evaluator reads them
/// through the handle.
#[test]
fn randomize_wide_sub_object_members() {
    let out = run(WIDE_SUB);
    assert_eq!(out, ["wide_top bad=0", "reader f=3 g=abcdef"], "{out:?}");
}

/// §18.5.10: `solve ... before` across the parent/child boundary, on a
/// member, a struct field and an array element of a sub-object.
#[test]
fn randomize_solve_before_sub_objects() {
    let out = run(SOLVE_BEFORE);
    assert_eq!(
        out,
        ["p1 c0_mid=1", "p2 t0_16th=1", "p3 t0_half=1"],
        "{out:?}"
    );
}

const OBJ_ARRAYS: &str = r#"
class leaf;
  rand bit [7:0] x;
  rand bit [7:0] y;
  constraint cx { x < 100; y == x + 1; }
endclass

// Fixed array: a foreach over the object array and parent items over elements.
class fixed_arr;
  rand leaf a[3];
  rand bit [7:0] s;
  function new(); foreach (a[i]) a[i] = new(); endfunction
  constraint c1 { foreach (a[i]) a[i].x > s; }
  constraint c2 { a[0].x + a[1].x + a[2].x == 150; }
  constraint c3 { s inside {[10:30]}; }
  constraint c4 { a[1].y == a[0].x + 3; }
endclass

// Dynamic array and queue of objects.
class dyn_arr;
  rand leaf d[];
  rand leaf q[$];
  rand bit [7:0] t;
  function new(); leaf c; d = new[2]; foreach (d[i]) d[i] = new(); c = new(); q.push_back(c); c = new(); q.push_back(c); endfunction
  constraint c1 { d[0].x + q[1].x == t; t == 120; }
  constraint c2 { foreach (q[i]) q[i].x == d[i].x + 10; }
endclass

// A null element is skipped; a non-rand handle array is state.
class null_elem;
  rand leaf a[3];
  leaf n[2];
  rand bit [7:0] u;
  function new(); a[0] = new(); a[2] = new(); n[0] = new(); n[1] = new(); n[0].x = 7; n[1].x = 9; endfunction
  constraint c1 { a[0].x + a[2].x == u; u == n[0].x + n[1].x + 40; }
  constraint c2 { a[0].x == a[2].x + 2; }
  constraint c3 { foreach (a[i]) if (a[i] != null) a[i].y > 3; }
endclass

// Only elements, no scalar of its own: a pair the trials rarely hit.
class pair_only;
  rand leaf a[2];
  function new(); foreach (a[i]) a[i] = new(); endfunction
  constraint c { a[0].x + a[1].x == 50; a[0].x == a[1].x + 10; }
endclass

module top;
  initial begin
    fixed_arr p; dyn_arr q; null_elem r; pair_only w; int ok, bad;
    p = new; bad = 0;
    for (int k = 0; k < 10; k++) begin
      ok = p.randomize();
      if (!ok || p.a[0].x + p.a[1].x + p.a[2].x != 150 || p.a[1].y != p.a[0].x + 3 || p.s < 10 || p.s > 30) bad++;
      foreach (p.a[i]) if (p.a[i].x <= p.s || p.a[i].x >= 100 || p.a[i].y != p.a[i].x + 1) bad++;
    end
    $display("fixed_arr bad=%0d", bad);
    q = new; bad = 0;
    for (int k = 0; k < 10; k++) begin
      ok = q.randomize();
      if (!ok || q.d[0].x + q.q[1].x != 120 || q.q[0].x != q.d[0].x + 10 || q.q[1].x != q.d[1].x + 10) bad++;
      foreach (q.d[i]) if (q.d[i].x >= 100 || q.d[i].y != q.d[i].x + 1) bad++;
    end
    $display("dyn_arr bad=%0d", bad);
    r = new; bad = 0;
    for (int k = 0; k < 10; k++) begin
      ok = r.randomize();
      if (!ok || r.u != 56 || r.a[0].x + r.a[2].x != 56 || r.a[0].x != r.a[2].x + 2 || r.a[1] != null) bad++;
    end
    $display("null_elem bad=%0d x0=%0d x2=%0d", bad, r.a[0].x, r.a[2].x);
    w = new;
    ok = w.randomize();
    $display("pair_only ok=%0d x0=%0d x1=%0d y0=%0d", ok, w.a[0].x, w.a[1].x, w.a[0].y);
  end
endmodule
"#;

const WIDE_SUB: &str = r#"
typedef struct packed { bit [3:0] a; bit [3:0] b; } in_t;
typedef struct packed { in_t hdr; bit [99:0] body; bit [63:0][23:0] lim; } w_t;
typedef struct packed { in_t hdr; bit [7:0] body; } n_t;

class wide_leaf; rand w_t r; endclass
class narrow_leaf; rand n_t r; endclass

// A nested field of a wide (>64-bit) packed-struct member of a sub-object,
// an element of a packed-array field, and a field of a narrow struct member.
class wide_top;
  rand wide_leaf w;
  rand wide_leaf e[2];
  rand narrow_leaf n;
  rand bit [3:0] k;
  rand bit [39:0] m;
  function new(); w = new; n = new; foreach (e[i]) e[i] = new; endfunction
  constraint c1 { w.r.hdr.a == k; k inside {[2:5]}; w.r.hdr.a + w.r.hdr.b == 10; }
  constraint c2 { n.r.hdr.a == w.r.hdr.b - 4; n.r.body == 8'h5A; }
  constraint c3 { w.r.body[99:90] == 10'h155; w.r.body[9:0] == e[0].r.body[9:0]; }
  constraint c4 { foreach (e[i]) e[i].r.lim[k] == 24'h12345 + i; e[1].r.lim[k+1] > e[1].r.lim[k]; }
  constraint c5 { m == (40'h800_0000 * w.r.hdr.b) - 1; }
endclass

// The evaluator reads a nested field and an element through a handle.
class reader;
  rand narrow_leaf n; rand wide_leaf w; int j;
  function new(); n = new; w = new; endfunction
  function int f(); return n.r.hdr.a; endfunction
  function int g(); return w.r.lim[j]; endfunction
endclass

module top;
  initial begin
    wide_top t; reader rd; int ok, bad;
    t = new; bad = 0;
    for (int i = 0; i < 10; i++) begin
      ok = t.randomize();
      if (!ok) bad++;
      if (t.w.r.hdr.a != t.k || t.k < 2 || t.k > 5 || t.w.r.hdr.a + t.w.r.hdr.b != 10) bad++;
      if (t.n.r.hdr.a != t.w.r.hdr.b - 4'd4 || t.n.r.body != 8'h5A) bad++;
      if (t.w.r.body[99:90] != 10'h155 || t.w.r.body[9:0] != t.e[0].r.body[9:0]) bad++;
      if (t.e[0].r.lim[t.k] != 24'h12345 || t.e[1].r.lim[t.k] != 24'h12346 || !(t.e[1].r.lim[t.k+1] > t.e[1].r.lim[t.k])) bad++;
      if (t.m != (40'h800_0000 * t.w.r.hdr.b) - 1) bad++;
    end
    $display("wide_top bad=%0d", bad);
    rd = new;
    rd.n.r = 16'h3456; rd.w.r.lim = '0; rd.w.r.lim[2] = 24'hABCDEF; rd.j = 2;
    $display("reader f=%0h g=%0h", rd.f(), rd.g());
  end
endmodule
"#;

const SOLVE_BEFORE: &str = r#"
// solve...before across the parent/child boundary (distribution, §18.5.10)
typedef struct packed { bit [3:0] n; bit [3:0] f; } h_t;
class C;
  rand bit [7:0] d;
  rand h_t h;
endclass
class P0;  // no ordering: (c==0) has 1 solution of 257
  rand bit c; rand C o;
  function new(); o = new; endfunction
  constraint k { (c == 0) -> (o.d == 0); }
endclass
class P1;  // parent first
  rand bit c; rand C o;
  function new(); o = new; endfunction
  constraint k { (c == 0) -> (o.d == 0); solve c before o.d; }
endclass
class P2;  // child field first: o.h.n uniform over 16 then t follows
  rand bit [7:0] t; rand C o;
  function new(); o = new; endfunction
  constraint k { (o.h.n == 0) -> (t == 0); (o.h.n != 0) -> (t > 0); solve o.h.n before t; }
endclass
class P3;  // array element child first
  rand bit [7:0] t; rand C a[2];
  function new(); foreach (a[i]) a[i] = new; endfunction
  constraint k { (a[1].d == 0) -> (t == 0); (a[1].d != 0) -> (t != 0); a[1].d < 2; solve a[1].d before t; }
endclass
module top;
  initial begin
    P0 p0; P1 p1; P2 p2; P3 p3; int z0, z1, z2, z3;
    p0 = new; p1 = new; p2 = new; p3 = new;
    z0 = 0; z1 = 0; z2 = 0; z3 = 0;
    for (int n = 0; n < 1000; n++) begin
      void'(p0.randomize()); if (p0.c == 0) z0++;
      void'(p1.randomize()); if (p1.c == 0) z1++;
      void'(p2.randomize()); if (p2.t == 0) z2++;
      void'(p3.randomize()); if (p3.t == 0) z3++;
    end
    $display("p1 c0_mid=%0d", (z1 > 400 && z1 < 600));
    $display("p2 t0_16th=%0d", (z2 > 30 && z2 < 100));
    $display("p3 t0_half=%0d", (z3 > 400 && z3 < 600));
  end
endmodule
"#;
