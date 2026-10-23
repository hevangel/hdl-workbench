//! §6.18 / §26.3 / §3.12.1: a typedef's dimensions and enum member values are
//! evaluated in the scope where the typedef is DECLARED — 16 ivtest cases
//! (`sv_typedef_{array,queue,darray}_base1-4`, `sv_typedef_chained`,
//! `sv_typedef_fwd_*`, `sv_ps_type_{enum,struct}1`), all reference-validated.
//!
//! Three distinct defects:
//! 1. **$unit/package typedefs resolved dims in the USE scope.** $unit
//!    localparams are injected into module bodies (and skipped when
//!    shadowed), so `localparam A=8; typedef logic [A-1:0] T[1:0];` in a
//!    module declaring `localparam A=4` sized elements at 4 bits — or 1 bit
//!    when shadowed, since the typedef then saw no `A` at all. Fixed by
//!    folding typedef dims (packed, unpacked, TypeReference dims, struct
//!    member types, enum bases AND enum member initializers) to literals at
//!    capture time, against the declaring scope's parameter environment —
//!    per-package tables, so same-named localparams in different packages
//!    cannot bleed into each other.
//! 2. **A scoped `P::T` was captured by a module-local `typedef ... T`.**
//!    Package typedefs register only bare names in the flat table. They are
//!    now mirrored under `pkg::name`, and every TypeReference lookup
//!    (width, signedness, typedef chain) tries the qualified key first.
//! 3. **`typedef struct T;` (keyword-qualified forward form) parsed as a
//!    BODYLESS struct** and re-registered T at width 0 over the real
//!    definition. Parsed as forward now; module typedefs are also
//!    pre-registered before the item walk so `typedef T; T x; typedef int
//!    T;` sizes `x` correctly (§6.18 use-before-completion).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("test.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// $unit typedef dims use the $unit localparam even when the module shadows it.
#[test]
fn unit_typedef_dims_use_declaring_scope() {
    let src = r#"
localparam A = 8;
typedef logic [A-1:0] T[1:0];
typedef logic [A-1:0] S;
typedef struct packed { logic [A-1:0] f; } PS[1:0];
module test;
  localparam A = 4;
  T x;
  S y;
  PS z;
  int bx, by, bz, keep;
  initial begin
    x[0] = 8'hff; y = 8'hff; z[0] = 8'hff;
    bx = $bits(x[0]); by = $bits(y); bz = $bits(z[0]);
    keep = A;  // the module's own A is untouched
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "bx"), 8, "array element from the $unit A");
    assert_eq!(u(&sim, "by"), 8, "scalar typedef");
    assert_eq!(u(&sim, "bz"), 8, "struct member dim");
    assert_eq!(u(&sim, "keep"), 4, "the module's shadowing A still reads 4");
}

/// Chained package typedefs: each link resolves in its own package's scope,
/// and a qualified reference beats a same-named local typedef.
#[test]
fn package_typedefs_resolve_in_their_own_scope() {
    let src = r#"
package P1;
  localparam A = 8;
  typedef logic [A-1:0] T;
endpackage
package P2;
  localparam A = 4;
  typedef P1::T T;
endpackage
package P3;
  localparam X = 8;
  typedef enum logic [X-1:0] { EA, EB = X } E;
  typedef struct packed { logic [X-1:0] f; } S;
endpackage
module test;
  localparam A = 2;
  typedef int T;
  P1::T a;
  P2::T b;
  T c;
  P3::E e;
  P3::S s;
  int ba, bb, bc, bs, ev;
  initial begin
    a = 8'hff; b = 8'hff; c = -1;
    e = P3::EB; s = 8'hff;
    ba = $bits(a); bb = $bits(b); bc = $bits(c);
    bs = $bits(s); ev = e;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "ba"), 8, "qualified P1::T");
    assert_eq!(u(&sim, "bb"), 8, "chained through P2");
    assert_eq!(u(&sim, "bc"), 32, "the local T is still int");
    assert_eq!(u(&sim, "bs"), 8, "package struct through a qualified ref");
    assert_eq!(u(&sim, "ev"), 8, "enum member value from the PACKAGE's X");
}

/// §6.18 forward typedefs: bare and keyword-qualified, with uses between the
/// forward and the completion.
#[test]
fn forward_typedefs_complete_later() {
    let src = r#"
module test;
  typedef T1;
  typedef struct T2;
  typedef union T3;
  T1 x;
  T2 s;
  typedef int T1;
  typedef struct packed { int f; } T2;
  typedef union packed { int a; int b; } T3;
  T1 y;
  T3 w;
  int bx, bs, by, bw;
  initial begin
    x = -1;
    bx = $bits(x); bs = $bits(s); by = $bits(y); bw = $bits(w);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "bx"), 32, "use between forward and completion");
    assert_eq!(u(&sim, "bs"), 32, "keyword-qualified struct forward");
    assert_eq!(u(&sim, "by"), 32, "use after completion");
    assert_eq!(u(&sim, "bw"), 32, "keyword-qualified union forward");
}
