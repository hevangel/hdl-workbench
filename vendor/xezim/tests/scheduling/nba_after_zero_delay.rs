//! IEEE 1800-2017 §4.4.2.3 / §4.5 — `#0` and the NBA region.
//!
//! A `#0` suspends the process and reschedules its continuation into the
//! Inactive region of the SAME time slot. Per the LRM §4.5 scheduling
//! algorithm the Inactive region activates BEFORE the NBA region, so an
//! NBA posted before the `#0` is NOT yet visible when the continuation
//! resumes — it commits afterwards, within the same time slot. This is
//! the reference-simulator behavior xezim follows; a differing commercial
//! camp (several commercial simulators) applies the NBA region first and would show the
//! updated value at the `#0` resume.

use xezim::simulate;

fn lookup(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// An NBA posted in the active region is visible NEITHER before the `#0`
/// (same active pass) NOR at the `#0` resume (Inactive precedes NBA,
/// §4.5) — only after the time slot completes.
#[test]
fn nba_visible_after_zero_delay() {
    const SRC: &str = r#"
module tb;
  logic [7:0] nb;
  logic [7:0] before_z = 8'hFF;
  logic [7:0] after_z  = 8'hFF;
  logic [7:0] later    = 8'hFF;
  initial begin
    nb = 8'h00;
    nb <= 8'hAA;
    before_z = nb; // pre-#0: NBA not yet applied -> 00
    #0;
    after_z = nb;  // #0 resume: STILL pre-NBA (inactive before NBA) -> 00
    #1;
    later = nb;    // next time step: committed -> aa
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    let before_z = lookup(&sim, "before_z") & 0xFF;
    let after_z = lookup(&sim, "after_z") & 0xFF;
    let later = lookup(&sim, "later") & 0xFF;
    assert_eq!(
        before_z, 0x00,
        "before #0 the NBA must not have been applied yet (§4.4.2.3), got {:02x}",
        before_z
    );
    assert_eq!(
        after_z, 0x00,
        "at the #0 resume the NBA region has NOT yet run (§4.5, reference-simulator verified), got {:02x}",
        after_z
    );
    assert_eq!(
        later, 0xAA,
        "after the time step the NBA must have committed, got {:02x}",
        later
    );
}

/// The classic NBA swap, observed PAST the time slot: both right-hand
/// sides sampled in the active pass, both commit in the NBA region; the
/// `#0` resume still sees the OLD values (§4.5), the next time step the
/// swapped ones.
#[test]
fn nba_swap_visible_after_zero_delay() {
    const SRC: &str = r#"
module tb;
  int a, b;
  int za = -1, zb = -1;
  int ra = -1, rb = -1;
  initial begin
    a = 1; b = 2;
    a <= b; b <= a; // RHS sampled pre-commit: classic swap
    #0;
    za = a; zb = b; // still pre-NBA: 1, 2
    #1;
    ra = a; // swapped: 2
    rb = b; // swapped: 1
  end
endmodule
"#;
    let sim = simulate(SRC, 100).expect("simulate failed");
    let za = lookup(&sim, "za") & 0xFFFFFFFF;
    let zb = lookup(&sim, "zb") & 0xFFFFFFFF;
    let ra = lookup(&sim, "ra") & 0xFFFFFFFF;
    let rb = lookup(&sim, "rb") & 0xFFFFFFFF;
    assert_eq!(za, 1, "at the #0 resume a still holds 1 (§4.5), got {}", za);
    assert_eq!(zb, 2, "at the #0 resume b still holds 2 (§4.5), got {}", zb);
    assert_eq!(
        ra, 2,
        "after the time step a must hold the swapped value 2, got {}",
        ra
    );
    assert_eq!(
        rb, 1,
        "after the time step b must hold the swapped value 1, got {}",
        rb
    );
}
