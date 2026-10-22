//! §9.4.5 — intra-assignment EVENT control on a NONBLOCKING assignment
//! (`lhs <= @(posedge clk) rhs;` and the `repeat (n) @(...)` form).
//!
//! The parser discards intra-assignment timing; a pre-parse text pass
//! canonicalizes it into marker calls the simulator implements. Only the
//! BLOCKING arm handled `$__xz_intra_ev`, so on an NBA the marker fell
//! through to plain expression eval — an unknown system call, i.e. ZERO —
//! and the NBA posted 0 immediately. Doubly wrong: the variable was
//! CLOBBERED with 0 in the same timestep, and the real value never arrived.
//!
//! §9.4.5 semantics: the RHS is captured at the assignment, the process is
//! NOT blocked, and the update lands when the event has fired n times.
//!
//! All expectations byte-identical to a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The core shape: value unchanged until the edge, updated after, and the
/// issuing process does not block.
#[test]
fn nba_event_control_updates_at_the_edge() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] c;
  int in_flight, post_edge, not_blocked_t;
  initial begin
    c = 8'h55;
    #1;
    c <= @(posedge clk) 8'h66;
    not_blocked_t = $time;        // still t=1: the NBA must not block
    #1 in_flight = c;             // t=2, before the t=5 edge: still 55
    #10 post_edge = c;            // t=12, after the edge: 66
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "not_blocked_t"),
        1,
        "an NBA never blocks the process"
    );
    assert_eq!(
        u(&sim, "in_flight"),
        0x55,
        "unchanged before the edge — not clobbered"
    );
    assert_eq!(u(&sim, "post_edge"), 0x66, "updated at the edge");
}

/// `repeat (2) @(posedge clk)` waits two edges; RHS captured at issue time.
#[test]
fn nba_repeat_event_and_rhs_capture() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] d, src;
  int after_one, after_two;
  initial begin
    src = 8'h77;
    #1;
    d <= repeat (2) @(posedge clk) src;
    src = 8'h99;                   // must NOT affect the captured value
    #10 after_one = d;             // one edge (t=5): not yet
    #10 after_two = d;             // two edges (t=15): landed
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_ne!(u(&sim, "after_one"), 0x77, "not yet after one edge");
    assert_eq!(
        u(&sim, "after_two"),
        0x77,
        "captured RHS lands after two edges"
    );
}

/// An event that never fires: the update never lands, and the variable is
/// NOT zeroed — the silent-clobber half of the bug.
#[test]
fn a_never_firing_event_leaves_the_variable_alone() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic clk = 0;      // never toggles
  logic [7:0] c;
  int mid, later;
  initial begin
    c = 8'h55;
    c <= @(posedge clk) 8'h66;
    #1 mid = c;
    #5 later = c;
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "mid"), 0x55, "no clobber");
    assert_eq!(u(&sim, "later"), 0x55, "no phantom update");
}

/// The guards: plain NBA, `<= #d`, and the BLOCKING event forms all keep
/// their behavior.
#[test]
fn other_assignment_timing_forms_unchanged() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] a, b, c2, c3;
  int t_block_ev, t_block_rep, b_early, b_late;
  initial begin
    a = 8'h11; b = 8'h11;
    #1;
    a <= 8'h22;
    b <= #2 8'h33;
    #1 b_early = b;                // t=2: delay not elapsed
    #4 b_late = b;                 // t=6: landed
    c2 = 8'h11;
    c2 = @(posedge clk) 8'h44;     // blocking: suspends to the edge
    t_block_ev = $time;
    c3 = repeat (2) @(posedge clk) 8'h55;
    t_block_rep = $time;
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 400).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 0x22, "plain NBA");
    assert_eq!(u(&sim, "b_early"), 0x11, "<= #d not yet");
    assert_eq!(u(&sim, "b_late"), 0x33, "<= #d landed");
    assert_eq!(u(&sim, "c2"), 0x44, "blocking event assign");
    assert_eq!(
        u(&sim, "t_block_ev"),
        15,
        "blocking form suspends to the edge"
    );
    assert_eq!(u(&sim, "c3"), 0x55, "blocking repeat form");
    assert_eq!(u(&sim, "t_block_rep"), 35, "two further edges");
}
