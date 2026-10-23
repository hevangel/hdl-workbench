//! §5.7.1 / §5.9.1 / §6.3.1 / §6.16 / §6.19 — lexical + string + literal
//! semantics. Reference-validated (agent chapter audit, 2026-08-09).
//!
//! Seven defects fixed together:
//!  * `"\xf1"` — the escape decoder routed decoded bytes through a lossy
//!    UTF-8 conversion, so every byte >= 0x80 collapsed to U+FFFD and read
//!    back as 0xBD. Strings now carry one Latin-1 char per byte end-to-end
//!    (decoder ↔ `Value::from_string` ↔ stdout sink).
//!  * `u64 = 'bx` — an unsized all-x/all-z based literal materialized as 32
//!    bits and zero-extended above; it must FILL the consuming context
//!    (64 x bits), like an unbased-unsized literal.
//!  * `bit b = 1'bx;` — the declaration-initializer path skipped the
//!    4-state→2-state conversion, storing an "impossible" x in a 2-state var.
//!  * bare `enum {E0,E1} k;` — default base type is `int` (2-STATE), so the
//!    default initial value is 0, not x.
//!  * `.atoi()` — trimmed whitespace C-style ("  512" read 512, must be 0),
//!    dropped `_` separators ("12_34" read 12, must be 1234).
//!  * `string'(24'hab0063)` — kept interior NUL bytes (len 3, must be 2).
//!  * `{<<8{qi}}` assigned to a queue — elements were written to the slow-path
//!    signals map while reads resolved the compact table's zeros.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

const SRC: &str = r#"
module tb;
  byte esc_hi, esc_oct;
  logic [63:0] u64x;
  logic ok_u64x;
  bit bx_init = 1'bx;
  typedef enum {E0, E1, E2} bare_e;
  bare_e k;
  int enum_def;
  string t;
  int atoi_ws, atoi_us, atoi_neg, cast_len;
  byte q[$];
  int qi[$];
  int q0, q3;
  initial begin
    esc_hi  = "\xf1";
    esc_oct = "\377";
    u64x = 'bx;
    ok_u64x = (u64x === 64'hxxxx_xxxx_xxxx_xxxx);
    enum_def = k;
    t = "  512cats"; atoi_ws  = t.atoi();
    t = "12_34";     atoi_us  = t.atoi();
    t = "-873";      atoi_neg = t.atoi();
    t = string'(24'hab0063); cast_len = t.len();
    qi = '{32'hAABBCCDD};
    q = {<<8{qi}};
    q0 = q[0]; q3 = q[3];
  end
endmodule
"#;

#[test]
fn lexical_string_and_literal_semantics() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(u(&sim, "esc_hi"), 0xf1, "\\xf1 escape byte");
    assert_eq!(u(&sim, "esc_oct"), 0xff, "\\377 escape byte");
    assert_eq!(u(&sim, "ok_u64x"), 1, "'bx fills 64-bit context with x");
    assert_eq!(u(&sim, "bx_init"), 0, "bit decl-init converts x to 0");
    assert_eq!(u(&sim, "enum_def"), 0, "bare enum default-initializes to 0");
    assert_eq!(u(&sim, "atoi_ws"), 0, "atoi does not skip whitespace");
    assert_eq!(u(&sim, "atoi_us"), 1234, "atoi accepts underscores");
    assert_eq!(
        u(&sim, "atoi_neg") as u32 as i32,
        -873,
        "atoi signed result"
    );
    assert_eq!(u(&sim, "cast_len"), 2, "string cast strips NUL bytes");
    // {<<8{32'hAABBCCDD}} = DD CC BB AA, element 0 first; the signed byte
    // elements sign-extend into the int destinations (0xDD = -35).
    assert_eq!(u(&sim, "q0") as u32 as i32, -35, "streamed queue element 0");
    assert_eq!(u(&sim, "q3") as u32 as i32, -86, "streamed queue element 3");
}
