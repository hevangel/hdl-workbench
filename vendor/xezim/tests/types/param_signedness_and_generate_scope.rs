//! Two parameter defects found by auditing §6.20 / §11.8 / §27.6.
//!
//! 1. **Signedness lost against an unsized literal (const-eval only).**
//!    §11.8.2 step 2 converts every operand of a signed expression to the
//!    expression's width by SIGN extension. `Value::add/sub/mul` widened via
//!    `to_u64`, which zero-extends — so an 8-bit `parameter signed [7:0]`
//!    meeting a 32-bit literal arrived as its unsigned bit pattern and
//!    `SP * 2` evaluated to 506 instead of -6. `div`/`mod` already used
//!    `to_i64`, which is why only `+`/`-`/`*` were wrong, and only when the
//!    operand widths DIFFERED (two params of equal width were fine).
//!
//!    The same expression evaluated at RUNTIME was always correct, because the
//!    simulator pre-extends operands before calling these. That asymmetry is
//!    what made it hard to see: `localparam D = SP * 2;` was wrong while
//!    `d = SP * 2;` in an initial block was right.
//!
//! 2. **Hierarchical references into a generate block.** §27.6 makes a
//!    labelled generate block a scope, so `gblk.u.ID` and `g[0].u.ID` are
//!    legal. The undeclared-identifier validator runs BEFORE generate
//!    inlining creates the flattened `g[0].u.ID` keys, so it saw the bare root
//!    (`g`, `gblk`) and rejected it — failing elaboration of the whole
//!    enclosing block. Instance names were already pre-registered for exactly
//!    this reason, which is why a non-generate `u1.ID` worked.
//!
//! All expectations are byte-identical to a reference simulator.

use xezim::simulate;

fn i(sim: &xezim::compiler::Simulator, n: &str) -> i64 {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n));
    v as u32 as i32 as i64
}

/// A signed parameter keeps its sign against an unsized literal, for every
/// arithmetic operator, in a constant expression.
#[test]
fn signed_parameter_sign_extends_against_a_literal() {
    let src = r#"
module top;
  parameter signed [7:0] SP = -3;
  localparam MUL = SP * 2;
  localparam SUB = SP - 1;
  localparam ADD = SP + 1;
  localparam DIV = SP / 1;
  localparam MOD = SP % 5;
  localparam NEG = -SP;
  localparam SHR = SP >>> 1;
  int mul, sub, add, dv, md, ng, sr;
  initial begin
    mul = MUL; sub = SUB; add = ADD; dv = DIV; md = MOD; ng = NEG; sr = SHR;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(i(&sim, "mul"), -6, "SP * 2");
    assert_eq!(i(&sim, "sub"), -4, "SP - 1");
    assert_eq!(i(&sim, "add"), -2, "SP + 1");
    assert_eq!(i(&sim, "dv"), -3, "SP / 1 (already worked)");
    assert_eq!(i(&sim, "md"), -3, "SP % 5 (already worked)");
    assert_eq!(i(&sim, "ng"), 3, "-SP");
    assert_eq!(i(&sim, "sr"), -2, "SP >>> 1");
}

/// Const-eval and runtime must agree — the divergence between them is what
/// hid this bug.
#[test]
fn const_eval_and_runtime_agree_on_signed_arithmetic() {
    let src = r#"
module top;
  parameter signed [7:0] SP = -3;
  localparam C_MUL = SP * 2;
  localparam C_SUB = SP - 1;
  int c_mul, c_sub, r_mul, r_sub;
  initial begin
    c_mul = C_MUL; c_sub = C_SUB;
    r_mul = SP * 2; r_sub = SP - 1;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        i(&sim, "c_mul"),
        i(&sim, "r_mul"),
        "mul: const-eval == runtime"
    );
    assert_eq!(
        i(&sim, "c_sub"),
        i(&sim, "r_sub"),
        "sub: const-eval == runtime"
    );
    assert_eq!(i(&sim, "c_mul"), -6);
    assert_eq!(i(&sim, "c_sub"), -4);
}

/// The guard: an UNSIGNED parameter of the same width must still wrap, and two
/// same-width signed parameters (which already worked) must not change.
#[test]
fn unsigned_parameters_and_equal_width_operands_are_unchanged() {
    let src = r#"
module top;
  parameter        [7:0] UP = -3;   // unsigned: 253
  parameter signed [7:0] SP = -3;
  parameter signed [7:0] SQ = 4;
  localparam U_ADD  = UP + 1;
  localparam SS_MUL = SP * SQ;
  localparam MIXED  = UP + SP;      // any operand unsigned -> unsigned (§11.8.1)
  int u_add, ss_mul, mixed, up;
  initial begin
    u_add = U_ADD; ss_mul = SS_MUL; mixed = MIXED; up = UP;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        i(&sim, "up"),
        253,
        "an unsigned parameter keeps its bit pattern"
    );
    assert_eq!(
        i(&sim, "u_add"),
        254,
        "unsigned arithmetic does not sign-extend"
    );
    assert_eq!(i(&sim, "ss_mul"), -12, "two signed params of equal width");
    assert_eq!(
        i(&sim, "mixed"),
        250,
        "one unsigned operand makes it unsigned"
    );
}

/// A third defect, pre-existing and surfaced by fixing the first: a parameter
/// whose DECLARED type is unsigned kept the signedness of its negative
/// INITIALIZER. `parameter [7:0] UP = -3` behaved as -3 in every
/// signed-sensitive operation, so `UP >>> 1` was -2 rather than 126.
///
/// It hid because `+`/`-`/`*` never sign-extended, which made the common cases
/// look right by accident; only the operators that DID honour the flag
/// (`>>>`, `/`, `<`) exposed it.
#[test]
fn an_unsigned_parameter_does_not_inherit_its_initializers_sign() {
    let src = r#"
module top;
  parameter [7:0] UP = -3;         // unsigned by declaration -> 253
  localparam SHR  = UP >>> 1;
  localparam LT   = UP < 0;
  localparam DIVU = UP / 2;
  parameter IMP = -3;              // §6.20.2 implicit -> signed 32-bit
  localparam I_SHR = IMP >>> 1;
  int shr, lt, divu, i_shr;
  initial begin
    shr = SHR; lt = LT; divu = DIVU; i_shr = I_SHR;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        i(&sim, "shr"),
        126,
        "arithmetic shift of an unsigned parameter"
    );
    assert_eq!(i(&sim, "lt"), 0, "an unsigned parameter is never < 0");
    assert_eq!(i(&sim, "divu"), 126, "unsigned division");
    assert_eq!(
        i(&sim, "i_shr"),
        -2,
        "an implicit-type parameter IS signed (§6.20.2)"
    );
}

/// §27.6: reach into a NAMED if-generate block by its label.
#[test]
fn hierarchical_reference_into_a_named_generate_block() {
    let src = r#"
module cellx #(parameter int ID = 0) (output int o);
  assign o = ID * 10;
endmodule
module top;
  int val;
  generate
    if (1) begin : gblk
      cellx #(.ID(9)) u (val);
    end
  endgenerate
  int seen;
  initial begin
    #1 seen = gblk.u.ID;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(i(&sim, "seen"), 9, "gblk.u.ID resolves");
    assert_eq!(
        i(&sim, "val"),
        90,
        "and the block still elaborates normally"
    );
}

/// §27.6: reach into an INDEXED for-generate iteration.
#[test]
fn hierarchical_reference_into_an_indexed_generate_block() {
    let src = r#"
module cellx #(parameter int ID = 0) (output int o);
  assign o = ID * 10;
endmodule
module top;
  localparam int N = 3;
  int outs[N];
  genvar k;
  generate
    for (k = 0; k < N; k++) begin : g
      cellx #(.ID(k + 1)) u (outs[k]);
    end
  endgenerate
  int id0, id1, id2, o0, o2;
  initial begin
    #1;
    id0 = g[0].u.ID; id1 = g[1].u.ID; id2 = g[2].u.ID;
    o0 = outs[0]; o2 = outs[2];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(i(&sim, "id0"), 1, "g[0].u.ID");
    assert_eq!(i(&sim, "id1"), 2, "g[1].u.ID");
    assert_eq!(i(&sim, "id2"), 3, "g[2].u.ID");
    assert_eq!(i(&sim, "o0"), 10, "the generated instances still work");
    assert_eq!(i(&sim, "o2"), 30);
}

/// The guard: a genuinely undeclared identifier is still rejected — making
/// generate labels legal scope roots must not disable the check.
#[test]
fn a_genuinely_undeclared_identifier_is_still_an_error() {
    let src = r#"
module top;
  int a;
  initial a = no_such_signal_anywhere;
endmodule
"#;
    let err = match simulate(src, 20) {
        Ok(_) => panic!("must reject an undeclared identifier"),
        Err(e) => e.to_string(),
    };
    assert!(
        err.contains("no_such_signal_anywhere"),
        "the error should name it: {err}"
    );
}
