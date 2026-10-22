//! §23.2.2.1: a non-ANSI port declared without a net or variable type may be
//! declared ONCE more, by a net or variable declaration, and when the port
//! declaration has a range the two ranges shall be identical. Allowing the
//! completing declaration (see `nonansi_port_completion`) accepted any
//! number of them and any width (ivtest `module_nonansi_fail3/4`,
//! `module_nonansi_vec_fail1/3`). The reference simulator rejects the double
//! declarations ("already declared in this scope") but accepts the width
//! mismatches silently; the width rule is enforced here per the LRM text.

use xezim::simulate;

fn reject(name: &str, src: &str) {
    assert!(
        simulate(src, 10).is_err(),
        "{name}: expected an elaboration error"
    );
}

fn accept(name: &str, src: &str) {
    let r = simulate(src, 10);
    assert!(r.is_ok(), "{name}: expected success, got {:?}", r.err());
}

#[test]
fn second_completing_declaration_is_rejected() {
    reject(
        "reg twice",
        "module test(x); output x; reg x; reg x; endmodule",
    );
    reject(
        "wire then reg",
        "module test(x); input x; wire x; reg x; endmodule",
    );
    reject(
        "wire twice",
        "module test(x); output x; wire x; wire x; endmodule",
    );
}

#[test]
fn completing_declaration_must_match_the_port_range() {
    reject(
        "[3:0] vs [7:0]",
        "module test(x); output [3:0] x; reg [7:0] x; endmodule",
    );
    reject(
        "[7:0] vs scalar reg",
        "module test(x); output [7:0] x; reg x; endmodule",
    );
}

#[test]
fn legal_completions_still_elaborate() {
    accept(
        "reg completes",
        "module test(x); output x; reg x; endmodule",
    );
    accept(
        "wire completes",
        "module test(x); input x; wire x; endmodule",
    );
    accept(
        "same range",
        "module test(x); output [7:0] x; reg [7:0] x; endmodule",
    );
    accept(
        "bare wire takes the port range",
        "module test(x); input [7:0] x; wire x; endmodule",
    );
    accept(
        "equal width through a parameter",
        "module test(x); parameter W = 8; output [W-1:0] x; reg [7:0] x; endmodule",
    );
}
