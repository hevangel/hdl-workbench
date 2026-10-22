//! §5.8: "The time literal is interpreted as a realtime value scaled to the
//! current time unit and rounded to the current time precision." There is no
//! exemption for constant expressions, and §6.20.2 does not carve one out for
//! parameter declarations.
//!
//! The scope-time pass applied that scaling to STATEMENTS only, so procedural
//! code was correct — `#100ns`, `x = 100ns`, an automatic-variable initialiser
//! — while a DECLARATION initialiser kept its `NumberLiteral::Time` and fell
//! through to the fixed-1ns fallback in expression evaluation (`secs * 1e9`).
//! It was therefore scaled to nanoseconds no matter what the module declared:
//!
//! ```text
//!                                 `timescale 1fs/1fs, both meaning 100 ns
//!   #100ns                                   100000000   correct
//!   localparam realtime T = 100ns                  100   1e6 short
//! ```
//!
//! Two things kept this quiet. The error is the ratio `1ns / timeunit`, so a
//! module at the default 1 ns unit was ACCIDENTALLY correct — the bug only
//! appeared once someone chose a fine timescale on purpose. And `100ns / 1ns`
//! is `100`, the same number a "drop the suffix" bug would produce, so the
//! obvious test case cannot tell the two mechanisms apart. `300ps` and `1us`
//! can, and both are pinned below.
//!
//! Worse than a scale factor: a sub-nanosecond literal did not shrink, it
//! VANISHED. Under `1ps/1ps` a `localparam realtime D = 300ps` folded to 0.3,
//! which rounds to zero at the module's precision — a reset-path delay in a
//! PFD written that way disappeared, giving a zero-width reset rather than an
//! obviously wrong one.
//!
//! Cross-checked against Icarus Verilog 14.0, which produces the "want"
//! column below for every case here.

use xezim::simulate;

/// The `NOTE:` lines a run printed, in order.
fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

/// Every elaboration-time form must agree with the run-time forms. Under a
/// 1 fs time unit, `100ns` is 100_000_000 time units in all nine.
#[test]
fn declaration_initialisers_scale_like_procedural_code() {
    let src = r#"
`timescale 1fs/1fs
module top #(parameter realtime P_PORT = 100ns) ();
  localparam realtime P_LOCAL = 100ns;
  const     realtime  C_CONST = 100ns;
  realtime            v_init  = 100ns;   // static (elaboration) initialiser
  realtime            v_proc;
  initial begin
    realtime a_auto = 100ns;             // automatic (run-time) initialiser
    v_proc = 100ns;
    $display("NOTE: port %0.0f",   P_PORT);
    $display("NOTE: local %0.0f",  P_LOCAL);
    $display("NOTE: const %0.0f",  C_CONST);
    $display("NOTE: static %0.0f", v_init);
    $display("NOTE: auto %0.0f",   a_auto);
    $display("NOTE: proc %0.0f",   v_proc);
    $finish;
  end
endmodule
"#;
    assert_eq!(
        notes(src),
        vec![
            "NOTE: port 100000000",
            "NOTE: local 100000000",
            "NOTE: const 100000000",
            "NOTE: static 100000000",
            "NOTE: auto 100000000",
            "NOTE: proc 100000000",
        ]
    );
}

/// A suffix that is NOT nanoseconds separates "scaled to the module's unit"
/// from "suffix discarded, bare number kept" — the two theories that
/// `100ns` alone cannot distinguish. Under a 1 ps unit, `300ps` is 300 and
/// `1us` is 1_000_000; the discard theory would give 300 and 1.
#[test]
fn sub_nanosecond_and_micro_second_literals_scale_by_the_unit() {
    let src = r#"
`timescale 1ps/1ps
module top;
  localparam realtime P300 = 300ps;
  localparam realtime P1U  = 1us;
  initial begin
    $display("NOTE: p300 %0.0f", P300);
    $display("NOTE: p1u %0.0f",  P1U);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: p300 300", "NOTE: p1u 1000000"]);
}

/// The value has to survive being SPENT as a delay, not merely printed. This
/// is the shape that failed in the field: a period constant feeding `#()`.
/// The old fold made this delay 0.000 ps — the delay disappeared entirely.
#[test]
fn a_sub_nanosecond_constant_delay_is_not_rounded_away() {
    let src = r#"
`timescale 1ps/1ps
module top;
  localparam realtime D = 300ps;
  initial begin
    realtime t0;
    t0 = $realtime;
    #(D);
    $display("NOTE: elapsed %0.0f", $realtime - t0);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: elapsed 300"]);
}

/// The same module body at three timeunits. This is the ratio the bug was:
/// `1ns / timeunit`, right at 1 ns and wrong everywhere finer.
#[test]
fn the_same_constant_scales_with_each_modules_own_timeunit() {
    let src = r#"
module m_ns;                       // no directive -> 1ns default
  localparam realtime P = 100ns;
  initial $display("NOTE: ns %0.0f", P);
endmodule
`timescale 1ps/1ps
module m_ps;
  localparam realtime P = 100ns;
  initial $display("NOTE: ps %0.0f", P);
endmodule
`timescale 1fs/1fs
module top;
  localparam realtime P = 100ns;
  m_ns u_ns();
  m_ps u_ps();
  initial begin
    $display("NOTE: fs %0.0f", P);
    $finish;
  end
endmodule
"#;
    let got = notes(src);
    assert!(got.contains(&"NOTE: ns 100".to_string()), "got {:?}", got);
    assert!(
        got.contains(&"NOTE: ps 100000".to_string()),
        "got {:?}",
        got
    );
    assert!(
        got.contains(&"NOTE: fs 100000000".to_string()),
        "got {:?}",
        got
    );
}

/// A design already at the default 1 ns unit was accidentally correct before
/// the fix and must stay exactly as it was — this is the case that made the
/// bug invisible, and it is the one most existing designs are in.
#[test]
fn identity_at_the_default_nanosecond_timeunit() {
    let src = r#"
`timescale 1ns/1ns
module top;
  localparam realtime P = 10ns;
  initial begin
    realtime t0;
    t0 = $realtime;
    #(P);
    $display("NOTE: %0.0f %0t", P, $time);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: 10 10"]);
}

/// A parameter OVERRIDE at the instantiation (`m #(.P(50ns))`) and a
/// `defparam` are written in the instantiating module's unit and scale like
/// the declaration default. The reference simulator prints 50000 / 25000 /
/// 100000 here under `1ps/1ps`; before this was covered, the default scaled
/// and the override did not (50), so one design mixed the two.
#[test]
fn instantiation_overrides_and_defparam_scale_like_the_default() {
    let out = notes(
        r#"
`timescale 1ps/1ps
module m #(parameter realtime P = 100ns) ();
  initial $display("NOTE: %m %0.0f", P);
endmodule
module tb;
  m u();
  m #(.P(50ns)) w();
  m d();
  defparam d.P = 25ns;
endmodule
"#,
    );
    assert!(out.iter().any(|m| m == "NOTE: tb.u 100000"), "{out:?}");
    assert!(out.iter().any(|m| m == "NOTE: tb.w 50000"), "{out:?}");
    assert!(out.iter().any(|m| m == "NOTE: tb.d 25000"), "{out:?}");
}
