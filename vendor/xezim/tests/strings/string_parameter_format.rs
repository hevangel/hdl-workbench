//! §6.16/§6.20: a `string` parameter holds its text, and `%s` prints that
//! text the way it prints a `string` variable. The parameter used to be fit
//! to the 1024-bit placeholder width of `string`, so `%s` printed it behind
//! 125 leading spaces; an untyped parameter initialized by a string literal
//! was cut to 32 bits ("yped" for "untyped") in the top module and in
//! packages. Every expected line below was cross-checked against the
//! reference simulator.

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

const DESIGN: &str = r#"
package p;
  parameter string PK = "pkgstr";
  localparam PKU = "pkg_untyped";
endpackage
module tb #(parameter string PS = "def", parameter PU = "untyped") ();
  localparam string LS = "loc";
  localparam LU = "locu_long";
  localparam string LC = {PS, "_x"};
  string sv = "var";
  sub #(.NAME("inst")) u();
  initial begin
    $display("[%s] [%s] [%s] [%s] [%s] [%s]", PS, PU, LS, LU, LC, sv);
    $display("[%s]", $sformatf("%s-%s", PS, LS));
    $display("[%0s] [%10s] [%-8s]", PS, LS, PU);
    $display("[%s] [%s]", p::PK, p::PKU);
    $display("bits %0d %0d %0d", $bits(PS), $bits(PU), $bits(LC));
  end
endmodule
module sub #(parameter string NAME = "d", parameter UN = "untyped_long");
  localparam string L = "sub_local";
  initial $display("sub [%s] [%s] [%s]", NAME, UN, L);
endmodule
"#;

#[test]
fn string_parameters_print_their_text() {
    let o = out(DESIGN);
    for want in [
        "[def] [untyped] [loc] [locu_long] [def_x] [var]",
        "[def-loc]",
        "[def] [       loc] [untyped ]",
        "[pkgstr] [pkg_untyped]",
        "bits 24 56 40",
        "sub [inst] [untyped_long] [sub_local]",
    ] {
        assert!(o.contains(want), "missing `{}`:\n{}", want, o);
    }
}

/// A `string` parameter has no declared length: text past the 128 characters
/// of the placeholder width is kept whole.
#[test]
fn long_string_parameter_is_not_cut() {
    let long = "ab".repeat(80);
    let src = format!(
        "module tb; localparam string S = \"{}\"; \
         initial $display(\"%0d %s\", S.len(), S); endmodule",
        long
    );
    let o = out(&src);
    assert!(o.contains(&format!("160 {}", long)), "{}", o);
}
