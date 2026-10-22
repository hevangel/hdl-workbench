//! §9.4.5: an intra-assignment event control may name an event without
//! parentheses — `a = @ e v;`, `b = repeat (2) @ e v;` (ivtest
//! `always3.1.1G/H/K`). The pre-parse canonicalization only recognized the
//! parenthesized `@(...)` form; the bare one fell to the parser, which
//! discards intra-assignment timing, so the assignment happened at once and
//! `always a = @ e v;` became an untimed loop. Cross-checked against the
//! reference simulator.

#[test]
fn bare_event_identifier_suspends_the_assignment() {
    let sim = xezim::simulate(
        r#"
module main;
  reg [3:0] a, b, c;
  event e;
  reg s = 0;
  initial begin
    #5 -> e;
    #5 -> e;
    #1 s = 1;
  end
  always a = @ e 4'h5;
  initial begin b = repeat (2) @ e 4'h9; end
  initial begin c = @ (s) 4'h3; end
  initial begin
    $display("I|%0t %h %h %h", $time, a, b, c);
    #6 $display("I|%0t %h %h %h", $time, a, b, c);
    #6 $display("I|%0t %h %h %h", $time, a, b, c);
    $finish;
  end
endmodule
"#,
        100,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(
        o.iter()
            .filter(|l| l.starts_with("I|"))
            .cloned()
            .collect::<Vec<_>>(),
        vec!["I|0 x x x", "I|6 5 x x", "I|12 5 9 3"],
        "{o:?}"
    );
}
