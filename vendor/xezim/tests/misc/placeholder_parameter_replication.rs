//! A library module elaborated on its own keeps the placeholder defaults of
//! its parameters (`els_p = -1`, overridden by every real instantiation), so
//! xezim does not judge a replication count built from such a default (the
//! reference simulator, elaborating the module with `-1`, reports a negative
//! count; sv-tests elaborates BaseJump STL modules this way). A localparam or
//! a non-negative default is still judged.

use xezim::simulate;

#[test]
fn negative_default_parameter_is_not_folded() {
    let src = "module lib #(parameter els_p = -1) ();\n\
               wire [7:0] s;\n\
               assign s = {8{1'b1}} | {els_p{1'b1}};\n\
               endmodule\n";
    assert!(simulate(src, 10).is_ok());
}

#[test]
fn nonnegative_defaults_and_localparams_are_folded() {
    for src in [
        "module top; parameter wid = 9; wire [31:0] a; assign a = {(wid-16){8'b0}}; endmodule",
        "module top; localparam integer N = -1; initial begin int x, y; y = N'(x); end endmodule",
    ] {
        assert!(simulate(src, 10).is_err(), "accepted:\n{src}");
    }
}
