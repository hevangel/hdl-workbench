//! Concurrent-assertion semantics reported as issues #176–#183, one
//! reproduction each, plus a mixed testbench exercising property `or`/`and`,
//! `##[m:n]` consequents, `not` of a matching sequence, `$past(x, 2)` with
//! too little history and `disable iff` on a multi-cycle antecedent; then
//! the sibling shapes: repetition, throughout/within/intersect, first_match,
//! negedge and `iff` clocks, sequence `or`/`and` in antecedents, `if`/`else`,
//! sampled-value functions, named properties with formals, default clocking,
//! cover of a sequence and strong obligations at the end of simulation. Every
//! expected FAIL/COVER line (and every absence of one) is the reference
//! simulator's; the timestamps are the clock ticks the reference reports.
use std::path::PathBuf;
use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sva_semantics_reference");
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join(format!("{name}.sv"));
    std::fs::write(&sv, src).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "-s", "t", "--no-cache", sv.to_str().unwrap()])
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "run failed:\n{text}");
    text
}

/// The FAIL lines the run printed, sorted: the order in which two
/// different assertions report at the same clock tick is not specified.
fn fails(text: &str) -> Vec<String> {
    let mut v: Vec<String> = text
        .lines()
        .filter(|l| l.starts_with("FAIL"))
        .map(|l| l.trim().to_string())
        .collect();
    v.sort();
    v
}

/// FAIL and COVER lines, sorted.
fn reports(text: &str) -> Vec<String> {
    let mut v: Vec<String> = text
        .lines()
        .filter(|l| l.starts_with("FAIL") || l.starts_with("COVER"))
        .map(|l| l.trim().to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn past_in_nonoverlap_consequent_reads_previous_cycle() {
    let text = run(
        "past_in_nonoverlap_consequent_reads_previous_cycle",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0; logic [1:0] c = 0;
  p: assert property (@(posedge clk) a |=> c == $past(c)) else $display("FAIL %0d", $time);
  initial begin #1 a = 1; c = 1; #10 c = 2; #30 $finish; end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    assert_eq!(
        fails(&text),
        vec!["FAIL 15"] as Vec<&str>,
        "FAIL lines:\n{text}"
    );
}

/// §16.8: a sequence after a cycle delay may begin with a unary expression.
#[test]
fn delayed_unary_sequence_is_evaluated() {
    let text = run(
        "delayed_unary_sequence_is_evaluated",
        r#"
module t; logic phase = 0; always #5 phase = ~phase;
  logic launch = 0, guard = 1;
  p: assert property (@(posedge phase) launch |-> ##1 !guard)
     else $display("FAIL unary %0d", $time);
  initial begin #1 launch = 1; #10 launch = 0; #30 $finish; end
endmodule
"#,
    );
    assert_eq!(
        fails(&text),
        vec!["FAIL unary 15"] as Vec<&str>,
        "FAIL lines:\n{text}"
    );
}

/// §16.9.3: one signal is sampled once per property clock even when several
/// sampled-value calls in the same property reference it.
#[test]
fn repeated_sampled_value_calls_share_one_history_step() {
    let text = run(
        "repeated_sampled_value_calls_share_one_history_step",
        r#"
module t;
  logic phase = 0; always #5 phase = ~phase;
  logic serial_level, clear;
  integer step;
  localparam logic [0:11] LEVELS = 12'b110001000000;
  localparam logic [0:11] CLEARS = 12'b110000000000;
  p: assert property (@(posedge phase) disable iff (clear)
       (!serial_level) && $past(serial_level, 1) |->
       (##1 (!serial_level)) ##1
       (1'b1 ##1 (serial_level == $past(serial_level, 1)))[*2])
     else $display("FAIL sampled %0d", $time);
  initial begin
    for (step = 0; step < 12; step++) begin
      serial_level = LEVELS[step]; clear = CLEARS[step];
      @(posedge phase); #1;
    end
    $finish;
  end
endmodule
"#,
    );
    assert_eq!(
        fails(&text),
        vec!["FAIL sampled 55"] as Vec<&str>,
        "FAIL lines:\n{text}"
    );
}

#[test]
fn multi_cycle_antecedent_triggers_on_its_last_cycle() {
    let text = run(
        "multi_cycle_antecedent_triggers_on_its_last_cycle",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0;
  p: assert property (@(posedge clk) a ##1 b |-> c) else $display("FAIL %0d", $time);
  initial begin #1 a = 1; #10 a = 0; b = 1; #30 $finish; end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    assert_eq!(
        fails(&text),
        vec!["FAIL 15"] as Vec<&str>,
        "FAIL lines:\n{text}"
    );
}

#[test]
fn delay_range_consequent_waits_for_the_whole_window() {
    let text = run(
        "delay_range_consequent_waits_for_the_whole_window",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0;
  p: assert property (@(posedge clk) a |-> ##[1:2] b) else $display("FAIL %0d", $time);
  initial begin #1 a = 1; #10 a = 0; #10 b = 1; #30 $finish; end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    assert_eq!(fails(&text), vec![] as Vec<&str>, "FAIL lines:\n{text}");
}

#[test]
fn not_of_a_sequence_fails_when_its_weak_operand_is_unfinished() {
    let text = run(
        "not_of_a_sequence_fails_only_on_a_match",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0;
  p: assert property (@(posedge clk) not (a ##1 b)) else $display("FAIL %0d", $time);
  initial begin #1 a = 1; #40 $finish; end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    assert_eq!(fails(&text), vec!["FAIL 41"], "FAIL lines:\n{text}");
}

#[test]
fn disable_iff_cancels_an_attempt_in_flight() {
    let text = run(
        "disable_iff_cancels_an_attempt_in_flight",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, r = 0;
  p: assert property (@(posedge clk) disable iff (r) a |=> b) else $display("FAIL %0d", $time);
  initial begin
    @(posedge clk); #1 a = 1;
    @(posedge clk); #1 a = 0; r = 1;
    @(posedge clk); #1 r = 0;
    repeat (3) @(posedge clk); $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    assert_eq!(fails(&text), vec![] as Vec<&str>, "FAIL lines:\n{text}");
}

#[test]
fn property_and_or_parse_and_run() {
    let text = run(
        "property_and_or_parse_and_run",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0;
  p1: assert property (@(posedge clk) a |-> b or c);
  p2: assert property (@(posedge clk) (a |-> b) and (c |-> a));
  initial #20 $finish;
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    assert_eq!(fails(&text), vec![] as Vec<&str>, "FAIL lines:\n{text}");
}

#[test]
fn parenthesised_consequent_matches_the_bare_form() {
    let text = run(
        "parenthesised_consequent_matches_the_bare_form",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0;
  p1: assert property (@(posedge clk) a |-> ##2 b) else $display("FAIL p1 %0d", $time);
  p2: assert property (@(posedge clk) (a) |-> (##2 (b))) else $display("FAIL p2 %0d", $time);
  initial begin #1 a = 1; #10 a = 0; #10 b = 1; #30 $finish; end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    assert_eq!(fails(&text), vec![] as Vec<&str>, "FAIL lines:\n{text}");
}

#[test]
fn mixed_property_shapes_match_the_reference() {
    let text = run(
        "mixed",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0, d = 0, e = 0, r = 0; logic [1:0] v = 0;
  // property or: a=1 at 5 with b=0,c=1 holds; at 15 with b=0,c=0 fails
  p_or:  assert property (@(posedge clk) a |-> b or c) else $display("FAIL or %0d", $time);
  // property and of two implications: d=0 while a,c=1 fails at 5
  p_and: assert property (@(posedge clk) (a |-> b or c) and (c |-> d)) else $display("FAIL and %0d", $time);
  // ranged consequent: e rises 2 cycles after a's first pulse -> ok; never after a's second -> fail when window closes
  p_rng: assert property (@(posedge clk) a |=> ##[1:2] e) else $display("FAIL rng %0d", $time);
  // not of a sequence that DOES match: a at 25 then b at 35
  p_not: assert property (@(posedge clk) not (a ##1 b)) else $display("FAIL not %0d", $time);
  // $past depth 2 in a consequent
  p_p2:  assert property (@(posedge clk) a |=> v == $past(v, 2)) else $display("FAIL p2 %0d v=%0d", $time, v);
  // disable iff kills a multi-cycle antecedent in flight
  p_dis: assert property (@(posedge clk) disable iff (r) a ##1 b |-> d) else $display("FAIL dis %0d", $time);
  initial begin
    #1 a = 1; c = 1; v = 1;          // sampled at 5
    #10 c = 0; v = 2;                // 15: a=1 b=0 c=0
    #10 a = 0; e = 1; v = 3;         // 25: e at 25 -> covers |=> ##[1:2] from attempt at 5 (window 15..25)
    #10 a = 1; e = 0; b = 0;         // 35: a=1
    #10 a = 0; b = 1; r = 1;         // 45: b=1 (matches a ##1 b) but r=1 disables p_dis
    #10 b = 0; r = 0;
    #40 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec![
        "FAIL and 5",
        "FAIL or 15",
        "FAIL p2 15 v=2",
        "FAIL and 15",
        "FAIL p2 25 v=3",
        "FAIL or 35",
        "FAIL and 35",
        "FAIL rng 45",
        "FAIL not 45",
        "FAIL rng 65",
    ];
    expected.sort();
    assert_eq!(fails(&text), expected, "FAIL lines:\n{text}");
}

#[test]
fn consecutive_repetition_in_antecedent_and_consequent() {
    let text = run(
        "s1_repeat",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0;
  p1: assert property (@(posedge clk) a |=> b [*2]) else $display("FAIL rep2 %0d", $time);
  p2: assert property (@(posedge clk) a |=> b [*1:2] ##1 c) else $display("FAIL rep12 %0d", $time);
  p3: assert property (@(posedge clk) a [*2] |-> c) else $display("FAIL antrep %0d", $time);
  initial begin
    #1 a = 1;            // 5: a
    #10 a = 0; b = 1;    // 15: b
    #10 b = 1;           // 25: b
    #10 b = 0; c = 1;    // 35: c
    #10 c = 0; a = 1;    // 45: a
    #10 a = 1;           // 55: a (a[*2] matches at 55, c=0 -> antrep fails 55)
    #10 a = 0; b = 1;    // 65: b  (rep2 from 45: b at 65, then b at 75?)
    #10 b = 0;           // 75: b=0 -> rep2 from 45 fails 75; rep12 from 45: b at 65, c at 75? c=0 -> fails 75 or 85
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec![
        "FAIL antrep 55",
        "FAIL rep12 55",
        "FAIL rep12 75",
        "FAIL rep2 55",
        "FAIL rep2 75",
    ];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn throughout_within_and_intersect() {
    let text = run(
        "s2_within",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0, d = 0, e = 0;
  p1: assert property (@(posedge clk) a |-> (e throughout (b ##1 c))) else $display("FAIL thr %0d", $time);
  p2: assert property (@(posedge clk) a |-> ((b ##1 c) within (d ##[1:3] e))) else $display("FAIL within %0d", $time);
  p3: assert property (@(posedge clk) a |-> ((b ##1 c) intersect (d ##1 e))) else $display("FAIL isect %0d", $time);
  initial begin
    #1 a = 1; b = 1; e = 1; d = 1;   // 5
    #10 a = 0; b = 0; c = 1; e = 1;  // 15: c, e stays -> thr ok; isect: d##1 e ok
    #10 c = 0; e = 0; a = 1; b = 1; d = 1; // 25: second attempt, e=0 during -> thr fails at 25 or 35
    #10 a = 0; b = 0; c = 1;         // 35
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec!["FAIL isect 35", "FAIL thr 25", "FAIL within 55"];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn first_match_stops_at_the_earliest_match() {
    let text = run(
        "s3_firstmatch",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0;
  p1: assert property (@(posedge clk) a |-> first_match(##[1:2] b) ##1 c) else $display("FAIL fm %0d", $time);
  initial begin
    #1 a = 1;          // 5
    #10 a = 0; b = 1;  // 15: first match of b at 15 -> c required at 25
    #10 b = 1; c = 0;  // 25: c=0 -> FAIL 25 (first_match forbids the b@25 alternative)
    #10 b = 0; c = 1;  // 35
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec!["FAIL fm 25"];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn negedge_and_iff_clocking_events() {
    let text = run(
        "s4_clocks",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, en = 0;
  p1: assert property (@(negedge clk) a |=> b) else $display("FAIL neg %0d", $time);
  p2: assert property (@(posedge clk iff en) a |=> b) else $display("FAIL iff %0d", $time);
  initial begin
    #6 a = 1;           // negedge 10: a=1 ; posedge 15 en=0
    #10 a = 0; en = 1;  // negedge 20: b=0 -> FAIL neg 20 ; posedge 25 (en=1): a=0
    #5 a = 1;           // posedge 25? a set at 21 -> sampled 25: a=1 -> b at 35 required
    #10 a = 0;          // 35: b=0 -> FAIL iff 35
    #10 en = 0; a = 1;  // 45: en=0, no tick for p2
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec![
        "FAIL iff 35",
        "FAIL neg 20",
        "FAIL neg 40",
        "FAIL neg 60",
        "FAIL neg 70",
    ];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn sequence_or_and_in_the_antecedent() {
    let text = run(
        "s5_seqor",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0, d = 0;
  p1: assert property (@(posedge clk) (a or b) |-> c) else $display("FAIL or1 %0d", $time);
  p2: assert property (@(posedge clk) ((a ##1 b) or c) |-> d) else $display("FAIL or2 %0d", $time);
  p3: assert property (@(posedge clk) (a and b) |-> c) else $display("FAIL and1 %0d", $time);
  initial begin
    #1 b = 1;              // 5: b -> or1 needs c: c=0 -> FAIL or1 5; and1 vacuous
    #10 b = 0; a = 1;      // 15: a -> or1 FAIL 15
    #10 a = 0; b = 1; c = 1; // 25: a##1 b matches -> or2 needs d=0 -> FAIL or2 25; c=1 -> or2 FAIL 25 too (one per match?)
    #10 b = 1; a = 1; c = 1; // 35: a and b -> c=1 ok; or1 ok
    #10 a = 0; b = 0; c = 0;
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec![
        "FAIL or1 15",
        "FAIL or1 5",
        "FAIL or2 25",
        "FAIL or2 25",
        "FAIL or2 35",
    ];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn if_else_nested_implication_not_and_zero_delay() {
    let text = run(
        "s6_ifelse",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, c = 0, s = 0;
  p1: assert property (@(posedge clk) a |-> if (s) b else c) else $display("FAIL if %0d", $time);
  p2: assert property (@(posedge clk) a |-> (b |=> c)) else $display("FAIL nest %0d", $time);
  p3: assert property (@(posedge clk) a |=> not b) else $display("FAIL notb %0d", $time);
  p4: assert property (@(posedge clk) a |-> ##0 b) else $display("FAIL d0 %0d", $time);
  initial begin
    #1 a = 1; s = 1; b = 0; c = 1;   // 5: if(s) b -> b=0 FAIL if 5; nest: b=0 vacuous; d0: b=0 FAIL d0 5
    #10 s = 0; b = 1; c = 0;         // 15: if: c=0 FAIL if 15; nest: b=1 -> c at 25; notb: b=1 -> FAIL notb 15; d0 ok
    #10 c = 0; b = 1;                // 25: nest FAIL 25 (c=0); notb FAIL 25
    #10 a = 0; b = 0; c = 1;         // 35: nest from 25 -> c=1 ok; notb from 25: b=0 ok
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec![
        "FAIL d0 5",
        "FAIL if 15",
        "FAIL if 25",
        "FAIL if 5",
        "FAIL nest 25",
        "FAIL notb 15",
        "FAIL notb 25",
    ];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn sampled_value_functions_in_sequences() {
    let text = run(
        "s7_sampled",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0; logic [1:0] v = 0;
  p1: assert property (@(posedge clk) $rose(a) |-> ##[1:2] $fell(b)) else $display("FAIL rf %0d", $time);
  p2: assert property (@(posedge clk) $changed(v) |-> $past(v) != v) else $display("FAIL chg %0d", $time);
  p3: assert property (@(posedge clk) $stable(v) |-> a) else $display("FAIL stb %0d", $time);
  initial begin
    #1 a = 1; b = 1; v = 1;   // 5: rose(a) -> fell(b) in 15..25; changed(v); stable? no
    #10 v = 1;                // 15: stable(v) -> a=1 ok; b still 1
    #10 b = 0; a = 0; v = 2;  // 25: fell(b) ok; changed
    #10 a = 1; v = 2;         // 35: rose(a) -> fell(b) needed at 45/55; stable -> a=1 ok
    #10 a = 1; b = 0;         // 45: stable v, a=1 ok
    #10 a = 0;                // 55: stable v, a=0 -> FAIL stb 55; fell(b) never -> FAIL rf 55
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec!["FAIL rf 55", "FAIL stb 55", "FAIL stb 65", "FAIL stb 75"];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn named_properties_with_formals_disable_iff_and_default_clocking() {
    let text = run(
        "s8_propdecl",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0, r = 0;
  property p_dis; @(posedge clk) disable iff (r) a |=> b; endproperty
  property p_arg(x, y); @(posedge clk) x |=> y; endproperty
  sequence s_arg(x); x ##1 !x; endsequence
  default clocking cb @(posedge clk); endclocking
  p1: assert property (p_dis) else $display("FAIL pdis %0d", $time);
  p2: assert property (p_arg(a, b)) else $display("FAIL parg %0d", $time);
  p3: assert property (a |=> s_arg(b)) else $display("FAIL sarg %0d", $time);
  p4: assert property (a |=> b) else $display("FAIL dflt %0d", $time);
  initial begin
    #1 a = 1;             // 5
    #10 a = 0; r = 1;     // 15: b=0 -> parg FAIL 15, dflt FAIL 15, sarg: b=0 fails 15; pdis cancelled
    #10 r = 0; a = 1;     // 25
    #10 a = 0; b = 1;     // 35: b=1 -> ok for parg/dflt/pdis; sarg: b=1 then !b at 45
    #10 b = 0;            // 45: sarg ok
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec!["FAIL dflt 15", "FAIL parg 15", "FAIL sarg 15"];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}

#[test]
fn cover_sequence_and_strong_obligations_at_end_of_sim() {
    let text = run(
        "s9_cover_strong",
        r#"
module t; logic clk = 0; always #5 clk = ~clk;
  logic a = 0, b = 0;
  c1: cover property (@(posedge clk) a ##[1:2] b) $display("COVER %0d", $time);
  p1: assert property (@(posedge clk) a |-> s_eventually b) else $display("FAIL sev %0d", $time);
  p2: assert property (@(posedge clk) a |-> strong(##[1:$] b)) else $display("FAIL strong %0d", $time);
  p3: assert property (@(posedge clk) a |-> weak(##[1:$] b)) else $display("FAIL weak %0d", $time);
  initial begin
    #1 a = 1;           // 5
    #10 a = 0; b = 1;   // 15: cover hits (a@5 ##1 b@15)
    #10 b = 0; a = 1;   // 25: a, b never again -> sev/strong fail at end
    #10 a = 0;
    #30 $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut expected = vec!["COVER 15", "FAIL sev 61", "FAIL strong 61"];
    expected.sort();
    assert_eq!(reports(&text), expected, "report lines:\n{text}");
}
