//! §8.23 / §8.25.1 — an unpacked dimension sized by a class-scoped
//! constant (`a[2 * model_c::BEATS]`) elaborated to ZERO elements (#316,
//! reported by urbanij): `$size` read 0, `foreach` did nothing and
//! `randomize()` "succeeded" leaving the elements X. A class's constants
//! were not visible to the constant evaluator through the class scope, so
//! `model_c::BEATS` read 0; a bare `[cls::N]` (like `[N]`) also parses as an
//! associative "key type" and stayed associative. Expected values are the
//! reference simulator's.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .collect()
}

fn t_lines(src: &str) -> Vec<String> {
    messages(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// The issue's reproducer.
#[test]
fn class_localparam_sizes_rand_array() {
    let msgs = messages(SRC_ISSUE);
    for want in [
        "$size(by_class_param) = 146  (expect 146)",
        "by_class_param[0..2] = 01 02 03  (expect 01 02 03)",
        "PASS",
    ] {
        assert!(
            msgs.iter().any(|m| m == want),
            "missing {want:?} in {msgs:#?}"
        );
    }
    assert!(!msgs.iter().any(|m| m.contains("randomize() returned 0")));
}

/// `cls::P` directly and in expressions, localparam chains, `parameter`
/// vs `localparam`, a package class (`pkg::cls::N`), a parameterized class
/// (`cls#(4)::N`, `cls#()::N`); the dimension on a class property, a
/// struct member, a method local, a module variable, a packed range and a
/// multi-dimensional array, with `$size`/`$bits`/`foreach`/`randomize`.
#[test]
fn class_scoped_constants_in_dimensions() {
    assert_eq!(t_lines(SRC_DIMS), EXPECT_DIMS);
}

/// Specializations (positional and named), a nested class
/// (`outer_c::inner_c::I`), one class's constant defined through another's,
/// and a module localparam built from a class constant.
#[test]
fn class_scoped_constants_specialized_nested_chained() {
    assert_eq!(t_lines(SRC_SPEC), EXPECT_SPEC);
}

/// Locals of functions and class methods sized by a module parameter, a
/// package parameter, a class constant or the class's own localparam, and
/// unpacked struct members sized the same ways (module, `$unit` and
/// class-local typedefs).
#[test]
fn scope_sized_locals_and_struct_members() {
    assert_eq!(t_lines(SRC_LOCALS), EXPECT_LOCALS);
}

const SRC_ISSUE: &str = r#"
// a fixed-size unpacked dimension sized by a class-scoped localparam
// (cls::N) elaborates to ZERO elements. Reads return a 32-bit 0, foreach
// over it is a no-op, and randomize() "succeeds" leaving every element 0.
// A literal size works.
class model_c;
  localparam int ROWS = 145;
  localparam int BEATS = (ROWS + 1) / 2;  // 73
endclass

class seq_c;
  rand logic [7:0] by_class_param[2 * model_c::BEATS];  // expect 146 elements
  rand logic [7:0] by_literal[146];                     // reference
endclass

module top;
  initial begin
    seq_c s = new();
    $display("$size(by_class_param) = %0d  (expect 146)", $size(s.by_class_param));
    $display("$size(by_literal)     = %0d  (expect 146)", $size(s.by_literal));
    $display("$bits(by_class_param[0]) = %0d, value printed: %h  (expect 8, 2 hex digits)",
             $bits(s.by_class_param[0]), s.by_class_param[0]);
    
    if (!s.randomize() with {
      foreach (by_class_param[i]) by_class_param[i] == 8'(i + 1);
    })

    $display("randomize() returned 0");
    $display("by_class_param[0..2] = %h %h %h  (expect 01 02 03)",
             s.by_class_param[0], s.by_class_param[1], s.by_class_param[2]);

    if ($size(s.by_class_param) == 146 && s.by_class_param[2] == 8'h03) $display("PASS");
    else $display("FAIL");
    $finish;
  end
endmodule
"#;

const SRC_DIMS: &str = r#"
package pkg;
  class pc;
    localparam int N = 5;
    parameter int M = N + 1;
  endclass
endpackage

class model_c;
  localparam int ROWS = 145;
  localparam int BEATS = (ROWS + 1) / 2;
  parameter int P = 3;
  localparam int CH = BEATS - 70;   // 3
endclass

class prm #(int W = 2);
  localparam int N = W * 2;
endclass

class dims_c;
  rand logic [7:0] a1[model_c::BEATS];
  rand logic [7:0] a2[2 * model_c::BEATS];
  rand logic [7:0] a3[model_c::P];
  rand logic [7:0] a4[model_c::CH];
  rand logic [7:0] a5[pkg::pc::N];
  rand logic [7:0] a6[pkg::pc::M];
  rand logic [7:0] a7[prm#(4)::N];
  rand logic [7:0] a8[prm#()::N];
  rand logic [model_c::P-1:0] pk;
  rand logic [7:0] md[model_c::P][model_c::CH + 1];
  rand logic [7:0] rg[0:model_c::P-1];
  typedef struct { logic [7:0] m[model_c::P]; } st_t;
  st_t s;
  function int loc();
    logic [3:0] l[model_c::CH * 2];
    foreach (l[i]) l[i] = i;
    return $size(l) * 100 + l[5];
  endfunction
endclass

module top;
  logic [7:0] mv[model_c::P + 1];
  logic [model_c::CH:0] mp;
  int cnt;
  initial begin
    automatic dims_c d = new();
    $display("T| a1 %0d", $size(d.a1));
    $display("T| a2 %0d", $size(d.a2));
    $display("T| a3 %0d", $size(d.a3));
    $display("T| a4 %0d", $size(d.a4));
    $display("T| a5 %0d", $size(d.a5));
    $display("T| a6 %0d", $size(d.a6));
    $display("T| a7 %0d", $size(d.a7));
    $display("T| a8 %0d", $size(d.a8));
    $display("T| pk %0d", $bits(d.pk));
    $display("T| md %0d %0d", $size(d.md, 1), $size(d.md, 2));
    $display("T| rg %0d %0d", $size(d.rg), $right(d.rg));
    $display("T| st %0d %0d", $size(d.s.m), $bits(d.s));
    $display("T| loc %0d", d.loc());
    $display("T| mv %0d", $size(mv));
    $display("T| mp %0d", $bits(mp));
    $display("T| expr %0d %0d %0d", model_c::BEATS, 2 * model_c::BEATS + 1, pkg::pc::M);
    cnt = 0; foreach (d.a2[i]) cnt++;
    $display("T| foreach a2 %0d", cnt);
    cnt = 0; foreach (d.md[i, j]) cnt++;
    $display("T| foreach md %0d", cnt);
    if (!d.randomize() with { foreach (a2[i]) a2[i] == 8'(i + 1);
                              foreach (a4[i]) a4[i] == 8'(i + 10);
                              foreach (md[i, j]) md[i][j] == 8'(i * 16 + j);
                              foreach (a7[i]) a7[i] == 8'(i + 20); })
      $display("T| randomize failed");
    $display("T| rnd a2 %h %h %h last %h", d.a2[0], d.a2[1], d.a2[2], d.a2[145]);
    $display("T| rnd a4 %h %h", d.a4[0], d.a4[2]);
    $display("T| rnd md %h %h", d.md[0][0], d.md[2][3]);
    $display("T| rnd a7 %h %h", d.a7[0], d.a7[7]);
    $finish;
  end
endmodule
"#;

const EXPECT_DIMS: [&str; 22] = [
    "T| a1 73",
    "T| a2 146",
    "T| a3 3",
    "T| a4 3",
    "T| a5 5",
    "T| a6 6",
    "T| a7 8",
    "T| a8 4",
    "T| pk 3",
    "T| md 3 4",
    "T| rg 3 2",
    "T| st 3 24",
    "T| loc 605",
    "T| mv 4",
    "T| mp 4",
    "T| expr 73 147 6",
    "T| foreach a2 146",
    "T| foreach md 12",
    "T| rnd a2 01 02 03 last 92",
    "T| rnd a4 0a 0c",
    "T| rnd md 00 23",
    "T| rnd a7 14 1b",
];

const SRC_SPEC: &str = r#"
class prm #(int W = 2);
  localparam int N = W * 2;
  logic [7:0] arr[N];
endclass

class tp #(type T = byte);
  localparam int K = 3;
endclass

class base_c;
  localparam int B = 4;
endclass

class outer_c;
  class inner_c;
    localparam int I = 6;
  endclass
  localparam int O = inner_c::I + 1;
endclass

class chain_b;
  localparam int B2 = 9;
endclass
class chain_a;
  localparam int A = chain_b::B2 + 1;
endclass

class user_c;
  logic [7:0] u1[prm#(4)::N];
  logic [7:0] u2[prm#(.W(5))::N];
  logic [7:0] u3[tp#()::K];
  logic [7:0] u4[outer_c::inner_c::I];
  logic [7:0] u5[outer_c::O];
  logic [7:0] u6[chain_a::A];
  logic [base_c::B*2-1:0] u7;
endclass

module top;
  localparam int MP = chain_a::A * 2;
  logic [7:0] m1[MP];
  logic [7:0] m2[prm#(3)::N];
  initial begin
    automatic user_c u = new();
    automatic prm #(4) p4 = new();
    $display("T| u1 %0d", $size(u.u1));
    $display("T| u2 %0d", $size(u.u2));
    $display("T| u3 %0d", $size(u.u3));
    $display("T| u4 %0d", $size(u.u4));
    $display("T| u5 %0d", $size(u.u5));
    $display("T| u6 %0d", $size(u.u6));
    $display("T| u7 %0d", $bits(u.u7));
    $display("T| m1 %0d MP %0d", $size(m1), MP);
    $display("T| m2 %0d", $size(m2));
    $display("T| p4arr %0d", $size(p4.arr));
    $display("T| rt %0d %0d", prm#()::N, chain_a::A);
    $finish;
  end
endmodule
"#;

const EXPECT_SPEC: [&str; 11] = [
    "T| u1 8",
    "T| u2 10",
    "T| u3 3",
    "T| u4 6",
    "T| u5 7",
    "T| u6 10",
    "T| u7 8",
    "T| m1 20 MP 20",
    "T| m2 6",
    "T| p4arr 8",
    "T| rt 4 10",
];

const SRC_LOCALS: &str = r#"
package pk; localparam int PN = 6; localparam int SN = 3; endpackage
class model_c;
  localparam int CH = 3;
  localparam int P = 3;
endclass
typedef struct { logic [7:0] m[model_c::P]; } gst_t;
class lit_c;
  localparam int Q = 3;
  typedef struct { logic [7:0] m[model_c::P]; } stc_t;
  typedef struct { logic [7:0] m[model_c::P + 0]; } ste_t;
  typedef struct { logic [7:0] m[Q]; } stq_t;
  stc_t sc; ste_t se; stq_t sq; gst_t sg;
  function int locc();
    logic [3:0] l[model_c::CH * 2];
    foreach (l[i]) l[i] = i;
    return $size(l) * 100 + l[5];
  endfunction
  function int locq();
    logic [3:0] l[Q];
    foreach (l[i]) l[i] = i + 1;
    return $size(l) * 100 + l[2];
  endfunction
endclass
module top;
  localparam int LP = 6;
  localparam int SP = 3;
  typedef struct { logic [7:0] m[SP]; } s1_t;
  typedef struct { logic [7:0] m[pk::SN]; } s2_t;
  typedef struct { logic [7:0] m[model_c::P]; } s3_t;
  s1_t a; s2_t b; s3_t c;
  function automatic int f1();
    logic [3:0] l[LP];
    foreach (l[i]) l[i] = i;
    return $size(l) * 100 + l[5];
  endfunction
  function automatic int f2();
    logic [3:0] l[pk::PN];
    foreach (l[i]) l[i] = i;
    return $size(l) * 100 + l[5];
  endfunction
  function automatic int f3();
    logic [3:0] l[model_c::CH * 2];
    foreach (l[i]) l[i] = i;
    return $size(l) * 100 + l[5];
  endfunction
  function automatic int f4();
    logic [3:0] l[model_c::CH];
    return $size(l);
  endfunction
  initial begin
    automatic lit_c d = new();
    logic [3:0] l6[model_c::CH * 2];
    int aa[pk::PN];
    d.sc.m[2] = 8'h11;
    $display("T| loc %0d %0d %0d %0d %0d %0d %0d %0d", f1(), f2(), f3(), f4(), $size(l6), d.locc(), d.locq(), $size(aa));
    $display("T| cst %0d %0d %0d %0d %h", $size(d.sc.m), $size(d.se.m), $size(d.sq.m), $size(d.sg.m), d.sc.m[2]);
    $display("T| mst %0d %0d %0d | %0d %0d", $size(a.m), $size(b.m), $size(c.m), $bits(b), $bits(c));
    $finish;
  end
endmodule
"#;

const EXPECT_LOCALS: [&str; 3] = [
    "T| loc 605 605 605 3 6 605 303 6",
    "T| cst 3 3 3 3 11",
    "T| mst 3 3 3 | 24 24",
];

/// Sibling shapes that size through the same constant evaluation: packed
/// widths, a module parameter default and a port width, a generate-for
/// bound and a generate-if condition, typedefs (packed, unpacked, packed
/// struct), `$unit`/package/package-class constants in dimensions, an
/// inherited constant (`der_c::B`), a specialization in a packed width, an
/// enum base, a replication count, a part-select, class properties, a
/// queue bound and a `size()` constraint.
#[test]
fn class_scoped_constant_sibling_shapes() {
    // Time-0 order across instances is not specified (§4.7): compare sorted.
    let mut got = t_lines(SRC_SIBLINGS);
    got.sort();
    let mut want = EXPECT_SIBLINGS.to_vec();
    want.sort();
    assert_eq!(got, want);
}

/// More siblings: a class constant built from a package constant, an
/// associative array keyed by a class typedef (stays associative), an
/// interface parameter default, a size cast `cls::N'(v)`, a derived class
/// sized by its base's constant, and a constant inherited two levels up.
#[test]
fn class_scoped_constant_sibling_shapes_2() {
    assert_eq!(t_lines(SRC_SIBLINGS_2), EXPECT_SIBLINGS_2);
}

/// Run-time reads of a specialization's or a nested class's constant.
#[test]
fn class_scoped_constant_runtime_reads() {
    assert_eq!(t_lines(SRC_RUNTIME), EXPECT_RUNTIME);
}

/// `$bits` of a 2-D class property and of an unpacked struct whose member
/// is sized by a bare module parameter.
#[test]
fn bits_of_unpacked_aggregates() {
    assert_eq!(t_lines(SRC_BITS), EXPECT_BITS);
}

const SRC_SIBLINGS: &str = r#"
package pk;
  localparam int PN = 5;
  class pc;
    localparam int N = 6;
  endclass
endpackage
localparam int UN = 4;
class model_c;
  localparam int N = 3;
  localparam int W = 2 * N;   // 6
endclass
class base_c;
  localparam int B = 7;
endclass
class der_c extends base_c;
endclass
class prm #(int W = 2);
  localparam int N = W * 2;
endclass
class prop_c;
  logic [7:0] q1[$:model_c::N];
  logic [7:0] u1[UN];
  logic [7:0] u2[pk::PN];
  logic [pk::pc::N-1:0] pw;
  rand logic [7:0] dyn[];
  constraint c_sz { dyn.size() == model_c::N; }
endclass
module sub #(parameter int W = 1) (input logic [W-1:0] a);
  initial $display("T| port_w %0d %0d", W, $bits(a));
endmodule
module subp #(parameter int P = model_c::W) ();
  initial $display("T| param_default %0d", P);
endmodule
typedef logic [7:0] arr_t[model_c::N];
typedef logic [model_c::W-1:0] w_t;
typedef struct packed { logic [model_c::N-1:0] f; logic g; } sp_t;
module top;
  localparam int LX = model_c::W * 2;
  logic [model_c::W-1:0] pv;
  logic [LX-1:0] lxv;
  arr_t av;
  w_t wv;
  sp_t spv;
  logic [7:0] mu[UN];
  logic [7:0] mp[pk::PN];
  logic [7:0] mpc[pk::pc::N];
  logic [7:0] md[der_c::B];
  logic [prm#(4)::N-1:0] spec_pk;
  enum logic [model_c::N-1:0] {E0, E1} ev;
  logic [model_c::N-1:0] cv;
  sub #(.W(model_c::W)) u_sub (.a(pv));
  subp u_subp ();
  genvar gi;
  int gen_cnt = 0;
  for (gi = 0; gi < model_c::N; gi++) begin : g
    initial gen_cnt++;
  end
  if (model_c::N > 2) begin : gif
    initial $display("T| gen_if taken");
  end
  initial begin
    automatic prop_c p = new();
    #1;
    $display("T| packed %0d %0d", $bits(pv), $bits(lxv));
    $display("T| gen_for %0d", gen_cnt);
    $display("T| typedef_unpacked %0d", $size(av));
    $display("T| typedef_packed %0d", $bits(wv));
    $display("T| struct_packed %0d", $bits(spv));
    $display("T| unit_dim %0d pkg_dim %0d pkgcls_dim %0d", $size(mu), $size(mp), $size(mpc));
    $display("T| inherited %0d", $size(md));
    $display("T| spec_packed %0d", $bits(spec_pk));
    $display("T| enum %0d", $bits(ev));
    $display("T| repl %b", {model_c::N{1'b1}});
    cv = 9'h1ff;
    $display("T| psel %b", cv[model_c::N-1:1]);
    $display("T| prop unit %0d pkg %0d pw %0d", $size(p.u1), $size(p.u2), $bits(p.pw));
    repeat (5) p.q1.push_back(8'h1);
    $display("T| queue_bound %0d", p.q1.size());
    void'(p.randomize());
    $display("T| rand_size %0d", p.dyn.size());
    $display("T| inh_rt %0d", der_c::B);
    $finish;
  end
endmodule
"#;

const EXPECT_SIBLINGS: [&str; 18] = [
    "T| port_w 6 6",
    "T| param_default 6",
    "T| gen_if taken",
    "T| packed 6 12",
    "T| gen_for 3",
    "T| typedef_unpacked 3",
    "T| typedef_packed 6",
    "T| struct_packed 4",
    "T| unit_dim 4 pkg_dim 5 pkgcls_dim 6",
    "T| inherited 7",
    "T| spec_packed 8",
    "T| enum 3",
    "T| repl 111",
    "T| psel 11",
    "T| prop unit 4 pkg 5 pw 6",
    "T| queue_bound 4",
    "T| rand_size 3",
    "T| inh_rt 7",
];

const SRC_SIBLINGS_2: &str = r#"
package pk;
  localparam int PN = 5;
endpackage
class model_c;
  localparam int N = 3;
  localparam int K = pk::PN + 1;   // 6
  typedef byte key_t;
endclass
class base_c;
  localparam int B = 7;
endclass
class mid_c extends base_c;
endclass
class der_c extends mid_c;
  logic [7:0] x[B];
  logic [7:0] y[B - 2];
  function int f(); return $size(x) * 10 + $size(y); endfunction
endclass
interface ifc #(parameter int W = model_c::N) ();
  logic [W-1:0] s;
endinterface
module top;
  logic [7:0] kd[model_c::K];
  int aa[model_c::key_t];
  ifc i0 ();
  logic [7:0] cst;
  logic [7:0] chn[der_c::B];
  initial begin
    automatic der_c d = new();
    aa[3] = 1; aa[9] = 2;
    cst = model_c::N'(5'b11111) ;
    $display("T| cls_from_pkg %0d", $size(kd));
    $display("T| assoc_cls_typedef %0d", aa.num());
    $display("T| iface_param %0d", $bits(i0.s));
    $display("T| size_cast %b", cst);
    $display("T| derived_dims %0d", d.f());
    $display("T| chain_inh %0d", $size(chn));
    $finish;
  end
endmodule
"#;

const EXPECT_SIBLINGS_2: [&str; 6] = [
    "T| cls_from_pkg 6",
    "T| assoc_cls_typedef 2",
    "T| iface_param 3",
    "T| size_cast 00000111",
    "T| derived_dims 75",
    "T| chain_inh 7",
];

const SRC_RUNTIME: &str = r#"
class prm #(int W = 2);
  localparam int N = W * 2;
endclass
class outer_c;
  class inner_c;
    localparam int I = 6;
  endclass
endclass
module top;
  initial $display("T| rt %0d %0d %0d", prm#(4)::N, prm#(.W(5))::N, outer_c::inner_c::I);
endmodule
"#;

const EXPECT_RUNTIME: [&str; 1] = ["T| rt 8 10 6"];

const SRC_BITS: &str = r#"
class c2d;
  logic [7:0] md[3][4];
endclass
module top;
  localparam int SP = 3;
  typedef struct { logic [7:0] m[SP]; } s1_t;
  s1_t a;
  initial begin
    automatic c2d d = new();
    $display("T| bits %0d %0d", $bits(d.md), $bits(a));
  end
endmodule
"#;

const EXPECT_BITS: [&str; 1] = ["T| bits 96 24"];

/// §8.25/§8.23 siblings of the run-time reads: a header parameter, the
/// default specialization, an argument expression, a typedef of a
/// specialization, reads inside a module function and a class method, a
/// default naming an earlier parameter, a nested class constant built from
/// another, and a static initialized from a parameter.
#[test]
fn class_scoped_constant_runtime_siblings() {
    assert_eq!(
        t_lines(SRC_RUNTIME_SIBLINGS),
        [
            "T| rt 8 10 6",
            "T| hdr 3 4",
            "T| expr 6 12",
            "T| fn 10 14",
            "T| two 7 10 10",
            "T| nest 12 3",
            "T| sv 104 102",
        ]
    );
}

/// §20.6.2 siblings: 2-D and 3-D class properties read from outside, bare
/// and through `this` inside a method, unpacked structs whose members are
/// sized by module parameters (2-D, nested, `[SQ:0]`), the types
/// themselves, and a module 2-D array.
#[test]
fn bits_of_unpacked_aggregates_siblings() {
    assert_eq!(
        t_lines(SRC_BITS_SIBLINGS),
        [
            "T| bits 96 768 20",
            "T| in 96 768",
            "T| st 24 80 25 24 24",
            "T| ty 24 80 25",
            "T| mod 48 24",
        ]
    );
}

const SRC_RUNTIME_SIBLINGS: &str = r#"
class prm #(int W = 2);
  localparam int N = W * 2;
  static int sv = W + 100;
endclass
class prm2 #(int A = 1, int B = A + 1);
  localparam int S = A + B;
endclass
class outer_c;
  localparam int O = 3;
  class inner_c;
    localparam int I = 6;
    localparam int J = I * 2;
  endclass
endclass
typedef prm#(6) p6_t;
class user_c;
  function int f(); return prm#(4)::N + outer_c::inner_c::I; endfunction
endclass
module top;
  function automatic int mf(); return prm#(5)::N; endfunction
  initial begin
    automatic user_c u = new();
    $display("T| rt %0d %0d %0d", prm#(4)::N, prm#(.W(5))::N, outer_c::inner_c::I);
    $display("T| hdr %0d %0d", prm#(3)::W, prm#()::N);
    $display("T| expr %0d %0d", prm#(2+1)::N, p6_t::N);
    $display("T| fn %0d %0d", mf(), u.f());
    $display("T| two %0d %0d %0d", prm2#(3)::S, prm2#(3,7)::S, prm2#(.B(9))::S);
    $display("T| nest %0d %0d", outer_c::inner_c::J, outer_c::O);
    $display("T| sv %0d %0d", prm#(4)::sv, prm#()::sv);
  end
endmodule
"#;

const SRC_BITS_SIBLINGS: &str = r#"
class c2d;
  logic [7:0] md[3][4];
  int m3[2][3][4];
  bit [3:0] one[5];
  function int inside_bits(); return $bits(md); endfunction
  function int inside_this(); return $bits(this.m3); endfunction
endclass
module top;
  localparam int SP = 3;
  localparam int SQ = 2;
  typedef struct { logic [7:0] m[SP]; } s1_t;
  typedef struct { logic [7:0] m[SP][SQ]; int k; } s2_t;
  typedef struct { s1_t in; bit b; } s3_t;
  typedef struct { logic [7:0] m[3]; } s4_t;
  typedef struct { logic [7:0] m[SQ:0]; } s5_t;
  s1_t a; s2_t b; s3_t c; s4_t d4; s5_t e5;
  logic [7:0] mod2[SP][SQ];
  initial begin
    automatic c2d d = new();
    $display("T| bits %0d %0d %0d", $bits(d.md), $bits(d.m3), $bits(d.one));
    $display("T| in %0d %0d", d.inside_bits(), d.inside_this());
    $display("T| st %0d %0d %0d %0d %0d", $bits(a), $bits(b), $bits(c), $bits(d4), $bits(e5));
    $display("T| ty %0d %0d %0d", $bits(s1_t), $bits(s2_t), $bits(s3_t));
    $display("T| mod %0d %0d", $bits(mod2), $bits(a.m));
  end
endmodule
"#;
