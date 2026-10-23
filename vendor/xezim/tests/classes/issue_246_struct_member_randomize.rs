//! Issue #246: `randomize()` over struct FIELDS, and the storage views of a
//! nested struct member. The reporter's MWE runs verbatim in
//! `issue_cases_runner`; these pin each fix on its own.

use xezim::simulate;

fn tagged(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T "))
        .collect()
}

/// A module-scope write through a handle to a NESTED member
/// (`obj.s.f1.f2`, four segments or more) went to name-keyed storage that
/// every object shared and no method saw; a method's write was invisible to
/// the handle read. Both views must alias the one per-object property.
#[test]
fn nested_member_handle_and_method_views_share_storage() {
    let out = tagged(
        r#"
typedef struct packed { bit [3:0] lo; bit [3:0] hi; } leaf_t;
typedef struct packed { leaf_t a; bit [7:0] b; } mid_t;
typedef struct packed { mid_t m; bit [7:0] z; } top_t;
class W;
  top_t t;
  function int rd(); return t.m.a.hi; endfunction
  function void wr(bit [3:0] x); t.m.a.lo = x; endfunction
endclass
module top;
  initial begin
    W p = new, q = new;
    p.t.m.a.hi = 4'h9;
    q.t.m.a.hi = 4'h3;
    p.wr(4'h5);
    $display("T p=%h q=%h", p.t, q.t);
    $display("T method reads p=%0h q=%0h", p.rd(), q.rd());
    $display("T handle reads lo p=%0h q=%0h", p.t.m.a.lo, q.t.m.a.lo);
  end
endmodule
"#,
    );
    assert_eq!(
        out,
        [
            "T p=590000 q=030000",
            "T method reads p=9 q=3",
            "T handle reads lo p=5 q=0",
        ]
    );
}

/// §18.4: a member, a nested member or a select of a member of a `rand`
/// packed struct is a solvable target for every constraint form, not only
/// one-level equality. Each used to draw the whole struct uniformly and
/// filter, so narrow ranges exhausted every trial and randomize() failed.
#[test]
fn constraints_on_struct_member_targets_are_solved() {
    let out = tagged(
        r#"
typedef struct packed { bit [31:0] F1; bit [7:0] F2; } sub_t;
typedef struct packed { sub_t N; bit [31:0] P; bit [31:0] M; } regs_t;
class C;
  rand regs_t s;
  bit sel;
  constraint c_dist   { s.M dist { [7:7] :/ 1 }; }
  constraint c_part   { s.P[15:0] == 16'd7; s.P[31:16] inside { [8:8] }; }
  constraint c_nested { s.N.F1 == 32'd9; }
  constraint c_rel    { 32'd5 > s.N.F2; s.N.F2 > 8'd2; }
endclass
class I;
  rand regs_t s;
  bit sel;
  constraint c { if (sel) { s.M dist { [3:3] :/ 1 }; } else { s.M inside { [4:4] }; } }
endclass
module top;
  initial begin
    C c = new; I i = new; int r, bad;
    for (int k = 0; k < 50; k++) begin
      r = c.randomize();
      if (!r || c.s.M != 7 || c.s.P != 32'h0008_0007 || c.s.N.F1 != 9
          || !(c.s.N.F2 inside {[3:4]})) bad++;
    end
    $display("T member targets bad=%0d", bad);
    i.sel = 1; r = i.randomize(); $display("T if-dist r=%0d M=%0d", r, i.s.M);
    i.sel = 0; r = i.randomize(); $display("T else-inside r=%0d M=%0d", r, i.s.M);
  end
endmodule
"#,
    );
    assert_eq!(
        out,
        [
            "T member targets bad=0",
            "T if-dist r=1 M=3",
            "T else-inside r=1 M=4",
        ]
    );
}

/// §18.5.4: a member `dist` follows its weights — alone, next to a dist on
/// another member, and next to a relational constraint on a sibling member.
/// The sibling used to send the whole set to the joint solver, which treats
/// the struct as one variable and drew the member uniformly.
#[test]
fn member_dist_follows_its_weights() {
    let out = tagged(
        r#"
typedef struct packed { bit [7:0] a; bit [15:0] b; bit [7:0] c; } s_t;
class Alone; rand s_t s; constraint c1 { s.a dist { 0 := 1, 1 := 3 }; } endclass
class Pair;
  rand s_t s;
  constraint c1 { s.a dist { 0 := 1, 1 := 3 }; }
  constraint c2 { s.b[3:0] dist { [0:1] :/ 1, [2:9] :/ 3 }; }
  constraint c3 { s.c < 8'd10; s.c > 8'd2; }
endclass
module top;
  initial begin
    Alone x = new; Pair y = new; int x0, y0, ylo, bad;
    for (int k = 0; k < 4000; k++) begin
      if (!x.randomize() || x.s.a > 1) bad++;
      if (!y.randomize() || y.s.a > 1 || y.s.b[3:0] > 9 || !(y.s.c inside {[3:9]})) bad++;
      if (x.s.a == 0) x0++;
      if (y.s.a == 0) y0++;
      if (y.s.b[3:0] <= 1) ylo++;
    end
    $display("T bad=%0d", bad);
    $display("T counts %0d %0d %0d", x0, y0, ylo);
  end
endmodule
"#,
    );
    assert_eq!(out[0], "T bad=0");
    // Each count expects 1000 of 4000 (weight 1 against 3); the bound is
    // about 5.5 standard deviations. A uniform draw lands near 2000.
    let counts: Vec<i64> = out[1]
        .trim_start_matches("T counts ")
        .split(' ')
        .map(|n| n.parse().unwrap())
        .collect();
    for n in counts {
        assert!((850..=1150).contains(&n), "weighted count {n} in {out:?}");
    }
}

/// §18.4/§18.5.8: unpacked-struct members — a `dist` on one, and a
/// `foreach` over an array member (which used to return 1 with the
/// constraint silently dropped).
#[test]
fn unpacked_struct_member_dist_and_foreach() {
    let out = tagged(
        r#"
typedef struct { rand int F; } u_t;
typedef struct { rand int A[4]; } ua_t;
class L; rand u_t u; constraint c { u.F dist { [7:7] :/ 1 }; } endclass
class M;
  rand ua_t u;
  constraint c { foreach (u.A[i]) u.A[i] inside { [i + 10 : i + 10] }; }
  function int at(int i); return u.A[i]; endfunction
endclass
class Bad;
  rand ua_t u;
  constraint c { foreach (u.A[i]) u.A[i] inside { [1:2] }; u.A[0] == 5; }
endclass
module top;
  initial begin
    L l = new; M m = new; Bad b = new; int r;
    r = l.randomize(); $display("T dist r=%0d F=%0d", r, l.u.F);
    r = m.randomize(); $display("T foreach r=%0d %0d %0d %0d %0d", r, m.at(0), m.at(1), m.at(2), m.at(3));
    r = b.randomize(); $display("T foreach unsat r=%0d", r);
  end
endmodule
"#,
    );
    assert_eq!(
        out,
        [
            "T dist r=1 F=7",
            "T foreach r=1 10 11 12 13",
            "T foreach unsat r=0",
        ]
    );
}

/// §18.5.5: `unique {m}` over a MULTI-dimensional fixed array covers every
/// element. Only 1-D arrays were handled: a 2-D draw kept duplicates, and a
/// pigeonhole-unsatisfiable one returned 1.
#[test]
fn unique_over_two_dimensional_array() {
    let out = tagged(
        r#"
class Sat; rand bit [3:0] m[4][4]; constraint c { unique { m }; } endclass
class Unsat; rand bit [3:0] m[6][6]; constraint c { unique { m }; } endclass
module top;
  initial begin
    Sat s = new; Unsat u = new; int r, dups;
    r = s.randomize();
    foreach (s.m[i, j]) foreach (s.m[k, l])
      if ((i * 4 + j) < (k * 4 + l) && s.m[i][j] == s.m[k][l]) dups++;
    $display("T sat r=%0d dups=%0d", r, dups);
    r = u.randomize();
    $display("T unsat r=%0d", r);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T sat r=1 dups=0", "T unsat r=0"]);
}
