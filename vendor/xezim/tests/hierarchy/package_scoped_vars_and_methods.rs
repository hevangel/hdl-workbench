//! §26.3: package-scoped VARIABLES and everything hanging off them — the
//! ivtest `sv_ps_method1-4` / `sv_ps_member_sel2-3` cluster, all
//! reference-validated.
//!
//! Five defects, one root: a package's SCALAR data declarations were never
//! registered (only arrays were), and the scoped spelling `P::x` resolved
//! through shape-keyed branches that only knew the bare form.
//!
//! 1. Package scalar vars: no signal, no initializer (`P::e` read x), no
//!    anonymous-enum member registration.
//! 2. `C c = new;` at package scope: bare `new` parses as an IDENT, not a
//!    Call, so the "defer side-effecting initializers to a static init"
//!    check missed it and the handle stayed null.
//! 3. `P::e.next(1)` — the CALL form with args: the general-receiver enum
//!    block only accepted no-arg calls, and the flattened `[P, e, next]`
//!    dispatch had no enum branch at all.
//! 4. `P::c.f1(10)` — a method call through a package-scoped class handle:
//!    the flattened path treated `P` as the receiver.
//! 5. `P::s.x` — member selects: every struct-slice branch keys on exact
//!    2-segment shapes, so the 3-segment scoped form read x. The package
//!    prefix is now stripped once at eval/call/name-resolution entry
//!    (guarded: only a REGISTERED package name that no signal/class shadows).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("test.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Package enum variable: value, methods (bare and stepped), qualified and
/// imported spellings.
#[test]
fn package_enum_variable_and_methods() {
    let src = r#"
package P;
  enum integer { A, B, C } e = B;
endpackage
module test;
  import P::*;
  int val, fst, nxt1, nxt2, prv1, wrap, num_m;
  initial begin
    val  = P::e;
    fst  = P::e.first;
    nxt1 = P::e.next(1);
    nxt2 = P::e.next(2);   // wraps past C to A
    prv1 = P::e.prev(1);
    wrap = e.prev(2);      // imported spelling, stepped back past A
    num_m = P::e.num;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "val"), 1, "initializer applied");
    assert_eq!(u(&sim, "fst"), 0);
    assert_eq!(u(&sim, "nxt1"), 2);
    assert_eq!(u(&sim, "nxt2"), 0, "next wraps");
    assert_eq!(u(&sim, "prv1"), 0);
    assert_eq!(u(&sim, "wrap"), 2, "prev wraps");
    assert_eq!(u(&sim, "num_m"), 3);
}

/// Package class-handle variable constructed by a bare `new`, methods called
/// through both spellings.
#[test]
fn package_class_handle_with_bare_new_initializer() {
    let src = r#"
package P;
  class C;
    int base = 5;
    function int f1(int x); return x + base; endfunction
  endclass
  C c = new;
endpackage
module test;
  import P::*;
  int notnull, scoped, imported;
  initial begin
    notnull  = (c != null);
    scoped   = P::c.f1(10);
    imported = c.f1(20);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "notnull"), 1, "the static init ran the constructor");
    assert_eq!(u(&sim, "scoped"), 15, "P::c.f1(10)");
    assert_eq!(u(&sim, "imported"), 25);
}

/// Package packed-struct variable: whole, member, and bit-of-member reads
/// through the scoped spelling.
#[test]
fn package_struct_member_selects() {
    let src = r#"
package P;
  localparam N = 1;
  struct packed { logic [3:0] x; } s = 4'b0101;
endpackage
module test;
  localparam N = 2;
  int whole, memb, bit_n;
  initial begin
    whole = P::s;
    memb  = P::s.x;
    bit_n = P::s.x[N];   // the USE-scope N indexes the select (§26.3)
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "whole"), 0b0101);
    assert_eq!(
        u(&sim, "memb"),
        0b0101,
        "member read through the scoped path"
    );
    assert_eq!(u(&sim, "bit_n"), 1, "s.x[2] with the module's N=2");
}
