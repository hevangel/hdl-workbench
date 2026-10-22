//! IEEE 1800-2017 §5.6.1: an escaped identifier (`\cpu3 `) is the SAME
//! identifier as the nonescaped spelling of its contents (`cpu3`), so both
//! forms must resolve to one symbol.
//!
//! Root cause: the parser stored the token text verbatim into the AST name,
//! so an escaped `\cpu3` never matched a reference spelled `cpu3` (or vice
//! versa). Under `` `default_nettype none `` the miss fell through to an
//! implicit net and elaborated to
//! `Simulation error: Implicit net 'cpu3' under `default_nettype none`.
//! The fix normalizes escaped identifiers at `parse_identifier` (strip the
//! leading backslash) while the lexer still reports the raw `\…` spelling
//! (`tests/sanity_tests.rs::test_lex_escaped_identifier`).
//!
//! Covered here: escaped declaration referenced nonescaped, nonescaped
//! declaration referenced escaped, and an escaped identifier with
//! non-identifier characters that can only be addressed in its escaped form.

use xezim::simulate;

/// Escaped declaration (`reg \cpu3 `), nonescaped reference (`cpu3`):
/// the LRM §5.6.1 identity rule.
#[test]
fn escaped_decl_nonescaped_ref_resolves() {
    let src = r#"
`default_nettype none
module identifiers;
  reg \cpu3 ;
  wire reference_test;
  assign reference_test = cpu3;
  initial begin
    \cpu3 = 1'b1;
    #1;
    if (reference_test !== 1'b1) $fatal(1, "escaped/nonescaped name mismatch");
  end
endmodule
"#;
    simulate(src, 10).expect("escaped-declared reg must resolve to its nonescaped name");
}

/// Nonescaped declaration, escaped reference (`\cpu3 `) — the mirror image
/// of the §5.6.1 rule.
#[test]
fn nonescaped_decl_escaped_ref_resolves() {
    let src = r#"
`default_nettype none
module identifiers;
  reg cpu3;
  wire reference_test;
  assign reference_test = \cpu3 ;
  initial begin
    cpu3 = 1'b1;
    #1;
    if (reference_test !== 1'b1) $fatal(1, "nonescaped/escaped name mismatch");
  end
endmodule
"#;
    simulate(src, 10).expect("nonescaped-declared reg must resolve to its escaped name");
}

/// An escaped identifier containing characters that a nonescaped identifier
/// cannot spell (a `-`) must still address the same object when referenced
/// in its escaped form.
#[test]
fn escaped_only_name_still_resolves_escaped() {
    let src = r#"
`default_nettype none
module identifiers;
  reg \a-b ;
  initial begin
    \a-b = 1'b1;
  end
endmodule
"#;
    simulate(src, 10).expect("escaped-only identifier must resolve when referenced escaped");
}
