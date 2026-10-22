//! §11.6.1, §11.8.2: the operands of a context-determined operator are
//! extended to the width of the expression's context before the operation,
//! and take the expression's signedness. Concurrent assertions follow the
//! same rules (§16.6 gives none of its own for the Boolean expressions of
//! sequences and properties).
//!
//! The evaluator sizes a comparison by the self-determined widths of both
//! operands, and fell back to the surrounding context when it could not
//! size one. It could size neither a signal read in a concurrent assertion
//! (resolved to its signal id when the assertion registers) nor a select, a
//! struct member or a concatenation. In a Boolean the surrounding context
//! has no width, so the operation ran at the operands' own width: over
//! 16-bit `a` and `b`, `a * b == 32'sh0001_0000` multiplied at 16 bits and
//! failed. A match item `x = u + v` also added at the operands' width, not
//! at the width of the local variable `x`. A size cast to a parameter width
//! (`W'(r)`) lost its width: its parameter was resolved like a signal read.
//!
//! Every expression below holds under those rules and fails at a narrower
//! width or with the other signedness. Verilator 5.038 holds each of them as
//! an assertion and as a `disable iff` guard, and holds the `$past` and
//! select checks of the second test and every check of the third. It can't
//! run property local variables or `##1` in a sequence; those forms rest on
//! §16.10 and §11.6.1.

use std::path::PathBuf;
use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sva_context_width");
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

/// The design's PASS and FAIL lines.
fn report(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|l| l.starts_with("PASS") || l.starts_with("FAIL"))
        .collect()
}

/// Expressions that hold only when every operand is sized and signed by its
/// context. `@u` is the 8-bit `u`, or in the local-variable form a property
/// local holding it (§16.10).
const EXPRS: &[(&str, &str)] = &[
    // + - * / % **
    ("add", "@u + v == 9'h100"),
    ("sub", "v - @u == 9'h102"),
    ("mul", "a * b == 32'sh0001_0000"),
    ("mul_case_eq", "a * b === 32'sh0001_0000"),
    ("mul_right", "m === a * b"),
    ("div", "(@u + v) / 2 == 9'h080"),
    ("mod", "(@u + v) % 9'h101 == 9'h100"),
    ("pow", "@u ** 2 == 16'hfe01"),
    ("neg", "-@u == 9'h101"),
    // shifts: the left operand takes the context, the amount does not
    ("shl", "(@u << 1) == 9'h1fe"),
    ("shr", "((@u + v) >> 1) == 9'h080"),
    ("ashl", "(t <<< 7) == 9'sh080"),
    ("ashr", "((@u + v) >>> 1) == 9'h080"),
    ("shift_amount", "(9'h001 << (@u + v)) == 9'h001"),
    ("signed_ashl", "(s <<< 1) == -16'sd2"),
    ("signed_ashr", "($signed(@u) >>> 1) == -16'sd1"),
    // bitwise
    ("not", "(~@u) == 9'h100"),
    ("xor", "((@u ^ 8'h00) + v) == 9'h100"),
    ("and_wide", "(@u & 16'hff00) == 16'h0000"),
    ("xor_wide", "(@u ^ 16'hffff) == 16'hff00"),
    // the conditional operator
    ("cond", "(c ? @u + v : 8'd0) == 9'h100"),
    ("cond_operand", "(c ? @u : v) + v == 9'h100"),
    ("cond_wide", "(c ? @u : 16'h0000) + v == 17'h00100"),
    ("cond_unsigned", "(c ? s : 16'h0) == 16'h00ff"),
    ("cond_signed", "(c ? s : 16'sh0) == -16'sd1"),
    // concatenation: its operands are self-determined, the result is not
    ("concat", "{@u + v} == 9'h000"),
    ("concat_two", "{@u + v, 1'b0} == 9'h000"),
    ("replicate", "{2{@u + v}} == 17'h00000"),
    ("concat_operand", "({@u, @u} + 16'd1) == 17'h10000"),
    ("replicate_operand", "({2{@u}} + 16'd1) == 17'h10000"),
    ("concat_unsigned", "{s} + 16'd0 == 16'h00ff"),
    // signed and unsigned operands
    ("unsigned_add", "s + v == 9'h100"),
    ("signed_add", "p + t == 10'sd128"),
    ("signed_zero_add", "(s + 8'sd0) == -16'sd1"),
    ("signed_mul", "s * t == -16'sd1"),
    ("unsigned_lt", "!(s < v)"),
    ("signed_lt", "s < 9'sd0"),
    ("signed_fn", "$signed(@u) == -9'sd1"),
    ("signed_fn_add", "($signed(@u) + 9'sd1) == 9'sd0"),
    ("unsigned_fn", "$unsigned(s) == 9'h0ff"),
    ("signed_div", "q / t == -16'sd128"),
    ("signed_mod", "(q % 16'sd3) == -16'sd2"),
    ("unsigned_div", "(q / 16'd3) == 16'd21802"),
    // comparisons of operands of different widths
    ("zero_extend", "s != 16'hffff"),
    ("sign_extend", "s == 16'shffff"),
    ("narrow_gt", "!((@u + v) > 8'd0)"),
    ("wide_gt", "(@u + v) > 9'd0"),
    ("wildcard", "(@u + v) ==? 9'h1??"),
    ("inside", "(@u + v) inside {9'h100}"),
    ("logical_and", "(@u + v == 9'h100) && 1'b1"),
    // self-determined operands
    ("reduction", "|(@u + v) == 1'b0"),
    ("logical_not", "!(@u + v)"),
    ("countones", "$countones(@u + v) == 0"),
    // selects and members
    ("part_select", "@u[7:0] + v == 9'h100"),
    ("bit_select", "@u[7] + @u[6] == 2'd2"),
    ("element", "mem[1] + v == 9'h100"),
    ("member", "st.f + v == 9'h100"),
    ("select_mul", "a[15:0] * b == 32'h0001_0000"),
    // a sized bound is truncated to its size: `3'd15` is 7, so 8 bits
    ("sized_bound", "(@u[3'd15:0] + 8'd1) == 8'h00"),
    // size casts: N bits wide, N a literal or a parameter (§6.24.1)
    ("mul_wide", "p32 == r16 * k16"),
    ("param_cast", "p32 == PW'(r16) * PW'(k16)"),
    (
        "cast_element",
        "PW'(mem[1]) * PW'(16'hffff) == 32'h00fe_ff01",
    ),
    ("param_cast_signed", "PW'(q) * PW'(t) == -32'sd128"),
    ("param_cast_operand", "(PW'(r16) + 16'd1) == 33'h1_0000"),
    (
        "size_cast_sum",
        "(32'(r16) + 32'hffff_0001) == 33'h1_0000_0000",
    ),
];

/// Each expression as an assertion, a cover, a named sequence, a `disable
/// iff` guard, a clocking-event guard and with a property local variable.
#[test]
fn operands_take_the_context_width_in_every_assertion_form() {
    let mut src = String::from(
        "module t;
  logic clk = 0;
  logic [7:0] u = 8'hff, v = 8'h01;
  logic signed [7:0] s = -8'sd1, t = 8'sd1, p = 8'sd127;
  logic signed [15:0] a = 16'sh4000, b = 16'sh0004, q = -16'sd128;
  logic signed [31:0] m = 32'sh0001_0000;
  logic c = 1;
  logic [7:0] mem [0:3] = '{8'h00, 8'hff, 8'h00, 8'h00};
  typedef struct packed { logic [3:0] g; logic [7:0] f; } st_t;
  st_t st = 12'h0ff;
  localparam int PW = 32;
  logic [15:0] r16 = 16'hffff, k16 = 16'h0002;
  logic [31:0] p32 = 32'h0001_fffe;
  int passes = 0, covers = 0, fails = 0;
  function void bad(string what);
    fails++;
    $display(\"FAIL %s at %0t\", what, $time);
  endfunction
  always #5 clk = ~clk;
",
    );
    for (name, e) in EXPRS {
        let sig = e.replace('@', "");
        let local = e.replace("@u", "x");
        src.push_str(&format!(
            "  {name}_a: assert property (@(posedge clk) {sig}) passes++; else bad(\"{name}: assert\");
  {name}_c: cover property (@(posedge clk) {sig}) covers++;
  sequence {name}_s; {sig}; endsequence
  {name}_q: assert property (@(posedge clk) 1'b1 ##1 {name}_s) passes++; else bad(\"{name}: sequence\");
  {name}_d: assert property (@(posedge clk) disable iff ({sig}) 1'b0) else bad(\"{name}: disable iff\");
  {name}_g: cover property (@(posedge clk iff ({sig})) 1'b1) covers++;
  property {name}_p; logic [7:0] x; (1'b1, x = u) |-> {local}; endproperty
  {name}_l: assert property (@(posedge clk) {name}_p) passes++; else bad(\"{name}: local variable\");
"
        ));
    }
    // Four ticks: 4 + 3 + 4 passes and 4 + 4 covers per expression.
    let n = EXPRS.len();
    src.push_str(&format!(
        "  initial begin
    repeat (4) @(negedge clk);
    if (passes != {} || covers != {})
      $display(\"FAIL %0d passes, %0d covers\", passes, covers);
    else if (fails == 0)
      $display(\"PASS\");
    $finish;
  end
endmodule
",
        11 * n,
        8 * n
    ));
    let text = run("forms", &src);
    assert_eq!(report(&text), ["PASS"], "{text}");
}

/// The operands and results of `$past`, a property's local variables and
/// its match item assignments, and selects sized by their value (an element
/// of a packed array, of a two-dimensional array, a member of an array
/// element, a part-select with parameter bounds), in an instance. `a_mult`
/// has the shape of a complex multiplier check: 16-bit operands, 32-bit
/// result.
#[test]
fn sampled_values_and_local_variables_take_the_context_width() {
    let text = run(
        "sampled",
        r#"
module m(input logic clk);
  logic [7:0] u = 8'hff, v = 8'h01;
  logic signed [7:0] s = -8'sd1;
  logic signed [15:0] ar = 16'sh4000, ai = -16'sh0003, br = 16'sh0004, bi = 16'sh0100;
  logic signed [31:0] re, im;
  logic valid = 1;
  localparam int W = 8;
  typedef struct packed { logic [3:0] g; logic [7:0] f; } st_t;
  logic [1:0][7:0] pm = 16'hff00;
  logic [7:0] m2 [0:1][0:1] = '{'{8'h00, 8'h00}, '{8'hff, 8'h00}};
  st_t sa [0:1] = '{12'h000, 12'h0ff};
  assign re = ar * br - ai * bi;
  assign im = ar * bi + ai * br;
  int passes = 0, fails = 0;
  function void bad(string what);
    fails++;
    $display("FAIL %s at %0t", what, $time);
  endfunction
  a_mult: assert property (@(posedge clk) valid |=>
      $signed(re) === ($signed($past(ar)) * $signed($past(br))) - ($signed($past(ai)) * $signed($past(bi))))
    passes++; else bad("complex product, real part");
  a_imag: assert property (@(posedge clk) valid |=> im === $past(ar) * $past(bi) + $past(ai) * $past(br))
    passes++; else bad("complex product, imaginary part");
  a_add: assert property (@(posedge clk) 1'b1 ##1 $past(u) + v == 9'h100) passes++; else bad("$past(u) + v");
  a_sext: assert property (@(posedge clk) 1'b1 ##1 $past(s) == -16'sd1) passes++; else bad("$past(s) signed");
  a_zext: assert property (@(posedge clk) 1'b1 ##1 $past(s) + 16'd0 == 16'h00ff) passes++; else bad("$past(s) unsigned");
  a_shr: assert property (@(posedge clk) 1'b1 ##1 (($past(u) + v) >> 1) == 9'h080) passes++; else bad("($past(u) + v) >> 1");
  a_stable: assert property (@(posedge clk) 1'b1 ##1 $stable(u) + $stable(v) == 2'd2) passes++; else bad("$stable + $stable");
  property p_sum; logic [8:0] x; (1'b1, x = u + v) |-> x == 9'h100; endproperty
  a_sum: assert property (@(posedge clk) p_sum) passes++; else bad("x = u + v");
  property p_sign; logic signed [15:0] x; (1'b1, x = s) |-> x == -16'sd1; endproperty
  a_sign: assert property (@(posedge clk) p_sign) passes++; else bad("x = s");
  property p_acc; logic [8:0] x; (1'b1, x = u) ##1 (1'b1, x = x + v) |-> x == 9'h100; endproperty
  a_acc: assert property (@(posedge clk) p_acc) passes++; else bad("x = x + v");
  property p_read; logic [7:0] x; (1'b1, x = u) |-> x + v == 9'h100; endproperty
  a_read: assert property (@(posedge clk) p_read) passes++; else bad("x + v");
  a_pm: assert property (@(posedge clk) pm[1] + v == 9'h100) passes++; else bad("pm[1] + v");
  a_m2: assert property (@(posedge clk) m2[1][0] + v == 9'h100) passes++; else bad("m2[1][0] + v");
  a_sa: assert property (@(posedge clk) sa[1].f + v == 9'h100) passes++; else bad("sa[1].f + v");
  a_par: assert property (@(posedge clk) u[W-1:0] + v == 9'h100) passes++; else bad("u[W-1:0] + v");
endmodule
module t;
  logic clk = 0;
  always #5 clk = ~clk;
  m i0(clk);
  initial begin
    repeat (4) @(negedge clk);
    // 3 passes for each of the seven ##1/|=> checks and a_acc, 4 for the rest
    if (i0.passes != 3 * 8 + 4 * 7) $display("FAIL %0d passes", i0.passes);
    else if (i0.fails == 0) $display("PASS");
    $finish;
  end
endmodule
"#,
    );
    assert_eq!(report(&text), ["PASS"], "{text}");
}

/// The same sizing applies to procedural code the evaluator runs (here an
/// `initial` block): a select, an array element, a struct member and a
/// concatenation in a comparison were also sized by the empty Boolean
/// context.
#[test]
fn procedural_selects_take_the_context_width() {
    let text = run(
        "procedural",
        r#"
module t;
  logic [7:0] u = 8'hff, v = 8'h01;
  logic [7:0] mem [0:3] = '{8'h00, 8'hff, 8'h00, 8'h00};
  typedef struct packed { logic [3:0] g; logic [7:0] f; } st_t;
  st_t st = 12'h0ff;
  initial begin
    #1;
    if (!(u[7:0] + v == 9'h100)) $display("FAIL part select");
    if (!(mem[1] + v == 9'h100)) $display("FAIL element");
    if (!(st.f + v == 9'h100)) $display("FAIL member");
    if (!({u, u} + 16'd1 == 17'h10000)) $display("FAIL concatenation");
    if (!({2{u}} + 16'd1 == 17'h10000)) $display("FAIL replication");
    $display("PASS");
  end
endmodule
"#,
    );
    assert_eq!(report(&text), ["PASS"], "{text}");
}
