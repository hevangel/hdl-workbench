//! ivtest FAIL_OUT mining, round 2 — two root causes behind
//! task_nonansi_struct1/2 and task_nonansi_parray1/2, both reference-validated.
//!
//! 1. **A task/function FORMAL never received its declared type's structural
//!    metadata.** Formals are bound as a flat value in the local frame; unlike
//!    a body-local declaration they registered no packed-struct layout and no
//!    packed element width, so `formal.field` found no layout and read 0 while
//!    `$bits(formal)` and the whole-value copy were correct. This is NOT a
//!    non-ANSI parsing problem — the ANSI form `task t(input T p)` failed
//!    identically; the non-ANSI tests just happened to expose it.
//!
//! 2. **A typedef whose base is a packed array lost its dimensions.**
//!    `typedef logic [7:0] T1; typedef T1 [3:0] T2; T2 v;` recorded no element
//!    width or dims, so `v[i]` degraded to a one-BIT select on both read and
//!    write (`v[0]` happened to read right, which disguised it). The chain has
//!    to be walked ACCUMULATING dims: `resolve_typedef_chain` walks to the end
//!    and discards the intermediate `[3:0]`.
//!
//!    The accumulating walk must also stop at a non-vector base: for
//!    `parcel_t [0:0][1:0]` (struct elements) the element width lives in the
//!    struct, not in a dimension — reporting dims there made `$bits(x[0])`
//!    read 2 instead of 134. That regression was caught by the existing suite.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Struct-typed formals: member selects must resolve, in every declaration
/// style, and a body-local must keep working.
#[test]
fn struct_typed_formals_resolve_members() {
    let src = r#"
module tb;
  typedef struct packed { reg [31:0] x; reg [7:0] y; } T;
  int lx, ly, ax, ay, nx, ny, bits_ansi;
  task tloc;                        // body-local (always worked)
    T v;
    begin v.x = 10; v.y = 20; lx = v.x; ly = v.y; end
  endtask
  task tansi(input T p);            // ANSI formal
    begin ax = p.x; ay = p.y; bits_ansi = $bits(p); end
  endtask
  task tnonansi;                    // non-ANSI: direction and type split
    input q;
    T q;
    begin nx = q.x; ny = q.y; end
  endtask
  initial begin
    static T val;
    val.x = 10; val.y = 20;
    tloc(); tansi(val); tnonansi(val);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "lx"), u(&sim, "ly")),
        (10, 20),
        "body-local struct"
    );
    assert_eq!(
        (u(&sim, "ax"), u(&sim, "ay")),
        (10, 20),
        "ANSI struct formal"
    );
    assert_eq!(
        (u(&sim, "nx"), u(&sim, "ny")),
        (10, 20),
        "non-ANSI struct formal"
    );
    assert_eq!(
        u(&sim, "bits_ansi"),
        40,
        "whole-value width was always right"
    );
}

/// A typedef of a packed array: element selects, in module scope, as an
/// automatic local, as a static local, and as a task formal.
#[test]
fn typedef_of_packed_array_element_selects() {
    let src = r#"
module tb;
  typedef logic [7:0] T1;
  typedef T1 [3:0] T2;
  T2 modscope;
  int m0, m1, a0, a1, s0, s1, f0, f1, bt;
  task tf;
    input p;
    T2 p;
    begin f0 = p[0]; f1 = p[1]; end
  endtask
  initial begin
    T2 autoloc;
    static T2 statloc;
    modscope[0] = 8'h1; modscope[1] = 8'h2;
    autoloc[0]  = 8'h1; autoloc[1]  = 8'h2;
    statloc[0]  = 8'h1; statloc[1]  = 8'h2;
    m0 = modscope[0]; m1 = modscope[1];
    a0 = autoloc[0];  a1 = autoloc[1];
    s0 = statloc[0];  s1 = statloc[1];
    bt = $bits(T2);
    tf(statloc);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!((u(&sim, "m0"), u(&sim, "m1")), (1, 2), "module scope");
    assert_eq!((u(&sim, "a0"), u(&sim, "a1")), (1, 2), "automatic local");
    assert_eq!((u(&sim, "s0"), u(&sim, "s1")), (1, 2), "static local");
    assert_eq!((u(&sim, "f0"), u(&sim, "f1")), (1, 2), "task formal");
    assert_eq!(u(&sim, "bt"), 32, "4 x 8 bits");
}

/// The accumulating walk must NOT claim dims for a chain that bottoms out in a
/// struct — the element width is in the struct there, not in a dimension.
#[test]
fn packed_array_of_struct_typedef_keeps_element_width() {
    let src = r#"
module tb;
  typedef struct packed { logic [7:0] a; logic [7:0] b; } pair_t;  // 16
  pair_t            single;
  pair_t [1:0]      arr;      // 32
  pair_t [1:0][1:0] nested;   // 64
  int w_single, w_arr, w_arr_sel, w_nested, w_nested_sel, w_nested_sel2;
  initial begin
    w_single      = $bits(single);
    w_arr         = $bits(arr);
    w_arr_sel     = $bits(arr[0]);
    w_nested      = $bits(nested);
    w_nested_sel  = $bits(nested[0]);
    w_nested_sel2 = $bits(nested[0][0]);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w_single"), 16);
    assert_eq!(u(&sim, "w_arr"), 32);
    assert_eq!(u(&sim, "w_arr_sel"), 16, "element is the struct, not a bit");
    assert_eq!(u(&sim, "w_nested"), 64);
    assert_eq!(
        u(&sim, "w_nested_sel"),
        32,
        "outer select keeps the sub-array"
    );
    assert_eq!(u(&sim, "w_nested_sel2"), 16);
}
