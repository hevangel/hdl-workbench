//! §26.6: `export P::n` exports a name the package imported from `P` and does
//! not declare itself. Each rejected case is also rejected by the reference
//! simulator, and the legal design prints the same line there.

use xezim::simulate;

#[test]
fn illegal_package_exports() {
    for src in [
        "package P1; integer x; integer y; endpackage\n\
         package P2; import P1::x; export P1::y; endpackage\nmodule test; endmodule",
        "package P1; integer x = 123; endpackage\n\
         package P2; import P1::*; integer x = 456; export P1::x; endpackage\n\
         module test; import P2::x; endmodule",
    ] {
        let e = simulate(src, 10)
            .err()
            .expect("an illegal export was accepted");
        assert!(e.contains("§26.6"), "{e}");
    }
}

#[test]
fn legal_package_exports() {
    let src = r#"
package P1; integer x = 3; integer y = 4; endpackage
package P2; import P1::x; export P1::x; import P1::*; export P1::y; endpackage
package P3; import P1::*; export *::*; endpackage
module test;
  import P1::*;
  initial $display("E|%0d %0d", x, y);
endmodule
"#;
    let sim = simulate(src, 10).expect("legal exports must run");
    assert!(
        sim.output.iter().any(|o| o.message == "E|3 4"),
        "{:?}",
        sim.output
    );
}
