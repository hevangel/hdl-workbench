//! GitHub issue #68: `@(ev.triggered)` (§15.5.3) must block until the event
//! fires. The sensitivity named a nonexistent "ev.triggered" signal, fell
//! into the no-signal delta-yield fallback and woke immediately at t=0 —
//! breaking every event-based cross-process sync. Reference-validated:
//! wakes exactly at the trigger time.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn at_event_triggered_blocks_until_fire() {
    let src = r#"
module tb;
  event ev;
  int woke_at = -1;
  int woke_plain = -1;
  initial begin
    @(ev.triggered);
    woke_at = $time;
  end
  initial begin
    @(ev);
    woke_plain = $time;
  end
  initial begin
    #10;
    -> ev;
    #1 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "woke_at"),
        10,
        "@(ev.triggered) must block until the trigger"
    );
    assert_eq!(u(&sim, "woke_plain"), 10, "@(ev) unchanged");
}
