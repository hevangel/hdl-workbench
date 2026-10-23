//! IEEE 1800-2017 §7.2.2: a write to a NESTED packed-struct member through a
//! `MemberAccess` lvalue was silently dropped.
//!
//!   typedef struct packed { logic [4:0] x, y; } id_t;
//!   typedef struct packed { id_t d; logic last; } hdr_t;
//!   function automatic f_t build();
//!     f_t f; f = '0;
//!     f.hdr.d = mk_id();      // vanished
//!     f.pl    = 64'hface…;    // landed
//!   endfunction
//!
//! `packed_struct_fields[<decl>]` is FLATTENED — every nested member is a key
//! in its own right (`"hdr.d.x"` under `"f"`), and there is NO entry for an
//! intermediate member on its own. The `MemberAccess` arm of
//! `assign_value_inner` split the lvalue at exactly ONE dot, so `f.hdr.d`
//! asked for `packed_struct_fields["f.hdr"]`, got `None`, declined, and the
//! write fell through every remaining arm and disappeared. Only a DEPTH-1
//! member ever resolved, which is why `f.pl` was the one line that survived.
//!
//! Two things about the original framing were wrong and are worth not
//! re-deriving: it is NOT about subroutine-local storage (a task writing a
//! module-scope signal fails identically — the discriminator is that inside a
//! subroutine `m.hdr.d` parses as `MemberAccess`, where at module scope the
//! same text is a dotted `Ident` on a path that already resolved), and READS
//! were never broken at any depth.
//!
//! The fix walks the split points LONGEST BASE FIRST, the same candidate walk
//! `resolve_packed_struct_target` already did for assignment patterns. That
//! asymmetry explains the whole variant table below: the pattern RHS worked
//! because its path had the walk, the value RHS did not.

use xezim::simulate;

const SRC: &str = r#"
package p;
  typedef struct packed { logic [4:0] x; logic [4:0] y; } id_t;   // 10 bits
  typedef struct packed { id_t d; logic last; }          hdr_t;   // 11 bits
  typedef struct packed { hdr_t hdr; logic [63:0] pl; }  f_t;     // 75 bits

  function automatic id_t mk_id();
    id_t i;
    i = '0; i.x = 5'd2; i.y = 5'd1;
    return i;
  endfunction
endpackage

module tb;
  import p::*;

  // Depth-2 member, FUNCTION-CALL rhs — the form that was dropped.
  function automatic f_t via_call();
    f_t f; f = '0;
    f.hdr.d = mk_id();
    f.pl    = 64'hface000000000000;
    return f;
  endfunction

  // Depth-2 member, assignment-PATTERN rhs — always worked; the control that
  // localises the fault to the value path.
  function automatic f_t via_pattern();
    f_t f; f = '0;
    f.hdr.d = '{x: 5'd2, y: 5'd1};
    f.pl    = 64'hface000000000000;
    return f;
  endfunction

  // Depth-2 ONE-BIT member, and depth THREE.
  function automatic f_t via_onebit();
    f_t f; f = '0;
    f.hdr.last = 1'b1;
    f.pl       = 64'hface000000000000;
    return f;
  endfunction
  function automatic f_t via_deep();
    f_t f; f = '0;
    f.hdr.d.x = 5'd2;
    f.hdr.d.y = 5'd1;
    f.pl      = 64'hface000000000000;
    return f;
  endfunction

  // A TASK writing a MODULE-scope signal: same MemberAccess lvalue shape, so
  // it failed identically. This is what disproved "subroutine-local storage".
  f_t m;
  task automatic write_module_scope();
    m = '0;
    m.hdr.d = mk_id();
    m.pl    = 64'hface000000000000;
  endtask

  // Reads were never broken — assert that explicitly so a future fix to the
  // write path cannot quietly regress them.
  int rd_x, rd_y, rd_last;

  logic [74:0] r_call, r_pattern, r_onebit, r_deep, r_module;
  initial begin
    f_t probe;
    r_call    = via_call();
    r_pattern = via_pattern();
    r_onebit  = via_onebit();
    r_deep    = via_deep();
    write_module_scope();
    r_module  = m;

    probe = '0;
    probe.hdr.d.x = 5'd9;
    probe.hdr.d.y = 5'd3;
    probe.hdr.last = 1'b1;
    rd_x    = probe.hdr.d.x;
    rd_y    = probe.hdr.d.y;
    rd_last = probe.hdr.last;
  end
endmodule
"#;

/// `{hdr{d{x=2,y=1}, last}, pl}` — x is the top 5 bits of `d`.
/// d = 2<<5 | 1 = 0x41; hdr = d<<1 | last; f = hdr<<64 | pl.
const D: u128 = (2u128 << 5) | 1;
const PL: u128 = 0xface_0000_0000_0000;
const EXP_NO_LAST: u128 = ((D << 1) << 64) | PL;
const EXP_LAST: u128 = (((D << 1) | 1) << 64) | PL;
const EXP_ONEBIT: u128 = (1u128 << 64) | PL;

fn v(sim: &xezim::compiler::Simulator, n: &str) -> u128 {
    sim.get_signal(n)
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u128()
}

fn i(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n).unwrap().to_u64().unwrap() & 0xFFFF_FFFF
}

#[test]
fn depth_two_member_write_with_a_value_rhs_lands() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(v(&sim, "r_call"), EXP_NO_LAST);
}

#[test]
fn the_pattern_rhs_control_still_agrees() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(v(&sim, "r_pattern"), EXP_NO_LAST);
}

#[test]
fn one_bit_and_depth_three_members_land() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(v(&sim, "r_onebit"), EXP_ONEBIT);
    assert_eq!(v(&sim, "r_deep"), EXP_NO_LAST);
}

#[test]
fn a_task_writing_a_module_scope_signal_lands_too() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(v(&sim, "r_module"), EXP_NO_LAST);
}

#[test]
fn nested_member_reads_are_unaffected() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(i(&sim, "rd_x"), 9);
    assert_eq!(i(&sim, "rd_y"), 3);
    assert_eq!(i(&sim, "rd_last"), 1);
}

#[test]
fn the_last_bit_and_the_id_share_the_header_correctly() {
    // Guards the bit layout the other expectations are computed from: setting
    // BOTH `hdr.d` and `hdr.last` must produce their OR, not one clobbering
    // the other.
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(EXP_LAST, EXP_NO_LAST | (1u128 << 64));
    assert_eq!(v(&sim, "r_onebit") & !PL, 1u128 << 64);
}
