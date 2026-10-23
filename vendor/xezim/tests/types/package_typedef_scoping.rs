//! §26.2/§26.3 — an unqualified type name must resolve to the package the
//! enclosing scope IMPORTS, and `$bits(P::T)` must reach THAT package's
//! typedef. Reference-validated.
//!
//! Package typedefs are hoisted into ONE global bare-name table with a
//! clobbering insert, so a same-named typedef in an unrelated package could end
//! up owning the bare key — and since the definition table is a HashMap, which
//! one won was iteration-order dependent, not source or alphabetical order.
//! (The neighbouring arms for functions/tasks/classes already avoid this: bare
//! name first-wins, plus an exact `pkg::name` key.)
//!
//! Two paths were affected:
//!
//!   * The inlined-instance path never processed imports AT ALL, so a
//!     declaration in a non-top module took whichever package last hoisted the
//!     bare name. `T [0:0][1:0] s;` on a 146-bit struct became 64*1*2 = 128,
//!     with no packed dims — so `s[0][0]` degraded to a bit-select of a
//!     bit-select and every field access read garbage. The top-module path was
//!     unaffected because it re-runs its own imports.
//!
//!   * `$bits(P::T)` dropped the qualifier and answered from the bare key.
//!
//! Reported as a 292-bit testbench signal elaborating to 128 while the DUT port
//! it connects to stayed 292.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

/// `CHIP::wdata_t` is 146 bits; `OTHER::wdata_t` is 64 and shares the bare name.
const SRC: &str = r#"
package CHIP;
   typedef struct packed {
      logic [1:0] [63:0] wdata;
      logic [1:0] [7:0]  mask;
      logic [1:0]        amask;
   } wdata_t;                                  // 146
endpackage
package OTHER;
   typedef logic [63:0] wdata_t;               // 64, same bare name
endpackage

// A NON-TOP module: its declarations resolve in the parent's walk.
module sub;
  import CHIP::*;
  wdata_t       [0:0][1:0] s_unq;              // 292, and packed
  CHIP::wdata_t [0:0][1:0] s_qual;             // 292 (was already right)
  logic [31:0] w_unq, w_qual, w_elem;
  assign w_unq  = $bits(s_unq);
  assign w_qual = $bits(s_qual);
  assign w_elem = $bits(s_unq[0][0]);
endmodule

// A module importing the OTHER one must still see 64 — the fix must bind the
// IMPORTED package, not simply prefer the widest or the first.
module sub_other;
  import OTHER::*;
  wdata_t v;
  logic [31:0] w_other;
  assign w_other = $bits(v);
endmodule

interface iface_t;
  import CHIP::*;
  wdata_t [0:0][1:0] i_sig;
endinterface

// The top imports NOTHING: its own import pre-pass would rebind the bare name
// design-wide and mask the sub-module defect being tested.
module tb;
  localparam int LP_CHIP  = $bits(CHIP::wdata_t);
  localparam int LP_OTHER = $bits(OTHER::wdata_t);
  logic [$bits(CHIP::wdata_t)-1:0] macro_decl;   // scoped $bits in a dimension

  sub        u_sub();
  sub_other  u_other();
  iface_t    u_if();

  logic [31:0] lp_chip, lp_other, w_macro, w_iface, w_top;
  CHIP::wdata_t [0:0][1:0] t_qual;
  assign lp_chip  = LP_CHIP;
  assign lp_other = LP_OTHER;
  assign w_macro  = $bits(macro_decl);
  assign w_iface  = $bits(u_if.i_sig);
  assign w_top    = $bits(t_qual);
  initial #1;
endmodule
"#;

#[test]
fn unqualified_type_in_a_non_top_module_binds_to_the_imported_package() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "u_sub.w_unq"), 292, "unqualified in a sub-module");
    assert_eq!(u(&sim, "u_sub.w_qual"), 292, "qualified in a sub-module");
    assert_eq!(
        u(&sim, "u_sub.w_elem"),
        146,
        "element select — 1 here means the signal lost its packed dims"
    );
}

#[test]
fn a_different_import_still_binds_to_its_own_package() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "u_other.w_other"),
        64,
        "importing OTHER must see OTHER's 64-bit type, not CHIP's"
    );
}

#[test]
fn interfaces_bind_their_imports_too() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w_iface"), 292);
}

#[test]
fn top_module_declarations_are_unaffected() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w_top"), 292);
}

/// The bare name must NOT simply become "whatever CHIP says" design-wide — a
/// scope that imports neither package still sees the hoisted definition, and a
/// scope importing OTHER must keep OTHER's.
#[test]
fn rebinding_is_per_scope_not_global() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "u_sub.w_unq"), 292, "sub imports CHIP");
    assert_eq!(u(&sim, "u_other.w_other"), 64, "sub_other imports OTHER");
}

#[test]
fn scoped_bits_reaches_that_packages_typedef() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "lp_chip"), 146, "$bits(CHIP::wdata_t)");
    assert_eq!(u(&sim, "lp_other"), 64, "$bits(OTHER::wdata_t)");
    assert_eq!(
        u(&sim, "w_macro"),
        146,
        "scoped $bits used as a declaration dimension"
    );
}
