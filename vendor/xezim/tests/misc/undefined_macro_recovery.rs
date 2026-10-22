//! #451, §22.5.1: an undefined statement macro does not consume following code.

#[test]
fn undefined_statement_invocations_preserve_the_following_assignment() {
    for invocation in [
        "`MISSING(1, 2)",
        "`MISSING()",
        "`MISSING(f(1, 2), '{3, 4})",
        "`MISSING(\"), (\", 2);",
        "`MISSING;",
    ] {
        let source = format!(
            "module top; int x; initial begin {invocation}\n x = 19; $display(\"T|x=%0d\", x); $finish; end endmodule"
        );
        let simulation = xezim::simulate(&source, 10).expect("macro recovery failed");
        assert!(
            simulation
                .output
                .iter()
                .any(|entry| entry.message == "T|x=19")
        );
    }
}
