//! §7.12.3 — array reduction methods produce a result of the ELEMENT type:
//! its width AND its signedness, with accumulation wrapping at that width.
//! Reference-validated.
//!
//! Three defects in the no-`with` reduction path:
//!  * `sum`/`product` accumulated `to_u64` into a plain 32-bit unsigned
//!    result — `byte e[4] = '{8'h12, 8'hFE, 8'h07, 8'hA0}; e.sum()` read
//!    439 where the element-typed answer is -73 (0xB7 as signed 8-bit).
//!  * the accumulator never wrapped at the element width, so a signed-byte
//!    product read 1800 instead of 8 (1800 mod 256).
//!  * `min`/`max` always compared unsigned, picking 3 as the "min" of
//!    `'{-5, 3, -120}`.

use xezim::simulate;

fn i(sim: &xezim::compiler::Simulator, n: &str) -> i64 {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n));
    let raw = v.to_u64().unwrap_or_else(|| panic!("{} is x/z", n));
    raw as u32 as i32 as i64
}

const SRC: &str = r#"
module tb;
  byte elems[4] = '{8'h12, 8'hFE, 8'h07, 8'hA0};
  byte eq[$] = '{-8'sd5, 8'sd3, -8'sd120};
  int unsigned ue[3] = '{32'd10, 32'hFFFF_FFF0, 32'd6};
  int r_sum, r_prod, r_min, r_max, r_usum, r_qsum;
  byte mq[$];
  initial begin
    r_sum  = elems.sum();
    r_prod = eq.product();
    mq = eq.min(); r_min = mq[0];
    mq = eq.max(); r_max = mq[0];
    r_usum = ue.sum();
    r_qsum = eq.sum();
  end
endmodule
"#;

#[test]
fn array_reductions_are_element_typed() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    // Signed byte elements: the sum is an 8-bit signed value, sign-extended
    // into the 32-bit destination.
    assert_eq!(i(&sim, "r_sum"), -73, "byte fixed-array sum");
    // -5 * 3 * -120 = 1800; wrapped at 8 bits = 8.
    assert_eq!(
        i(&sim, "r_prod"),
        8,
        "byte queue product wraps at element width"
    );
    assert_eq!(i(&sim, "r_min"), -120, "signed min comparison");
    assert_eq!(i(&sim, "r_max"), 3, "signed max comparison");
    // Unsigned 32-bit elements: 10 + 0xFFFF_FFF0 + 6 wraps to 0.
    assert_eq!(i(&sim, "r_usum"), 0, "unsigned int sum wraps at 32 bits");
    assert_eq!(i(&sim, "r_qsum"), -122, "byte queue sum");
}
