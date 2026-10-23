//! §6.7.1: a net's data type must be 4-state integral (or an aggregate of
//! such); a class handle can never be a net (ivtest `net_class_fail`; the
//! reference simulator reports "Illegal net data type").

use xezim::simulate;

#[test]
fn net_of_class_type_is_rejected() {
    let src =
        "module test; class C; endclass\n wire C x;\n initial $display(\"FAILED\");\nendmodule\n";
    assert!(simulate(src, 10).is_err());
}

#[test]
fn string_net_is_rejected() {
    assert!(simulate("module test; wire string s; endmodule", 10).is_err());
}

#[test]
fn integral_nets_still_elaborate() {
    let src = "module test; class C; endclass\n wire [3:0] x = 4'd9; wire logic y = 1'b1;\n\
               initial #1 $display(\"N|%0d %b\", x, y);\nendmodule\n";
    let sim = simulate(src, 10).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "N|9 1"));
}
