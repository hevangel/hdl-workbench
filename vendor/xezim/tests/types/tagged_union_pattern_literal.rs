//! §7.3.1 — tagged union pattern literal `tagged member '{ ... }`.
//!
//! The parser previously accepted only `tagged member` and
//! `tagged member(expr)`; a pattern literal (`tagged '{b: 8'h5a}`) failed to
//! parse. The pattern's assignment-pattern content is now parsed through the
//! shared item loop and evaluated into the union's member storage.

use xezim::simulate;

const SRC: &str = r#"
module tb;
  typedef union tagged packed {
    logic [7:0]  b;
    logic [15:0] h;
  } u_t;

  u_t u_byte;
  u_t u_half;
  u_t u_again;

  logic [7:0]  byte_val = 8'h0;
  logic [15:0] half_val = 16'h0;
  logic [15:0] again_h  = 16'h0;

  initial begin
    u_byte = tagged '{b: 8'h5a};
    u_half = tagged '{h: 16'h1234};
    byte_val = u_byte.b;
    half_val = u_half.h;
    // Re-tagging the same variable selects the new member: the tag must be
    // extracted from each pattern, not cached from the first assignment.
    u_again = tagged '{b: 8'h01};
    u_again = tagged '{h: 16'hABCD};
    again_h = u_again.h;
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

#[test]
fn tagged_union_pattern_literal_selects_member() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "byte_val"), 0x5a, "tagged pattern b member value");
    assert_eq!(u(&sim, "half_val"), 0x1234, "tagged pattern h member value");
    assert_eq!(
        u(&sim, "again_h"),
        0xABCD,
        "re-tagged pattern selects new member"
    );
}
