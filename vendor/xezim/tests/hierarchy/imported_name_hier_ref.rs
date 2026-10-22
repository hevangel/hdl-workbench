//! §26.3: a name a module only imports is not declared in it, so a
//! hierarchical reference through an instance cannot reach it. The reference
//! simulator rejects `m.x` for an imported `x` and runs the legal module with
//! the same output.

use xezim::simulate;

#[test]
fn hierarchical_reference_to_imported_name() {
    for import in ["import P::x;", "import P::*;"] {
        let src = format!(
            "package P;\n  integer x;\nendpackage\n\
             module M;\n  {import}\n  integer y;\n  always_comb y = x;\nendmodule\n\
             module test;\n  M m ();\n  initial begin\n    integer y;\n    y = m.x;\n  end\nendmodule\n"
        );
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn hierarchical_reference_to_declared_name() {
    let src = r#"
package P;
  integer x = 4;
endpackage
module M;
  import P::*;
  integer y;
  always_comb y = x + 1;
endmodule
module test;
  M m ();
  initial #1 $display("H|%0d %0d", m.y, P::x);
endmodule
"#;
    let sim = simulate(src, 10).expect("declared names are reachable");
    assert!(
        sim.output.iter().any(|o| o.message == "H|5 4"),
        "{:?}",
        sim.output
    );
}
