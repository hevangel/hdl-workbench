//! §9.4.2 — an event control may name an ARBITRARY expression, and it must
//! keep firing on every change. Reference-validated.
//!
//! Two defects:
//!  * `always @(a + 1)` — the sensitivity collector had no arm for Binary
//!    (or Unary/Conditional) expressions, so the block armed on an EMPTY
//!    sensitivity: it ran once at t=0 and then froze forever.
//!  * `always @(arr[1])` on an UNPACKED array — the collector walked past
//!    the Index to the base name, which is not a signal for an unpacked
//!    array, so the block never fired at all.

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
  int a = 0;
  int arr [3];
  int n_add = 0, n_idx = 0;
  always @(a + 1) n_add = n_add + 1;
  always @(arr[1]) n_idx = n_idx + 1;
  initial begin
    #1 a = 1; arr[1] = 1;
    #1 a = 2; arr[1] = 2;
    #1 a = 3; arr[1] = 3;
    #1;
  end
endmodule
"#;

#[test]
fn expression_event_controls_keep_firing() {
    let sim = simulate(SRC, 20).expect("simulate failed");
    assert_eq!(u(&sim, "n_add"), 3, "@(a+1) fires on every change of a");
    assert_eq!(u(&sim, "n_idx"), 3, "@(arr[1]) fires on element changes");
}
