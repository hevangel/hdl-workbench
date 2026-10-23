//! §7.4.1 / §8.25 — indexing a multi-dimensional packed class member, and
//! binding a `type` parameter left at its declared default.
//!
//! Two independent gaps, both surfaced by the same class-parameter matrix:
//!
//! 1. A class PROPERTY (or value PARAMETER) of multi-dimensional packed type
//!    carried no per-element width at run time — the module-scope equivalent
//!    comes from `packed_signal_elem_widths`, which only covers signals. An
//!    index therefore fell through to a plain BIT select: `arr[1]` on
//!    `16'hAABB` read `1` instead of `8'hAA`, and an indexed WRITE was dropped
//!    on the floor. The same declaration at module scope always worked.
//!
//! 2. A `type` parameter the specialization omitted was never bound to its
//!    declared default, so a property typed by it (`CLASS_T o;`) was never
//!    constructed — reading back x. It only worked when the default was
//!    spelled out explicitly in the `#(...)` list.

use xezim::simulate;

/// Element selects on a multi-dimensional packed class property, against the
/// identical declaration at module scope. Includes a write, which previously
/// vanished silently rather than merely landing in the wrong place.
const PACKED_ELEM: &str = r#"
module tb;
  class holder;
    bit [1:0][7:0] arr;
    bit [3:0][7:0] big;
  endclass
  bit [1:0][7:0] m_arr;
  int l1, l0, p1, p0, b3, b2, b1, b0, after_write;
  initial begin
    holder h = new();
    h.arr = 16'hAABB;
    h.big = 32'h11223344;
    m_arr = 16'hAABB;
    l1 = m_arr[1];  l0 = m_arr[0];
    p1 = h.arr[1];  p0 = h.arr[0];
    b3 = h.big[3];  b2 = h.big[2];  b1 = h.big[1];  b0 = h.big[0];
    h.arr[1] = 8'h77;
    after_write = h.arr;
  end
endmodule
"#;

/// The element width may itself come from a class parameter, and the outer
/// range may ascend (`[0:1]`), which puts index 0 at the MS end (§7.4.1).
const PACKED_ELEM_PARAM: &str = r#"
module tb;
  class holder #(parameter int EW = 8);
    bit [1:0][EW-1:0] arr;
  endclass
  class asc;
    bit [0:1][7:0] up;
  endclass
  int w1, w0, n1, n0, u0, u1;
  initial begin
    holder #(8)  h8;
    holder #(4)  h4;
    asc          a;
    h8 = new(); h4 = new(); a = new();
    h8.arr = 16'hAABB;
    h4.arr = 8'hCD;
    a.up   = 16'hAABB;
    w1 = h8.arr[1]; w0 = h8.arr[0];
    n1 = h4.arr[1]; n0 = h4.arr[0];
    u0 = a.up[0];   u1 = a.up[1];
  end
endmodule
"#;

/// A value PARAMETER of packed-array type is bound as an instance property, so
/// indexing it must select an element too.
const PACKED_PARAM: &str = r#"
module tb;
  class box #(parameter bit [1:0][7:0] V = '{8'h11, 8'h22});
    bit [1:0][7:0] mirror;
    function new(); mirror = V; endfunction
  endclass
  int d1, d0, o1, o0, m1, m0;
  initial begin
    box                          bd;
    box #('{8'hAA, 8'hBB})       bo;
    bd = new(); bo = new();
    d1 = bd.V[1]; d0 = bd.V[0];
    o1 = bo.V[1]; o0 = bo.V[0];
    m1 = bo.mirror[1]; m0 = bo.mirror[0];
  end
endmodule
"#;

/// An index means different things depending on whether the property also has
/// UNPACKED dimensions (§7.4.2): `tbl[2]` on `bit [1:0][7:0] tbl [4]` selects
/// an unpacked entry, not a packed element. The element-select path must not
/// hijack collections.
const PACKED_VS_UNPACKED: &str = r#"
module tb;
  class mixed;
    bit [1:0][7:0] arr;
    bit [1:0][7:0] tbl [4];
    bit [7:0]      mem [4];
    int            q [$];
  endclass
  int a1, a0, t2, t3, m1, q0;
  initial begin
    mixed m = new();
    m.arr    = 16'hAABB;
    m.tbl[2] = 16'h1234;
    m.tbl[3] = 16'h5678;
    m.mem[1] = 8'h9A;
    m.q.push_back(42);
    a1 = m.arr[1]; a0 = m.arr[0];
    t2 = m.tbl[2]; t3 = m.tbl[3];
    m1 = m.mem[1]; q0 = m.q[0];
  end
endmodule
"#;

/// A `type` parameter omitted from the specialization takes its declared
/// default, including when a LATER parameter is overridden explicitly.
const TYPE_PARAM_DEFAULT: &str = r#"
module tb;
  class payload;
    int id = 32'hC0FEBABE;
  endclass
  class other;
    int id = 32'h5A5A5A5A;
  endclass
  class box #(parameter type T = int, parameter type C = payload);
    C o;
    function new(); o = new(); endfunction
  endclass
  int all_def, t_given, both_given, c_over;
  initial begin
    box                                b1;
    box #(.T(int))                     b2;
    box #(.T(int), .C(payload))        b3;
    box #(.T(int), .C(other))          b4;
    b1 = new(); b2 = new(); b3 = new(); b4 = new();
    all_def    = b1.o.id;
    t_given    = b2.o.id;
    both_given = b3.o.id;
    c_over     = b4.o.id;
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn packed_element_select_on_class_property() {
    let sim = simulate(PACKED_ELEM, 100).expect("simulate failed");
    assert_eq!(u(&sim, "l1"), 0xAA, "module scope, element 1 (reference)");
    assert_eq!(u(&sim, "l0"), 0xBB, "module scope, element 0 (reference)");
    assert_eq!(
        u(&sim, "p1"),
        0xAA,
        "class property must element-select, not bit-select"
    );
    assert_eq!(u(&sim, "p0"), 0xBB, "class property element 0");
    assert_eq!(u(&sim, "b3"), 0x11, "4-element packed array, element 3");
    assert_eq!(u(&sim, "b2"), 0x22, "4-element packed array, element 2");
    assert_eq!(u(&sim, "b1"), 0x33, "4-element packed array, element 1");
    assert_eq!(u(&sim, "b0"), 0x44, "4-element packed array, element 0");
}

/// The indexed write used to be dropped entirely, leaving the property
/// unchanged — a silent data-loss bug, not just a wrong read.
#[test]
fn packed_element_write_on_class_property_lands() {
    let sim = simulate(PACKED_ELEM, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "after_write"),
        0x77BB,
        "arr[1] = 8'h77 must splice, not vanish"
    );
}

#[test]
fn packed_element_width_from_parameter_and_ascending_range() {
    let sim = simulate(PACKED_ELEM_PARAM, 100).expect("simulate failed");
    assert_eq!(u(&sim, "w1"), 0xAA, "8-bit elements from EW=8");
    assert_eq!(u(&sim, "w0"), 0xBB, "8-bit elements from EW=8");
    assert_eq!(u(&sim, "n1"), 0xC, "4-bit elements from EW=4");
    assert_eq!(u(&sim, "n0"), 0xD, "4-bit elements from EW=4");
    assert_eq!(
        u(&sim, "u0"),
        0xAA,
        "ascending [0:1] puts index 0 at the MS end"
    );
    assert_eq!(
        u(&sim, "u1"),
        0xBB,
        "ascending [0:1] puts index 1 at the LS end"
    );
}

#[test]
fn packed_element_select_on_class_value_parameter() {
    let sim = simulate(PACKED_PARAM, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "d1"),
        0x11,
        "defaulted packed-array parameter, element 1"
    );
    assert_eq!(
        u(&sim, "d0"),
        0x22,
        "defaulted packed-array parameter, element 0"
    );
    assert_eq!(
        u(&sim, "o1"),
        0xAA,
        "overridden packed-array parameter, element 1"
    );
    assert_eq!(
        u(&sim, "o0"),
        0xBB,
        "overridden packed-array parameter, element 0"
    );
    assert_eq!(
        u(&sim, "m1"),
        0xAA,
        "parameter copied into a property keeps its layout"
    );
    assert_eq!(
        u(&sim, "m0"),
        0xBB,
        "parameter copied into a property keeps its layout"
    );
}

/// Guards against the element-select path swallowing collection indexing.
#[test]
fn unpacked_dimensions_still_index_the_collection() {
    let sim = simulate(PACKED_VS_UNPACKED, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "a1"),
        0xAA,
        "packed-only property still element-selects"
    );
    assert_eq!(
        u(&sim, "a0"),
        0xBB,
        "packed-only property still element-selects"
    );
    assert_eq!(
        u(&sim, "t2"),
        0x1234,
        "packed+unpacked indexes the UNPACKED dimension"
    );
    assert_eq!(
        u(&sim, "t3"),
        0x5678,
        "packed+unpacked indexes the UNPACKED dimension"
    );
    assert_eq!(u(&sim, "m1"), 0x9A, "plain unpacked array unaffected");
    assert_eq!(u(&sim, "q0"), 42, "queue unaffected");
}

#[test]
fn omitted_type_parameter_takes_its_declared_default() {
    let sim = simulate(TYPE_PARAM_DEFAULT, 100).expect("simulate failed");
    assert_eq!(u(&sim, "all_def"), 0xC0FEBABE, "both type params defaulted");
    assert_eq!(
        u(&sim, "t_given"),
        0xC0FEBABE,
        "C defaulted while T is explicit"
    );
    assert_eq!(
        u(&sim, "both_given"),
        0xC0FEBABE,
        "both written out (already worked)"
    );
    assert_eq!(
        u(&sim, "c_over"),
        0x5A5A5A5A,
        "an explicit C still wins over the default"
    );
}
