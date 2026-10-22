//! §11.8.1: the DECLARED type decides signedness — an initializer's sign must
//! never leak into it. One root cause, four faces, all found via the Icarus
//! ivtest list (`sv_class_property_signed1..4`, `sv_class_method_signed1..2`)
//! and all confirmed against a reference simulator:
//!
//! 1. **Variable declarations**: `int unsigned x = -1;` / `bit [15:0] u = -1;`
//!    kept the literal's signedness, so `x > 0` was FALSE and `u + 10` was 9
//!    instead of 65545. (The parameter twin of this bug was fixed earlier —
//!    `param_signedness_and_generate_scope.rs`; these are the elaborate-time
//!    variable-declaration sites.)
//! 2. **Class properties**: same leak at class-elaboration time, and AGAIN at
//!    construction, where `property_inits` re-evaluation stored the raw
//!    literal value without re-stamping the declared width/signedness.
//! 3. **Class method returns**: a module-level `function bit [15:0] u;` had
//!    its return stamped with the declared type; the CLASS-method path only
//!    handled parameter-sized returns, so `c.u() > 0` read -1.
//! 4. **The conditional operator**: §11.4.11 makes BOTH branches determine
//!    the result type (either unsigned → result unsigned), but only the taken
//!    branch was evaluated, so `x ? s : x` with unsigned `x` gave -1 instead
//!    of 65535.
//!
//! The `?:` fix probes the untaken branch ONLY when it is a bare identifier
//! or literal: probing an arbitrary expression re-evaluates it, which doubles
//! hot-loop work and goes EXPONENTIAL on nested ternaries — two ivtest cases
//! (`br_gh661a`, `slongint_test`) went from pass to TIMEOUT before the probe
//! was narrowed. A function-call branch keeps its old typing rather than risk
//! a side effect; that trade-off is deliberate and documented here.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

fn i(sim: &xezim::compiler::Simulator, n: &str) -> i64 {
    u(sim, n) as u32 as i32 as i64
}

/// Face 1: declared-unsigned VARIABLES with negative initializers.
#[test]
fn unsigned_variable_does_not_inherit_initializer_sign() {
    let src = r#"
module top;
  int unsigned xu = -1;
  int          xs = -1;
  bit [15:0]   bu = -1;
  logic signed [15:0] ls = -1;
  int gt_u, gt_s, gt_b, gt_l, add_b, shr_u;
  initial begin
    gt_u = (xu > 0);
    gt_s = (xs > 0);
    gt_b = (bu > 0);
    gt_l = (ls > 0);
    add_b = bu + 10;
    shr_u = xu >>> 1;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "gt_u"), 1, "int unsigned holds 4294967295");
    assert_eq!(u(&sim, "gt_s"), 0, "a signed int is still negative");
    assert_eq!(u(&sim, "gt_b"), 1, "bit [15:0] holds 65535");
    assert_eq!(
        u(&sim, "gt_l"),
        0,
        "an explicitly signed vector stays signed"
    );
    assert_eq!(i(&sim, "add_b"), 65545, "unsigned arithmetic, not 9");
    assert_eq!(
        i(&sim, "shr_u"),
        2147483647,
        ">>> of unsigned shifts in zeros"
    );
}

/// Faces 2 and 3: class properties and class method returns.
#[test]
fn class_property_and_method_return_take_declared_type() {
    let src = r#"
module top;
  class C;
    shortint    s = -1;
    bit [15:0]  u = -1;
    function shortint fs; return -1; endfunction
    function bit [15:0] fu; return -1; endfunction
  endclass
  C c;
  int unsigned x = 10;
  int y = 10;
  int p_s, p_u, m_s, m_u, z_ux, z_uy, z_sx, z_sy;
  initial begin
    c = new;
    p_s = (c.s < 0);
    p_u = (c.u > 0);
    m_s = (c.fs() < 0);
    m_u = (c.fu() > 0);
    z_ux = c.u + x;   // unsigned + unsigned
    z_uy = c.u + y;   // one unsigned operand -> unsigned (§11.8.1)
    z_sx = c.s + x;
    z_sy = c.s + y;   // both signed -> signed
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "p_s"), 1, "signed property");
    assert_eq!(u(&sim, "p_u"), 1, "unsigned property: 65535, not -1");
    assert_eq!(u(&sim, "m_s"), 1, "signed method return");
    assert_eq!(u(&sim, "m_u"), 1, "unsigned method return: 65535, not -1");
    assert_eq!(i(&sim, "z_ux"), 65545);
    assert_eq!(i(&sim, "z_uy"), 65545);
    assert_eq!(i(&sim, "z_sx"), 65545, "the unsigned operand wins");
    assert_eq!(i(&sim, "z_sy"), 9, "both signed: -1 + 10");
}

/// Face 4: the conditional operator's result type spans both branches.
#[test]
fn conditional_result_is_unsigned_if_either_branch_is() {
    let src = r#"
module top;
  int unsigned x = 10;
  int y = 10;
  shortint s = -1;
  bit [15:0] u = -1;
  int t_ux, t_uy, t_sx, t_sy;
  initial begin
    t_ux = x ? u : x;
    t_uy = x ? u : y;
    t_sx = x ? s : x;   // unsigned else-branch makes the -1 read as 65535
    t_sy = x ? s : y;   // both signed: stays -1
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(i(&sim, "t_ux"), 65535);
    assert_eq!(i(&sim, "t_uy"), 65535);
    assert_eq!(
        i(&sim, "t_sx"),
        65535,
        "the untaken unsigned branch types the result"
    );
    assert_eq!(i(&sim, "t_sy"), -1, "two signed branches stay signed");
}

/// The guard: nested ternaries in a tight loop must not blow up — the type
/// probe is what sent two ivtest cases to TIMEOUT before it was narrowed.
#[test]
fn nested_ternaries_stay_linear() {
    let src = r#"
module top;
  int acc, k;
  initial begin
    acc = 0;
    for (k = 0; k < 200000; k++) begin
      acc += (k[0] ? (k[1] ? (k[2] ? (k[3] ? 1 : 2) : 3) : 4) : 5);
    end
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed (a hang here means the probe regressed)");
    // Per 16 iterations: 8 even ks give 5, k%4==1 (4 of them) give 4,
    // k%8==3 (2) give 3, k==7 gives 2, k==15 gives 1 -> 40+16+6+2+1 = 65.
    // 200000/16 = 12500 -> 812500.
    assert_eq!(u(&sim, "acc"), 812_500);
}
