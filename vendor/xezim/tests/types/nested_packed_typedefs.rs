//! §6.18 / §7.2.1 — packed struct typedefs whose MEMBERS are themselves
//! typedef'd structs. Both defects below are reference-validated.
//!
//! 1. **The typedef's width depended on HashMap iteration order.** A struct
//!    typedef's width is resolved from the width map built by the typedefs
//!    processed before it, and a member type that is not yet in that map falls
//!    back to 32 bits. Compilation-unit typedefs were processed by iterating a
//!    `HashMap`, so whenever the outer typedef hashed ahead of the inner one,
//!    `typedef struct packed { sp_t inner; logic [15:0] tail; } via_t;` came
//!    out 48 bits instead of 32 — and every signal declared with it was too
//!    wide. What made this pathological is that the order depends on the map's
//!    CONTENTS: adding an unrelated, uninstantiated, empty module to the file
//!    flipped it. The same typedefs declared inside a module were always fine,
//!    which is what disguised it as an instance-scope problem.
//!    Typedefs are now processed in dependency order.
//!
//! 2. **A two-level member read through an instance returned x.** Resolution
//!    split the dotted name at the LAST dot only, so `u.s.inner.hi` was tried
//!    as the signal `u.s.inner` plus the field `hi`, which names no layout. At
//!    top level `s.inner.hi` worked because each nested sub-struct also gets a
//!    layout of its own under `s.inner`; through an instance no such key
//!    exists. Every split point is now tried, longest base first — the layout
//!    of `u.s` already records nested members under their dotted path.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// `$bits` of a nested struct typedef, at compilation-unit scope, with a second
/// module present — the exact shape that flipped the iteration order. Several
/// differently-named chains are declared so no single hash ordering can make
/// them all accidentally correct.
#[test]
fn nested_struct_typedef_widths_are_order_independent() {
    let src = r#"
typedef struct packed { logic [7:0] hi; logic [7:0] lo; } sp_t;
typedef struct packed { sp_t inner; logic [15:0] tail; } via_typedef_t;
typedef struct packed { via_typedef_t deep; logic [7:0] t2; } three_level_t;
typedef struct packed { logic [3:0] a; logic [3:0] b; } aaa_t;
typedef struct packed { aaa_t x; aaa_t y; } zzz_t;
typedef struct packed { zzz_t p; aaa_t q; } mmm_t;
typedef union  packed { sp_t s; logic [15:0] w; } un_t;
module leaf; endmodule   // never instantiated: presence alone used to matter
module tb;
  int w_sp, w_via, w_three, w_zzz, w_mmm, w_un;
  initial begin
    w_sp    = $bits(sp_t);
    w_via   = $bits(via_typedef_t);
    w_three = $bits(three_level_t);
    w_zzz   = $bits(zzz_t);
    w_mmm   = $bits(mmm_t);
    w_un    = $bits(un_t);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w_sp"), 16);
    assert_eq!(
        u(&sim, "w_via"),
        32,
        "member typedef must not fall back to 32 bits"
    );
    assert_eq!(u(&sim, "w_three"), 40);
    assert_eq!(u(&sim, "w_zzz"), 16);
    assert_eq!(u(&sim, "w_mmm"), 24);
    assert_eq!(
        u(&sim, "w_un"),
        16,
        "a packed union is as wide as its widest member"
    );
}

/// A signal of a nested typedef must be exactly as wide as the type, so a whole
/// read has no extra high bits.
#[test]
fn nested_typedef_signal_is_not_over_wide() {
    let src = r#"
typedef struct packed { logic [7:0] hi; logic [7:0] lo; } sp_t;
typedef struct packed { sp_t inner; logic [15:0] tail; } via_t;
module leaf;
  via_t v;
  initial v = 32'h11223344;
endmodule
module tb;
  leaf u();
  int whole, w;
  initial begin
    #1;
    whole = u.v;
    w     = $bits(u.v);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w"), 32);
    assert_eq!(u(&sim, "whole"), 0x1122_3344);
}

/// Two-level member reads, from inside the child and hierarchically from the
/// parent, for a nested struct and for a union whose member is a struct.
#[test]
fn two_level_member_reads_through_an_instance() {
    let src = r#"
typedef struct packed { logic [7:0] hi; logic [7:0] lo; } sp_t;
typedef struct packed { sp_t inner; logic [15:0] tail; } st_nest_t;
typedef union  packed { sp_t s; logic [15:0] w; } un_nest_t;
module leaf;
  st_nest_t sn;
  un_nest_t un;
  logic [7:0] in_hi, in_lo, in_un_hi;
  initial begin
    sn.inner = 16'h1234;
    un.w     = 16'hbeef;
    #1;
    in_hi    = sn.inner.hi;
    in_lo    = sn.inner.lo;
    in_un_hi = un.s.hi;
  end
endmodule
module tb;
  leaf u();
  int out_hi, out_lo, out_un_hi, out_un_lo, tail_x;
  initial begin
    #2;
    out_hi    = u.sn.inner.hi;
    out_lo    = u.sn.inner.lo;
    out_un_hi = u.un.s.hi;
    out_un_lo = u.un.s.lo;
    tail_x    = $isunknown(u.sn.tail);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "u.in_hi"), u(&sim, "u.in_lo")),
        (0x12, 0x34),
        "read inside the child"
    );
    assert_eq!(
        u(&sim, "u.in_un_hi"),
        0xbe,
        "union member read inside the child"
    );
    assert_eq!(
        (u(&sim, "out_hi"), u(&sim, "out_lo")),
        (0x12, 0x34),
        "read from the parent"
    );
    assert_eq!(
        (u(&sim, "out_un_hi"), u(&sim, "out_un_lo")),
        (0xbe, 0xef),
        "union overlay"
    );
    assert_eq!(u(&sim, "tail_x"), 1, "an unwritten member stays x");
}

/// A member write through the OTHER member of a union, inside an instance,
/// lands at the right offset.
#[test]
fn union_member_write_through_a_nested_field() {
    let src = r#"
typedef struct packed { logic [7:0] hi; logic [7:0] lo; } sp_t;
typedef union  packed { sp_t s; logic [15:0] w; } un_t;
module leaf;
  un_t un;
  initial begin
    un.w    = 16'h0000;
    un.s.hi = 8'h12;
  end
endmodule
module tb;
  leaf u();
  int whole;
  initial begin
    #1;
    whole = u.un.w;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "whole"), 0x1200);
}
