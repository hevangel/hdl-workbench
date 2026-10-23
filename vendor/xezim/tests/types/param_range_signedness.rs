//! Parameter/localparam value semantics for an EXPLICIT type or range
//! (IEEE 1800 §6.20.2 / §10.7 / §23.10.2): the initializer, and every
//! override, is an ASSIGNMENT to the declared type: evaluated in the
//! declared width context, wrapped to that width, with the DECLARED
//! signedness.
//!
//! * A bare range `[1:0]` is UNSIGNED: `localparam [1:0] C = 1 + P` with
//!   P = 1 is 2 wherever it is used (an assign, a generate-if or ternary
//!   condition, `C + C`, `logic [C-1:0]`, a child port).
//! * `parameter [3:0] P = -1` is 15, `[7:0] P = 256` is 0, and `signed
//!   [3:0]` turns 15 into -1, for defaults and for named and positional
//!   overrides.
//! * A typedef'd SIGNED type keeps its sign, also when the typedef is
//!   local to the module.
//! * An override is evaluated in the formal's width (`.P(4'hF + 4'h1)` on
//!   `[7:0]` is 16), and a formal's range uses the instance's own
//!   parameters (`#(.W(16), .P(16'hABCD))` on `[W-1:0] P`).
//! * A value parameter typed by a type parameter takes that type's full
//!   width (64 bits by default, 48 when overridden with `logic [47:0]`).
//! * defparam values, unpacked-array elements, `signed` with no range,
//!   2-state types and real initializers convert the same way.
//! * An UNTYPED parameter keeps the RHS value and sign (§6.20.2), pinned by
//!   [control] checks along with signed wrap, typedef'd unsigned, `int`,
//!   real, shift/replication amounts, and the width context of
//!   `localparam [7:0] C = P + P` (30, not the 8-bit wrap).
//!
//! The .sv is self-checking: it prints `TEST_PASS`, or `FAIL @…` lines
//! and `TEST_FAIL count=N`. Expected values match the reference simulator.

use xezim::simulate;

const PARAM_RANGE_SIGNEDNESS: &str = include_str!("../lrm_9_value_param/param_range_signedness.sv");

#[test]
fn param_range_signedness_value_semantics() {
    let sim = simulate(PARAM_RANGE_SIGNEDNESS, 1000).expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("TEST_PASS")),
        "expected TEST_PASS in output\nfull output: {msgs:?}"
    );
    assert!(
        !msgs.iter().any(|m| m.contains("FAIL")),
        "unexpected FAIL in output\nfull output: {msgs:?}"
    );
}
