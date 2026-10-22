//! #453, §16.5 / §9.4.2: each event-list term clocks the same assertion site.

fn check(source: &str, expected: &str) {
    let simulation = xezim::simulate(source, 200).expect("simulation failed");
    let lines: Vec<_> = simulation
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .filter(|line| line.starts_with("T|"))
        .collect();
    assert_eq!(lines, [expected]);
}

const UNION: &str = r#"
module top;
  logic clk = 0, rst = 1;
  int n_or = 0, n_comma = 0;
  assert property (@(posedge clk or negedge rst) 1) n_or++;
  assert property (@(posedge clk, negedge rst) 1) n_comma++;
  initial begin
    #2 clk = 1;
    #2 clk = 0;
    #2 rst = 0;
    #2 rst = 1;
    #2 begin clk = 1; rst = 0; end
    #1 $display("T|union|%0d|%0d", n_or, n_comma);
    $finish;
  end
endmodule
"#;

#[test]
fn or_and_comma_clock_once_for_simultaneous_terms() {
    check(UNION, "T|union|3|3");
}

const GUARDED: &str = r#"
module top;
  logic clk = 0, rst = 1, cg = 0, rg = 1;
  int n = 0;
  assert property (@(posedge clk iff cg or negedge rst iff rg) 1) n++;
  initial begin
    #1 clk = 1;
    #1 rst = 0;
    #1 rst = 1;
    #1 begin cg = 1; rg = 0; end
    #1 clk = 0;
    #1 clk = 1;
    #1 rst = 0;
    #1 $display("T|guards|%0d", n);
    $finish;
  end
endmodule
"#;

#[test]
fn each_term_checks_its_own_iff() {
    check(GUARDED, "T|guards|2");
}

const NESTED: &str = r#"
module top;
  logic a = 0, b = 1, c = 0;
  int n = 0;
  assert property (@((posedge a) or (negedge b, c)) 1) n++;
  initial begin
    #1 a = 1;
    #1 b = 0;
    #1 c = 1;
    #1 c = 0;
    #1 $display("T|nested|%0d", n);
    $finish;
  end
endmodule
"#;

#[test]
fn nested_lists_keep_edge_and_any_change_terms() {
    check(NESTED, "T|nested|4");
}

const EXPRESSIONS: &str = r#"
module top;
  logic a = 0, b = 0;
  logic [3:0] bus = 0;
  int n = 0;
  assert property (@(posedge (a && b) or bus) 1) n++;
  initial begin
    #1 a = 1;
    #1 b = 1;
    #1 a = 0;
    #1 bus = 8;
    #1 bus = 0;
    #1 $display("T|expressions|%0d", n);
    $finish;
  end
endmodule
"#;

#[test]
fn expression_clocks_and_high_vector_bits_trigger() {
    check(EXPRESSIONS, "T|expressions|3");
}

const NAMED: &str = r#"
module monitor(input logic c, r);
  int np = 0, ns = 0;
  property p(x, y); @(posedge x or negedge y) 1; endproperty
  sequence s(x, y); @(posedge x, negedge y) 1; endsequence
  assert property (p(c, r)) np++;
  cover sequence (s(c, r)) ns++;
endmodule
module top;
  logic c = 0, r = 1;
  monitor u(c, r);
  initial begin
    #1 c = 1;
    #1 c = 0;
    #1 r = 0;
    #1 $display("T|named|%0d|%0d", u.np, u.ns);
    $finish;
  end
endmodule
"#;

#[test]
fn named_properties_and_sequences_rewrite_all_child_clock_formals() {
    check(NAMED, "T|named|2|2");
}

const HISTORY: &str = r#"
module top;
  logic a = 0, b = 0;
  int d = 7, expected = 7, passes = 0, failures = 0;
  assert property (@(posedge a or posedge b) $past(d) == expected)
    begin
      passes++;
      if ($past(d) != expected) failures++;
    end
    else failures++;
  initial begin
    #1 d = 11;
    #1 a = 1;
    #1 begin d = 22; expected = 11; end
    #1 b = 1;
    #1 begin a = 0; b = 0; end
    #1 begin d = 33; expected = 22; end
    #1 a = 1;
    #1 $display("T|history|%0d|%0d", passes, failures);
    $finish;
  end
endmodule
"#;

#[test]
fn sampled_calls_share_history_across_all_terms_and_actions() {
    check(HISTORY, "T|history|3|0");
}

#[test]
fn simultaneous_clock_terms_share_one_history_sample() {
    let mut source = HISTORY.to_string();
    let assignment = "#1 a = 1;";
    let start = source.rfind(assignment).unwrap();
    source.replace_range(
        start..start + assignment.len(),
        "#1 begin a = 1; b = 1; end",
    );
    check(&source, "T|history|3|0");
}

#[test]
fn duplicate_terms_do_not_duplicate_assertion_attempts() {
    let source = UNION.replace(
        "posedge clk or negedge rst",
        "posedge clk or posedge clk or negedge rst",
    );
    check(&source, "T|union|3|3");
}

const GUARDED_HISTORY: &str = r#"
module top;
  logic a = 0, b = 0, g = 0;
  int d = 7, expected = 7, passes = 0, failures = 0;
  assert property (@(posedge a iff g or posedge b) $past(d) == expected)
    passes++;
    else failures++;
  initial begin
    #1 d = 11;
    #1 a = 1;
    #1 d = 22;
    #1 b = 1;
    #1 begin a = 0; b = 0; g = 1; end
    #1 begin d = 33; expected = 22; end
    #1 a = 1;
    #1 $display("T|guard-history|%0d|%0d", passes, failures);
    $finish;
  end
endmodule
"#;

#[test]
fn rejected_clock_terms_do_not_advance_sampled_history() {
    check(GUARDED_HISTORY, "T|guard-history|2|0");
}

const EXPECT: &str = r#"
module top;
  logic clk = 0, rst = 1, valid = 1;
  int result = 0;
  initial begin
    expect (@(posedge clk or negedge rst) valid) result = 1;
    else result = -1;
  end
  initial begin
    #1 rst = 0;
    #1 $display("T|expect|%0d", result);
    $finish;
  end
endmodule
"#;

#[test]
fn expect_can_complete_on_the_second_clock_term() {
    check(EXPECT, "T|expect|1");
}
