//! ivtest `sv_typedef_circular1`/`2`: a circular type definition must be
//! DIAGNOSED, not crash the tool. Both aborted with "has overflowed its stack".
//!
//! Three separate walkers recursed on the cycle — `is_type_signed_resolved`,
//! `flatten_subfields`, and `flatten_struct_fields` — so guarding them one by
//! one was whack-a-mole (and bounding depth alone still spun for minutes,
//! because each level of a packed-array-of-struct member MULTIPLIES the
//! generated field names). The root fix is to reject the cycle at typedef
//! registration: `struct_typedef_self_reference` already looked for a struct
//! that transitively contains itself, but it bailed unless the TOP-LEVEL type
//! was a struct, so a cycle closing through a typedef alias slipped past.
//!
//! Both shapes are checked here, and — just as important — that ordinary
//! self-referential-looking-but-legal types still elaborate.

use xezim::simulate;

fn err(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("expected a circular-typedef diagnostic, but elaboration succeeded"),
        Err(e) => e,
    }
}

/// Cycle through typedef ALIASES only, no struct member on the path.
#[test]
fn alias_only_cycle_is_diagnosed() {
    let src = r#"
module tb;
  typedef T1;
  typedef T1 T2;
  typedef T2 T1;
  T2 x;
endmodule
"#;
    let e = err(src);
    assert!(
        e.contains("T1") && e.to_lowercase().contains("circular"),
        "want a circular-definition diagnostic naming T1; got: {e}"
    );
}

/// Longer cycle that closes through a packed struct member and a packed array
/// alias — the shape that outlived the first fix.
#[test]
fn cycle_through_struct_member_is_diagnosed() {
    let src = r#"
module tb;
  typedef T1;
  typedef struct packed { T1 x; } T2;
  typedef T2 [1:0] T3;
  typedef T3 T1;
  T1 x;
endmodule
"#;
    let e = err(src);
    assert!(
        e.contains("T1"),
        "want a diagnostic naming the offending type; got: {e}"
    );
}

/// The guard must not reject legitimate types: chained aliases, a struct that
/// merely CONTAINS another struct, and a forward typedef completed later.
#[test]
fn legal_typedef_chains_still_elaborate() {
    let src = r#"
module tb;
  typedef logic [7:0] byte_t;
  typedef byte_t      alias_t;
  typedef alias_t     alias2_t;
  typedef struct packed { byte_t a; byte_t b; } pair_t;
  typedef struct packed { pair_t lo; pair_t hi; } quad_t;
  typedef pair_t [1:0] pair_arr_t;
  typedef fwd_t;                       // forward, completed below
  typedef struct packed { byte_t v; } fwd_t;
  alias2_t   w;
  quad_t     q;
  pair_arr_t pa;
  fwd_t      f;
  int total;
  initial begin
    w = 8'hA5;
    q.lo.a = 8'h11; q.hi.b = 8'h22;
    pa[1].a = 8'h33;
    f.v = 8'h44;
    total = w + q.lo.a + q.hi.b + pa[1].a + f.v;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("legal typedef chains must still elaborate");
    let total = sim
        .get_signal("total")
        .or_else(|| sim.get_signal("tb.total"))
        .expect("total")
        .to_u64()
        .expect("total value");
    assert_eq!(
        total,
        0xA5 + 0x11 + 0x22 + 0x33 + 0x44,
        "legal nested/aliased packed types must still resolve members"
    );
}
