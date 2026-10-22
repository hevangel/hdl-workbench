//! §10.9/§7.4.2: assignment patterns onto PACKED targets — the ivtest
//! `sv_ap_parray1..4`, `sv_ap_struct1/3`, `sv_assign_pattern_{concat,expand}`
//! cluster, all confirmed against a reference simulator.
//!
//! The generic evaluator concatenated pattern items at their OWN widths; a
//! packed target instead converts each item to the ELEMENT type — a 1-bit
//! item occupies a full element, a real rounds (§6.12.2), a string keeps its
//! low bits — and a nested `'{...}` descends one packed dimension. Fixed at
//! four placements: procedural assign, declaration/localparam (elaborate's
//! `pack_packed_vector_pattern`, which also refused SINGLE-dim vectors),
//! continuous assign (whole signal and array element), and packed-STRUCT
//! members (nested vector members + real conversion).
//!
//! Also here:
//! - `'{expr}` items evaluate in the ELEMENT-TYPE context (§10.9.2):
//!   `int d[]; d = '{1'b1 + 1'b1}` is 2, not a 1-bit wrap.
//! - `'{3{y}}` (pattern multiplier, three elements) vs `'{{3{y}}}` (ONE
//!   concat item) parsed to the SAME AST node; the parser now wraps the
//!   explicit-brace form in Paren so the two are distinguishable.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Procedural, localparam, and nested patterns on packed vectors.
#[test]
fn packed_vector_patterns_convert_each_item() {
    let src = r#"
module top;
  bit [3:0][3:0] x;
  bit [1:0][3:0][3:0] y;
  localparam bit [2:0] LP = '{1'b1, 2.0, 2 + 1};
  int xv, yv, lpv;
  initial begin
    x = '{1'b1, 1 + 1, 3.0, "TEST"};
    y = '{'{1'b1, 1 + 1, 3.0, "TEST"},
          '{5, 6, '{1'b0, 1 * 1, 3, 1.0}, 8}};
    xv = x; yv = y; lpv = LP;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "xv"), 0x1234, "each item fits one 4-bit element");
    assert_eq!(
        u(&sim, "yv"),
        0x1234_5678,
        "a nested pattern in a 4-bit element packs per bit: 0111"
    );
    assert_eq!(
        u(&sim, "lpv"),
        0b101,
        "single-dim vector: one item per BIT, real rounds"
    );
}

/// Continuous assigns — whole signal and an element of an unpacked array.
#[test]
fn packed_patterns_in_continuous_assigns() {
    let src = r#"
`timescale 1ns/1ns
module top;
  wire [3:0][3:0] cw;
  wire [3:0][3:0] ca[2];
  typedef struct packed { logic [31:0] x; logic [15:0] y; logic [7:0] z; } T;
  T cs;
  assign cw = '{1'b1, 32'h2, 3.0, "TEST"};
  assign ca[0] = '{1'b1, 32'h2, 3.0, "TEST"};
  assign cs = '{1'b1, 2.0, 2 + 1};
  int cwv, cav, cs_x, cs_y, cs_z;
  initial begin
    #1;
    cwv = cw; cav = ca[0];
    cs_x = cs.x; cs_y = cs.y; cs_z = cs.z;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "cwv"), 0x1234, "CA to a packed vector");
    assert_eq!(
        u(&sim, "cav"),
        0x1234,
        "CA to an array ELEMENT of packed vectors"
    );
    assert_eq!(u(&sim, "cs_x"), 1, "struct member from 1'b1");
    assert_eq!(u(&sim, "cs_y"), 2, "real 2.0 CONVERTS to shortint");
    assert_eq!(u(&sim, "cs_z"), 3, "2 + 1");
}

/// Packed structs: converting items and a nested packed-VECTOR member.
#[test]
fn packed_struct_patterns_convert_members() {
    let src = r#"
module top;
  typedef struct packed { int x; shortint y; byte z; } T;
  T s;
  struct packed { T x; bit [2:0][3:0] y; } n;
  longint sv, nv;
  initial begin
    s = '{1'b1, 2.0, 2 + 1};
    n = '{'{1'b1, 2.0, 2 + 1}, '{4, 5, 6}};
    sv = s; nv = n;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "sv"),
        0x00000001_0002_03,
        "int/shortint/byte each converted"
    );
    assert_eq!(
        u(&sim, "nv"),
        0x00000001_0002_03_456,
        "nested struct + nested vector member"
    );
}

/// §10.9.2 element-type context, and multiplier-vs-concat disambiguation.
#[test]
fn element_context_and_multiplier_vs_concat() {
    let src = r#"
module top;
  int d[$];
  shortint x;
  bit [7:0] y;
  bit [23:0] q[$];
  int e_sum, e_signed, m_size, c_size, m0, c0;
  initial begin
    x = -11; y = 8'h0A;
    d = '{1'b1 + 1'b1};        // element context: 32-bit add
    e_sum = d[0];
    d = '{x};                  // sign-extends into the int element
    e_signed = d[0];
    d = '{3{5}};               // multiplier: THREE elements
    m_size = d.size(); m0 = d[0];
    q = '{{3{y}}};             // concat item: ONE 24-bit element
    c_size = q.size(); c0 = q[0];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "e_sum"),
        2,
        "1'b1 + 1'b1 in a 32-bit element context"
    );
    assert_eq!(
        u(&sim, "e_signed") as u32 as i32,
        -11,
        "signed item sign-extends"
    );
    assert_eq!(
        u(&sim, "m_size"),
        3,
        "the pattern multiplier yields three elements"
    );
    assert_eq!(u(&sim, "m0"), 5);
    assert_eq!(
        u(&sim, "c_size"),
        1,
        "the explicit-brace concat is a single item"
    );
    assert_eq!(u(&sim, "c0"), 0x0A0A0A, "whose value is the 24-bit concat");
}

/// §7.4.1: writes THROUGH a packed-struct member that is itself a packed
/// multi-D vector — `foo.a[5] = v`, `foo.a[7][1:0] = v`. The single-element
/// form silently created a phantom `foo.a[5]` name-keyed entry and the struct
/// never changed, while the RANGE form worked — which is exactly how it hid
/// (ivtest `struct7`/`struct10`). Reference-validated bit-for-bit.
#[test]
fn writes_through_packed_struct_vector_members() {
    let src = r#"
module top;
  struct packed {
    bit [7:0][7:0] a;
    bit [15:0] b;
  } foo;
  logic [79:0] snap_range, snap_elem, snap_whole, snap_bits, snap_b;
  initial begin
    foo = '0;
    foo.a[2:1] = 16'h1234;  snap_range = foo;
    foo.a[5] = 8'h42;       snap_elem  = foo;
    foo.a[7] = '1;          snap_whole = foo;
    foo.a[7][1:0] = '0;     snap_bits  = foo;
    foo.b = '1; foo.b[1:0] = 2'b00; snap_b = foo;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    let h = |n: &str| -> String {
        sim.get_signal(&format!("top.{}", n))
            .or_else(|| sim.get_signal(n))
            .unwrap_or_else(|| panic!("missing {n}"))
            .to_hex_string()
    };
    assert_eq!(
        h("snap_range"),
        "00000000001234000000",
        "element range (worked before)"
    );
    assert_eq!(
        h("snap_elem"),
        "00004200001234000000",
        "single element write lands in the struct"
    );
    assert_eq!(h("snap_whole"), "ff004200001234000000", "whole element");
    assert_eq!(
        h("snap_bits"),
        "fc004200001234000000",
        "bit range within an element"
    );
    assert_eq!(
        h("snap_b"),
        "fc00420000123400fffc",
        "plain member bit range"
    );
}
