//! class-perf: ENUM-typed formals on the compiled-method admission surface.
//!
//! The interpreter's TypeReference binding branch never width-adapts an enum
//! formal (`uvm_severity`-style typedef: the actual keeps the caller's width,
//! only signedness may be stamped) — the compiled path now mirrors that
//! exactly: the formal seeds from the frame-bound value at width 0 (keep
//! source). Covers read/compare/write/pass-through/omitted-default of an enum
//! formal, a narrow (`bit [3:0]`) enum base, an enum return, case-on-enum,
//! and an enum formal mixed with a string formal (the report-filter shape).
//!
//! Expectations are reference-simulator-validated (the distilled shape and
//! its reference run print identical values). The gate is forced ON at eager
//! tier so a plain `cargo test` exercises the compiled path.
use xezim::simulate;

fn gate_on() -> bool {
    super::compiled_method_test_env::eager()
}

fn n(sim: &xezim::compiler::Simulator, name: &str) -> i64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("top.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_i64()
        .expect("integral value")
}

#[test]
fn enum_formals_end_to_end() {
    if !gate_on() {
        return;
    }
    let src = r#"
typedef enum { EC_A = 1, EC_B = 5, EC_C = 9 } ec_t;
typedef enum bit [3:0] { EN_A = 4'h2, EN_B = 4'h9 } en_t;
class C;
   function int get_v(ec_t e);
      if (e == EC_B) return 10;
      if (e == EC_C) return 20;
      return 100 + int'(e);
   endfunction
   function int set_v(ec_t e);
      e = EC_C;
      return int'(e);
   endfunction
   function en_t pick(en_t a, int sel);
      if (sel) return a; else return EN_A;
   endfunction
   function int use_pick(int sel);
      en_t r = pick(EN_B, sel);
      return (r == EN_B) ? 7 : 3;
   endfunction
   function bit enabled(int verbosity, ec_t severity, string id);
      if (severity == EC_A && id == "x") return 1;
      return verbosity > 3;
   endfunction
   function int def(ec_t e = EC_B);
      return int'(e);
   endfunction
   function int cases(ec_t e);
      case (e)
        EC_A: return 1;
        EC_B: return 2;
        default: return 0;
      endcase
   endfunction
endclass
module top;
  int r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13;
  initial begin
    C c;
    en_t nv;
    c = new;
    nv = EN_B;
    r1 = c.get_v(EC_A);
    r2 = c.get_v(EC_B);
    r3 = c.get_v(EC_C);
    r4 = c.set_v(EC_A);
    r5 = c.use_pick(1);
    r6 = c.use_pick(0);
    r7 = c.enabled(5, EC_A, "x");
    r8 = c.enabled(2, EC_B, "y");
    r9 = c.def();
    r10 = c.def(EC_A);
    r11 = c.cases(EC_B);
    r12 = c.cases(EC_C);
    r13 = (c.pick(nv, 1) == EN_B) ? 1 : 0;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    assert_eq!(n(&sim, "r1"), 101, "enum formal read + arithmetic");
    assert_eq!(n(&sim, "r2"), 10, "enum formal equality-compare");
    assert_eq!(n(&sim, "r3"), 20, "second enum label");
    assert_eq!(n(&sim, "r4"), 9, "write to enum formal");
    assert_eq!(n(&sim, "r5"), 7, "narrow enum local + pass-through");
    assert_eq!(n(&sim, "r6"), 3, "narrow enum default return");
    assert_eq!(n(&sim, "r7"), 1, "enum+string formal match");
    assert_eq!(n(&sim, "r8"), 0, "enum+string formal mismatch");
    assert_eq!(n(&sim, "r9"), 5, "omitted enum default");
    assert_eq!(n(&sim, "r10"), 1, "explicit enum arg");
    assert_eq!(n(&sim, "r11"), 2, "case on enum formal");
    assert_eq!(n(&sim, "r12"), 0, "case default arm");
    assert_eq!(n(&sim, "r13"), 1, "narrow (4-bit) enum base round-trip");
}
