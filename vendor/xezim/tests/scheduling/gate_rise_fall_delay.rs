//! §28.11 — a gate declared `#(rise, fall)` must use the SECOND value for
//! transitions to 0.
//!
//! The parser captured only the first delay expression and discarded the rest
//! (its AST comment said so outright: "rise/fall pairs collapse to the
//! first"), so every 1→0 edge used the RISE delay. A `buf #(1, 6)` fell one
//! tick after its input instead of six — silently wrong timing on any
//! gate-level netlist that specifies asymmetric delays, which most do.
//!
//! The gate lowers to a `ContinuousAssignment` carrying a single delay, so the
//! fall value rides on `ElaboratedModule::gate_fall_delays` (keyed by driven
//! net) and is selected in `schedule_delayed_with_delay`, the one place where
//! both the target signal and the incoming value are known.

use xezim::simulate;

/// `a` rises at t=20 and falls at t=40. Each gate's output is sampled by a
/// separate always block that records the time of its own edges.
const SRC: &str = r#"
module tb;
  reg a;
  wire o_rf, o_single, o_buf;
  and #(2, 5) g_rf     (o_rf,     a, 1'b1);   // rise 2, fall 5
  and #(4)    g_single (o_single, a, 1'b1);   // both 4
  buf #(1, 6) g_buf    (o_buf,    a);         // rise 1, fall 6

  int rf_rise, rf_fall, single_rise, single_fall, buf_rise, buf_fall;

  always @(posedge o_rf)     rf_rise     = $time;
  always @(negedge o_rf)     rf_fall     = $time;
  always @(posedge o_single) single_rise = $time;
  always @(negedge o_single) single_fall = $time;
  always @(posedge o_buf)    buf_rise    = $time;
  always @(negedge o_buf)    buf_fall    = $time;

  initial begin
    a = 0;
    #20 a = 1;
    #20 a = 0;
    #20 $finish;
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The rise edge was always correct — asserted so a regression that swapped
/// the two values would still be caught.
#[test]
fn gate_rise_delay_uses_the_first_value() {
    let sim = simulate(SRC, 200).expect("simulate failed");
    assert_eq!(u(&sim, "rf_rise"), 22, "#(2,5) rises 2 after the input");
    assert_eq!(u(&sim, "single_rise"), 24, "#(4) rises 4 after the input");
    assert_eq!(u(&sim, "buf_rise"), 21, "#(1,6) rises 1 after the input");
}

#[test]
fn gate_fall_delay_uses_the_second_value() {
    let sim = simulate(SRC, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "rf_fall"),
        45,
        "#(2,5) must fall 5 after the input, not 2"
    );
    assert_eq!(
        u(&sim, "buf_fall"),
        46,
        "#(1,6) must fall 6 after the input, not 1"
    );
}

/// A single-delay spec still governs BOTH edges — the fall path must not
/// disturb the common form.
#[test]
fn single_delay_still_governs_both_edges() {
    let sim = simulate(SRC, 200).expect("simulate failed");
    assert_eq!(u(&sim, "single_rise"), 24, "#(4) rise");
    assert_eq!(u(&sim, "single_fall"), 44, "#(4) fall uses the same value");
}
