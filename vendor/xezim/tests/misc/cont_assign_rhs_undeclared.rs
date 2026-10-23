//! §6.10: an undeclared name implies a net only as the target of a
//! continuous assignment (or a port connection). Read on the right-hand side
//! it is undeclared. The reference simulator rejects those reads and runs the
//! legal module with the same output.

use xezim::simulate;

#[test]
fn undeclared_rhs_names() {
    for rhs in ["missing", "dup(missing)", "{missing, a}", "a ^ missing"] {
        let src = format!(
            "module test;\n  function [7:0] dup(input [7:0] i); dup = i; endfunction\n  \
             wire [7:0] a, b;\n  assign b = {rhs};\nendmodule\n"
        );
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn implicit_nets_from_targets_and_ports() {
    let src = r#"
module inv(input i, output o);
  assign o = ~i;
endmodule
module test;
  reg r = 0;
  assign n1 = r;
  inv u(.i(n1), .o(n2));
  assign n3 = n2 & n1 | n2;
  initial #1 $display("N|%b %b %b", n1, n2, n3);
endmodule
"#;
    let sim = simulate(src, 10).expect("implicit nets from targets are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "N|0 1 1"),
        "{:?}",
        sim.output
    );
}
