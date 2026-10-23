//! IEEE 1800-2017 §7.4.2 / §8.9: element writes to a CLASS-PROPERTY array of
//! UNPACKED structs went to a different store than element reads.
//!
//!   class C;
//!     typedef struct { int unsigned s; bit [1:0] x; } u_t;
//!     u_t A [3];
//!     function void go();
//!       A = '{ '{4,2'd2}, '{5,2'd1}, '{6,2'd3} };   // stored, then unreachable
//!       $display("%0d", A[0].s);                    // 0
//!     endfunction
//!   endclass
//!
//! Two stores for one datum. `queue_elem_struct` matched a class property
//! that is a FIXED ARRAY as though it were a queue, so a whole-ELEMENT write
//! took the instance-scoped SIGNAL route queues legitimately use — while a
//! fixed array's reads resolve through `class_unpacked_leaf` into the
//! instance PROPERTY map. Caught by printing the two keys side by side:
//!
//!   [B22] SIGWRITE "1#BELEM[0].s" = 4
//!         read consults heap[1].properties["BELEM[0].s"]
//!
//! The per-LEAF write (`arr[i].m = v`) and a scalar-struct property write
//! both already used the property map, which is why those were the only forms
//! that ever worked — and why the failure first looked like an initializer
//! bug rather than a storage split.
//!
//! Three earlier framings were wrong and are recorded so they are not
//! re-derived: it is not about unpacked-vs-packed element type (that
//! comparison confounded element type with storage class), not about
//! `localparam` (a plain runtime-assigned property fails identically), and
//! not a hard `Value::zero` in the read path (the read path is correct
//! throughout).
//!
//! Deliberately NOT covered here: a SCALAR unpacked-struct property written
//! with a pattern (`u_t S; S = '{9,2'd1};`) still reads back 0. That is a
//! separate defect on a separate path — it has no array element and never
//! reaches any of the three sites below — and it predates this fix.
//!
//! The fix asks the READ resolver where the leaves live and writes there, at
//! the three sites that were bypassing it. The guard is a QUESTION to the
//! read path rather than a heuristic, so a genuine queue/dynamic/assoc
//! element is not claimed and keeps its signal-name copy — which UVM's
//! `uvm_hdl_path_concat::add_slice` depends on.

use xezim::simulate;

const SRC: &str = r#"
module tb;
  typedef struct { int unsigned s; bit [1:0] x; } u_t;

  // MODULE scope: always worked, and is the control that isolates the fault
  // to per-instance class storage rather than to unpacked structs.
  u_t MVAR [3];
  int m0, m1, m2;

  class C;
    u_t A  [3];                       // whole-ARRAY pattern
    u_t B  [3];                       // whole-ELEMENT, pattern rhs
    u_t D  [3];                       // whole-ELEMENT, value rhs
    u_t E  [3];                       // per-LEAF (already worked)
    localparam u_t LP [3] = '{ '{4,2'd2}, '{5,2'd1}, '{6,2'd3} };
    const     u_t KP [3] = '{ '{4,2'd2}, '{5,2'd1}, '{6,2'd3} };

    function void go();
      u_t tmp;
      A = '{ '{4,2'd2}, '{5,2'd1}, '{6,2'd3} };
      B[0] = '{4,2'd2}; B[1] = '{5,2'd1}; B[2] = '{6,2'd3};
      tmp = '{7,2'd2};
      D[1] = tmp;
      E[0].s = 4; E[1].s = 5; E[2].s = 6;
    endfunction
  endclass

  C c;
  int a0, a1, a2, b0, b1, b2, d1, e0, e2;
  int lp0, lp2, kp0, kp2, a0x, b2x;
  initial begin
    MVAR = '{ '{4,2'd2}, '{5,2'd1}, '{6,2'd3} };
    m0 = MVAR[0].s; m1 = MVAR[1].s; m2 = MVAR[2].s;

    c = new();
    c.go();
    a0 = c.A[0].s; a1 = c.A[1].s; a2 = c.A[2].s;
    b0 = c.B[0].s; b1 = c.B[1].s; b2 = c.B[2].s;
    d1 = c.D[1].s;
    e0 = c.E[0].s; e2 = c.E[2].s;
    lp0 = c.LP[0].s; lp2 = c.LP[2].s;
    kp0 = c.KP[0].s; kp2 = c.KP[2].s;
    // The TRAILING member must land too, not just the first.
    a0x = c.A[0].x; b2x = c.B[2].x;
  end
endmodule
"#;

fn i(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

#[test]
fn whole_array_pattern_reaches_the_store_reads_use() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(i(&sim, "a0"), 4);
    assert_eq!(i(&sim, "a1"), 5);
    assert_eq!(i(&sim, "a2"), 6);
}

#[test]
fn whole_element_writes_reach_it_with_either_rhs() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // pattern rhs
    assert_eq!(i(&sim, "b0"), 4);
    assert_eq!(i(&sim, "b1"), 5);
    assert_eq!(i(&sim, "b2"), 6);
    // value rhs
    assert_eq!(i(&sim, "d1"), 7);
}

#[test]
fn every_member_lands_not_only_the_first() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(i(&sim, "a0x"), 2);
    assert_eq!(i(&sim, "b2x"), 3);
}

#[test]
fn class_localparam_and_const_arrays_read_back() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(i(&sim, "lp0"), 4);
    assert_eq!(i(&sim, "lp2"), 6);
    assert_eq!(i(&sim, "kp0"), 4);
    assert_eq!(i(&sim, "kp2"), 6);
}

#[test]
fn the_forms_that_already_worked_are_unchanged() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // per-leaf writes and module scope
    assert_eq!(i(&sim, "e0"), 4);
    assert_eq!(i(&sim, "e2"), 6);
    assert_eq!(i(&sim, "m0"), 4);
    assert_eq!(i(&sim, "m1"), 5);
    assert_eq!(i(&sim, "m2"), 6);
}
