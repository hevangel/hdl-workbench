//! IEEE 1800 §16.12.8 / §16.12.12: temporal operands retain their obligations.
use xezim::simulate;
fn check(property: &str, setup: &str, expected: &[&str]) {
    let source = format!(
        r#"module top;
 logic clk=0,start=1,a=1,b=0,trigger=1,done=0;
 always #5 clk=~clk;
 initial begin $assertvacuousoff; #10 start=0; end
 initial begin {setup} end
 assert property(@(posedge clk) start |-> ({property}))
 $display("T|P %0t",$time); else $display("T|F %0t",$time);
 initial #40 $finish;
 endmodule"#
    );
    let result = simulate(&source, 100).unwrap();
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(lines, expected);
}
#[test]
fn weak_until_without_terminator() {
    check("a until b", ";", &[]);
}
#[test]
fn strong_until_without_terminator_fails_at_end() {
    check("a s_until b", ";", &["T|F 40"]);
}
#[test]
fn terminating_tick_does_not_require_left() {
    check("a until b", "a=0;b=1;", &["T|P 5"]);
}
#[test]
fn left_failure_is_not_hidden_by_a_later_terminator() {
    check("a until b", "#10 a=0;#10 b=1;", &["T|F 15"]);
}
#[test]
fn pending_left_operand_is_not_a_success() {
    check("(1 ##1 a) until b", "#10 begin a=0;b=1;end", &["T|F 15"]);
}
#[test]
fn right_operand_starts_on_every_tick() {
    check("a s_until (trigger ##1 done)", "#20 done=1;", &["T|P 25"]);
}
#[test]
fn implication_is_a_legal_until_operand() {
    check("(b |-> a) s_until done", "#20 done=1;", &["T|P 25"]);
}
#[test]
fn iff_compares_completed_temporal_properties() {
    check("(1 ##1 a) iff (1 ##1 b)", ";", &["T|F 15"]);
}
#[test]
fn iff_accepts_two_failed_temporal_properties() {
    check("(1 ##1 a) iff (1 ##1 b)", "a=0;", &["T|P 15"]);
}

#[test]
fn negated_weak_until_fails_when_the_clock_stops() {
    check("not (a until b)", ";", &["T|F 40"]);
}
#[test]
fn until_of_vacuous_implications_remains_vacuous() {
    check("(b |-> a) until (b |-> done)", ";", &[]);
}
#[test]
fn iff_compares_four_state_property_truth() {
    for a in ['0', '1', 'x', 'z'] {
        for b in ['0', '1', 'x', 'z'] {
            let setup = format!("a=1'b{a};b=1'b{b};");
            let expected = if (a == '1') == (b == '1') {
                "T|P 5"
            } else {
                "T|F 5"
            };
            check("a iff b", &setup, &[expected]);
        }
    }
}
