//! §A.1.4/§27: a generate item is a `module_or_generate_item`, which excludes
//! `specparam` and module declarations (ivtest `generate_specparam`,
//! `sv_generate_module`; the reference simulator reports a syntax error for
//! both). Once `specparam` was parsed as a localparam and nested modules
//! were hoisted, both forms were silently accepted inside a generate block.

use xezim::simulate;

#[test]
fn specparam_inside_generate_is_rejected() {
    let src = "module test #(parameter A = 1);\n\
               generate if (A) begin specparam x = 10; end endgenerate\n\
               endmodule\n";
    assert!(simulate(src, 10).is_err());
}

#[test]
fn module_declaration_inside_generate_is_rejected() {
    let src = "module test #(parameter A = 1);\n\
               generate if (A) begin\n\
               module inner; initial $display(\"FAILED\"); endmodule\n\
               end endgenerate\n\
               endmodule\n";
    assert!(simulate(src, 10).is_err());
}

#[test]
fn module_level_specparam_and_nested_module_still_parse() {
    let src = "module test;\n\
               specparam D = 3;\n\
               module inner; endmodule\n\
               inner u();\n\
               initial $display(\"S|%0d\", D);\n\
               endmodule\n";
    let sim = simulate(src, 10).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "S|3"));
}
