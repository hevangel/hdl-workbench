//! §11.4.8: `0 & x` is 0 for every value the other operand can take — X and Z
//! included — so a masked-off function result cannot affect the expression.
//!
//! Gating a function result with a reset mask (`{N{rst_n}} & f(..)`) is
//! ordinary RTL, and xezim evaluated `f` anyway: on a 200k-cycle benchmark the
//! call cost the same during active reset as out of it, because a bitwise `&`
//! (unlike `&&`) does evaluate both operands per the LRM. A ternary
//! (`rst_n ? f(..) : '0`) already short-circuited; the mask form did not.
//!
//! Skipping the call is only legal when it cannot be observed, so it is gated
//! on a conservative purity walk: `ref`/`output`/`inout` formals, writes to
//! anything not declared in the function, tasks, system calls, timing and
//! recursion all make it impure, as does any construct the walker does not
//! positively recognise.
//!
//! The impure case below is the one that matters — a wrong answer there is a
//! silently skipped side effect, which is far worse than a slow simulation.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
module top;
  int calls = 0;
  logic [9:0] o_pure, o_impure, o_unmasked, o_x;
  logic mask = 1'b0;
  logic xm;

  function automatic [9:0] pf(input [4:0] i);
    pf = i + 10'd7;
  endfunction

  // Writes a module-scope variable: eliding this is observable.
  function automatic [9:0] imf(input [4:0] i);
    calls = calls + 1;
    imf = i + 10'd7;
  endfunction

  assign o_pure     = {10{mask}} & pf(5'd3);
  // Same reasoning mirrored: the mask may sit on either side, and `1 | x` is
  // `1` exactly as `0 & x` is `0`. A right-hand mask is only safe to evaluate
  // first because the call it guards is pure.
  logic [9:0] o_and_r, o_or_l, o_or_r;
  assign o_and_r    = pf(5'd3) & {10{mask}};
  assign o_or_l     = {10{~mask}} | pf(5'd3);
  assign o_or_r     = pf(5'd3) | {10{~mask}};
  assign o_impure   = {10{mask}} & imf(5'd3);
  assign o_unmasked = {10{1'b1}} & pf(5'd3);
  // An X mask is NOT all-zero, so nothing may be skipped: 'x & 10 is not 0.
  assign o_x        = {10{xm}} & pf(5'd3);

  initial begin
    xm = 1'bx;
    #1;
    $display("NOTE: pure=%0d", o_pure);
    $display("NOTE: impure=%0d", o_impure);
    $display("NOTE: unmasked=%0d", o_unmasked);
    $display("NOTE: xmask=%b", o_x);
    $display("NOTE: and_r=%0d", o_and_r);
    $display("NOTE: or_l=%0d", o_or_l);
    $display("NOTE: or_r=%0d", o_or_r);
    $display("NOTE: called=%0d", calls > 0);
    #1 $finish;
  end
endmodule
"#;

/// A masked-off PURE call may be skipped; a masked-off IMPURE one may not, and
/// an X mask disables the optimization entirely.
#[test]
fn zero_mask_elides_only_pure_calls() {
    let got = notes(SRC);
    assert!(got.contains(&"NOTE: pure=0".to_string()), "{got:?}");
    assert!(got.contains(&"NOTE: impure=0".to_string()), "{got:?}");
    assert!(
        got.contains(&"NOTE: unmasked=10".to_string()),
        "an unmasked call must still be evaluated: {got:?}"
    );
    assert!(
        got.contains(&"NOTE: called=1".to_string()),
        "the impure function MUST still run; eliding it drops a side effect: {got:?}"
    );
    // The mask may sit on either operand, and OR absorbs with all-ones.
    assert!(got.contains(&"NOTE: and_r=0".to_string()), "{got:?}");
    assert!(got.contains(&"NOTE: or_l=1023".to_string()), "{got:?}");
    assert!(got.contains(&"NOTE: or_r=1023".to_string()), "{got:?}");
    assert!(
        got.iter()
            .any(|l| l.starts_with("NOTE: xmask=") && l.contains('x')),
        "an x mask is not all-zero, so the result must carry x: {got:?}"
    );
}
