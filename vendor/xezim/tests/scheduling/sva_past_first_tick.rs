//! §16.9.3 — sampled-value functions at a clock tick before the first one.
//!
//! When fewer than `number_of_ticks` ticks have occurred, `$past` returns the
//! DEFAULT SAMPLED VALUE of its operand, and at the first tick `$rose`,
//! `$fell`, `$stable` and `$changed` compare the sampled value with it.
//! §16.5.1 defines that value as the one before time 0: a static variable's
//! declaration initializer, else its type's default (x for `logic`, 0 for
//! `bit`), and the type's default for a net. A write at time 0 does not count,
//! but an initializer xezim runs as a time-0 process (`fv` below) does.
//!
//! The defaults were taken when the watches registered: after the time-0
//! settle, so the net `w` read its driver's value instead of x; before a
//! function-call initializer ran, so `fv` read x instead of 9; and, for an
//! assertion inside a procedure, after the procedure's own time-0 writes.
//! Procedural calls (`always @(posedge clk) ... $past(w)`) had the same
//! problems.
//!
//! Expected values follow §16.5.1 and §16.9.3. Verilator 5.038 can't check
//! them: it is two-state and starts `$past` at 0 rather than at the
//! declaration's value, and it doesn't run procedural concurrent assertions,
//! `wait_order` or `randsequence`.

use std::path::PathBuf;
use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sva_past_first_tick");
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

#[test]
fn sampled_value_functions_use_default_sampled_values_before_the_first_tick() {
    let text = run(
        "defaults",
        r#"
module t;
  logic clk = 0;
  always #5 clk = ~clk;
  logic first = 1;                 // sampled as 1 at the first tick only
  always @(posedge clk) first <= 0;
  logic d = 0, q = 0, q2 = 0;
  always_ff @(posedge clk) begin q <= d; q2 <= q; end
  logic [7:0] v = 8'hA5;           // declaration initializer
  logic u;                         // no initializer: logic's default, x
  bit b;                           // no initializer: bit's default, 0
  int n = -7;
  logic r = 0, f = 1, i;           // all three written at time 0 below
  wire w; assign w = r;            // a net: its type's default, x
  logic en = 0;
  function automatic logic [7:0] triple(input int x); return 8'(x * 3); endfunction
  logic [7:0] fv = triple(3);      // an initializer run as a time-0 process
  int k = 0;                       // sampled as the number of earlier ticks
  logic t1 = 0;                    // sampled as 1 from the second tick on
  always_ff @(posedge clk) begin k <= k + 1; t1 <= 1; end
  initial begin
    r = 1; f = 0; i = 1;
    @(negedge clk) d = 1;
    @(negedge clk) d = 0; en = 1;
    repeat (3) @(negedge clk);
    $finish;
  end
  a_first: assert property (@(posedge clk) !first) else $display("FIRST %0t", $time);
  // The unguarded check of the report.
  a_q:  assert property (@(posedge clk) q == $past(d)) else $display("FAIL q %0t", $time);
  a_v:  assert property (@(posedge clk) $past(v) === 8'hA5) else $display("FAIL v %0t", $time);
  a_fv: assert property (@(posedge clk) $past(fv) === 8'd9 && $stable(fv)) else $display("FAIL fv %0t", $time);
  a_u:  assert property (@(posedge clk) $past(u) === 1'bx) else $display("FAIL u %0t", $time);
  a_b:  assert property (@(posedge clk) $past(b) === 1'b0) else $display("FAIL b %0t", $time);
  a_n:  assert property (@(posedge clk) $past(n) == -7) else $display("FAIL n %0t", $time);
  a_i:  assert property (@(posedge clk) $past(i) === (first ? 1'bx : 1'b1)) else $display("FAIL i %0t", $time);
  a_w:  assert property (@(posedge clk) $past(w) === (first ? 1'bx : 1'b1)) else $display("FAIL w %0t", $time);
  // Two ticks back: the default until a second tick has passed.
  a_d2: assert property (@(posedge clk) $past(d, 2) === q2) else $display("FAIL d2 %0t", $time);
  // Gated: no tick with `en` true yet.
  a_g:  assert property (@(posedge clk) $past(v, 1, en) === 8'hA5) else $display("FAIL g %0t", $time);
  a_rr: assert property (@(posedge clk) $rose(r) === first) else $display("FAIL rose_r %0t", $time);
  a_ri: assert property (@(posedge clk) $rose(i) === first) else $display("FAIL rose_i %0t", $time);
  a_rw: assert property (@(posedge clk) $rose(w) === first) else $display("FAIL rose_w %0t", $time);
  a_ff: assert property (@(posedge clk) $fell(f) === first) else $display("FAIL fell_f %0t", $time);
  a_sv: assert property (@(posedge clk) $stable(v)) else $display("FAIL stable_v %0t", $time);
  a_sr: assert property (@(posedge clk) $stable(r) === !first) else $display("FAIL stable_r %0t", $time);
  a_ci: assert property (@(posedge clk) $changed(i) === first) else $display("FAIL changed_i %0t", $time);
  // `t1` is written by an NBA at each tick and read only through $rose/$past.
  a_rt: assert property (@(posedge clk) $rose(t1) === (k == 1)) else $display("FAIL rose_t1 %0t", $time);
  a_pt: assert property (@(posedge clk) $past(t1) === (k >= 2)) else $display("FAIL past_t1 %0t", $time);
  // Procedural calls read the same default as the assertions.
  always @(posedge clk) begin
    if ($past(w) !== (first ? 1'bx : 1'b1) || $rose(w) !== first)
      $display("FAIL proc_w %0t", $time);
    if ($past(fv) !== 8'd9 || $changed(fv))
      $display("FAIL proc_fv %0t", $time);
  end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let reports: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("FAIL") || l.starts_with("FIRST"))
        .collect();
    assert_eq!(reports, ["FIRST 5"], "report lines:\n{text}");
}

/// An assertion written inline in a procedure under a `wait` reads the same
/// defaults, though the procedure writes its operands at time 0 before the
/// assertion registers (§16.9.3, §16.14.6).
#[test]
fn inline_assertion_under_wait_uses_default_sampled_values() {
    let text = run(
        "wait_inline",
        r#"
module t;
  logic clk = 0;
  always #5 clk = ~clk;
  bit b = 0;
  logic [3:0] v = 4'h9;
  initial begin
    b = 1;
    wait (1)
      a_wb: assert property (@(posedge clk) $past(b) === 1'b0) $display("ran wait_b");
        else $display("FAIL wait_b %0t", $time);
  end
  initial begin
    v = 4'h6;
    fork
      wait (clk == 0) a_wv: assert property (@(posedge clk) $past(v) === 4'h9) $display("ran wait_v");
        else $display("FAIL wait_v %0t", $time);
    join_none
  end
  initial begin repeat (3) @(negedge clk); $finish; end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut reports: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("FAIL") || l.starts_with("ran "))
        .collect();
    reports.sort_unstable();
    assert_eq!(
        reports,
        ["ran wait_b", "ran wait_v"],
        "report lines:\n{text}"
    );
}

/// An inline assertion in any statement that can hold one reads the same
/// defaults: a `randcase` arm, a `wait_order` pass or fail statement, a
/// `randsequence` code block, and a sampled-value call in an assertion's
/// action block, which the assertion clocks. Each operand is written at time
/// 0 just before its assertion registers, so reading the live value instead
/// of the default sampled value fails at the first tick, the only one an
/// assertion in an `initial` procedure attempts (§16.9.3, §16.14.6).
#[test]
fn inline_assertion_in_any_statement_uses_default_sampled_values() {
    let text = run(
        "nested_inline",
        r#"
module t;
  logic clk = 0;
  always #5 clk = ~clk;
  bit rc = 0, wp = 0, wf = 0, rs = 0, ab = 0;
  event e1, e2, e3, e4;
  initial begin
    rc = 1;
    randcase
      1: a_rc: assert property (@(posedge clk) $past(rc) === 1'b0) $display("ran randcase");
        else $display("FAIL randcase %0t", $time);
    endcase
  end
  initial begin
    wp = 1;
    wait_order (e1, e2)
      a_wp: assert property (@(posedge clk) $past(wp) === 1'b0) $display("ran wait_order");
        else $display("FAIL wait_order %0t", $time);
  end
  initial begin
    wf = 1;
    wait_order (e3, e4)
    else a_wf: assert property (@(posedge clk) $past(wf) === 1'b0) $display("ran wait_order_else");
        else $display("FAIL wait_order_else %0t", $time);
  end
  initial begin #1 -> e1; #1 -> e2; #1 -> e4; #1 -> e3; end
  initial begin
    rs = 1;
    randsequence (main)
      main : { a_rs: assert property (@(posedge clk) $past(rs) === 1'b0) $display("ran randsequence");
        else $display("FAIL randsequence %0t", $time); };
    endsequence
  end
  initial begin
    ab = 1;
    a_ab: assert property (@(posedge clk) 0)
      else if ($past(ab) === 1'b0) $display("ran action");
      else $display("FAIL action %0t", $time);
  end
  initial begin repeat (3) @(negedge clk); $finish; end
endmodule
"#,
    );
    assert!(text.contains("$finish called"), "did not finish:\n{text}");
    let mut reports: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("FAIL") || l.starts_with("ran "))
        .collect();
    reports.sort_unstable();
    let ran = [
        "ran action",
        "ran randcase",
        "ran randsequence",
        "ran wait_order",
        "ran wait_order_else",
    ];
    assert_eq!(reports, ran, "report lines:\n{text}");
}
