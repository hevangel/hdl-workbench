//! Parameter const-eval corners from a probe sweep, all reference-validated
//! (7 probe files byte-identical):
//!
//! 1. §11.5.1: part-selects of a parameter in constant context — `P[11:4]`,
//!    `P[7-:8]`, `P[4+:8]` — had no const-eval arm and read 0.
//! 2. Non-overridden `parameter type` defaults: `$bits(T)` in a body
//!    localparam read 0 (overridden T worked).
//! 3. §6.20.2: unpacked-array params declared in a module BODY
//!    (`localparam int A [0:2] = '{7,8,9}`) read 0 everywhere; elements now
//!    register (and resolve in const context too).
//! 4. §6.24.1 casts in constant context: `int'(RP * 4.0)` (and size casts)
//!    had no const arms — real-to-int conversion ROUNDS.
//! 5. §7.2.1: struct-typed PACKAGE parameters — member selects read x at
//!    runtime and 0 in const context; `$bits(CDEF.b)` read 0; instance
//!    header struct-params and package-scoped defaults (`cfg_pkg::CDEF`)
//!    likewise.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn param_part_selects_in_const_context() {
    let src = r#"
module tb;
  localparam logic [15:0] PBITS = 16'hABCD;
  localparam logic [7:0]  A = PBITS[11:4];
  localparam logic [3:0]  B = PBITS[7:4];
  localparam logic        C = PBITS[3];
  localparam logic [7:0]  D = PBITS[7-:8];
  localparam logic [7:0]  E = PBITS[4+:8];
  int a, b, c, d, e;
  initial begin a = A; b = B; c = C; d = D; e = E; end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 0xBC, "constant range");
    assert_eq!(u(&sim, "b"), 0xC);
    assert_eq!(u(&sim, "c"), 1, "bit select");
    assert_eq!(u(&sim, "d"), 0xCD, "indexed-down");
    assert_eq!(u(&sim, "e"), 0xBC, "indexed-up");
}

#[test]
fn default_type_parameter_bits() {
    let src = r#"
module typed #(parameter type T = logic [7:0]) ();
  localparam int TW = $bits(T);
  int tw;
  initial tw = TW;
endmodule
module tb;
  typed #(.T(logic [15:0])) t1();
  typed t0();
  int w0, w1;
  initial begin #1; w0 = t0.tw; w1 = t1.tw; end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "w0"), 8, "default type parameter width");
    assert_eq!(u(&sim, "w1"), 16, "overridden type parameter width");
}

#[test]
fn body_unpacked_array_params() {
    let src = r#"
module tb;
  localparam int A [0:2] = '{7, 8, 9};
  localparam int A1 = A[1];
  int r0, r2, rc;
  initial begin r0 = A[0]; r2 = A[2]; rc = A1; end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 7);
    assert_eq!(u(&sim, "r2"), 9);
    assert_eq!(u(&sim, "rc"), 8, "element in const context");
}

#[test]
fn casts_in_const_context() {
    let src = r#"
module tb;
  localparam real RP = 2.5;
  localparam int  RI = int'(RP * 4.0);
  localparam int  RK = int'(2.4);        // rounds to 2
  localparam int  RL = int'(2.5);        // rounds away from zero: 3
  localparam logic [4:0] SZ = 5'(3'd7 + 3'd6);
  int ri, rk, rl, sz;
  initial begin ri = RI; rk = RK; rl = RL; sz = SZ; end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "ri"), 10, "int'(real expr)");
    assert_eq!(u(&sim, "rk"), 2, "rounds down");
    assert_eq!(u(&sim, "rl"), 3, "rounds away from zero");
    assert_eq!(u(&sim, "sz"), 13, "size cast const");
}

#[test]
fn struct_typed_package_params() {
    let src = r#"
package cfg_pkg;
  typedef struct packed { logic [3:0] a; logic [11:0] b; } cs_t;
  parameter cs_t CDEF = '{a: 4'h9, b: 12'hABC};
endpackage
module consume #(parameter cfg_pkg::cs_t C = cfg_pkg::CDEF) ();
  int ca, cb;
  initial begin ca = C.a; cb = C.b; end
endmodule
module tb;
  import cfg_pkg::*;
  consume c0();
  consume #(.C('{a: 4'h3, b: 12'h123})) c1();
  localparam logic [3:0] MA = CDEF.a;
  localparam int BW = $bits(CDEF.b);
  int ma, bw, d_a, d_b, o_a, o_b;
  initial begin
    #1;
    ma = MA; bw = BW;
    d_a = c0.ca; d_b = c0.cb;
    o_a = c1.ca; o_b = c1.cb;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "ma"), 9, "package struct-param member in const");
    assert_eq!(u(&sim, "bw"), 12, "$bits of struct member");
    assert_eq!(
        u(&sim, "d_a"),
        9,
        "default (package-scoped) header param member"
    );
    assert_eq!(u(&sim, "d_b"), 0xABC);
    assert_eq!(u(&sim, "o_a"), 3, "overridden header param member");
    assert_eq!(u(&sim, "o_b"), 0x123);
}

/// §6.20.2 / §6.16 / §20.5, round 3 of the parameter probe sweep:
/// multi-dimensional packed and unpacked parameter arrays, string-parameter
/// text concatenation, and real→int conversion in constant context.
#[test]
fn multidim_string_and_real_parameters() {
    let src = r#"
module tb;
  // multi-dim PACKED parameter: element select, not bit select
  localparam logic [1:0][7:0] MP = '{8'hAA, 8'hBB};
  localparam logic [7:0] MP0 = MP[0];
  localparam logic [7:0] MP1 = MP[1];
  // 2-D UNPACKED parameter array
  localparam int UA [0:1][0:1] = '{'{1, 2}, '{3, 4}};
  // string parameters: text concatenation
  localparam string NAME = "widget";
  localparam string PFX  = "pre_";
  localparam string CAT  = {PFX, NAME};
  // real → int in constant context ($rtoi truncates)
  localparam real PI = 3.14159;
  localparam int  PI_I = $rtoi(PI * 100.0);
  int mp0, mp1, ua01, ua10, catlen, pii;
  initial begin
    mp0 = MP0; mp1 = MP1;
    ua01 = UA[0][1]; ua10 = UA[1][0];
    catlen = CAT.len();
    pii = PI_I;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "mp0"), 0xBB, "packed multi-dim element select");
    assert_eq!(u(&sim, "mp1"), 0xAA);
    assert_eq!(u(&sim, "ua01"), 2, "2-D unpacked parameter array");
    assert_eq!(u(&sim, "ua10"), 3);
    assert_eq!(
        u(&sim, "catlen"),
        10,
        "string parameter concat is text (pre_widget)"
    );
    assert_eq!(u(&sim, "pii"), 314, "$rtoi in constant context");
}
