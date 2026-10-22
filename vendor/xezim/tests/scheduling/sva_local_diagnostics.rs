//! §16.10: property and sequence local variables are accepted (they were
//! rejected with a diagnostic while assertion-local state was unmodelled),
//! and a match item may assign only a local variable.

#[test]
fn assertion_local_state_is_accepted() {
    for kind in ["property", "sequence"] {
        let source = format!(
            "module tb; bit clk; {kind} duration; time stamp; \
             @(posedge clk) (1'b1, stamp = $time) ##1 ($time > stamp); \
             end{kind} ap: assert property (duration); endmodule"
        );
        if let Err(error) = xezim::simulate(&source, 10) {
            panic!("{kind}-local state was rejected: {error}");
        }
    }
}

#[test]
fn match_item_assignment_to_a_signal_is_rejected() {
    let source = "module tb; bit clk, a; int d; \
                  ap: assert property (@(posedge clk) (a, d = 1) |-> a); endmodule";
    match xezim::simulate(source, 10) {
        Ok(_) => panic!("a match item assigning a signal was accepted"),
        Err(error) => assert!(
            error.contains("illegal assignment to 'd' in a match item list"),
            "wrong diagnostic: {error}"
        ),
    }
}
