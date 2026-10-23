//! §26.3: a name that two wildcard-imported packages both declare is visible
//! through neither import; using it needs a local declaration or an explicit
//! import. The reference simulator rejects the ambiguous uses and runs the
//! legal module with the same output.

use xezim::simulate;

const PKGS: &str = "package p1;\n  parameter p = 1;\n  typedef logic [1:0] word;\n  \
                    function int f(int g); return g + 1; endfunction\nendpackage\n\
                    package p2;\n  parameter p = 2;\n  typedef logic [2:0] word;\n  \
                    function int f(int g); return g + 2; endfunction\nendpackage\n";

#[test]
fn ambiguous_wildcard_imports() {
    for body in [
        "word my_v;",
        "initial $display(\"%0d\", p);",
        "initial $display(\"%0d\", f(1));",
    ] {
        let src =
            format!("{PKGS}module test;\n  import p1::*;\n  import p2::*;\n  {body}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn resolved_wildcard_imports() {
    let src = format!(
        "{PKGS}{}",
        r#"module test;
  import p1::*;
  import p2::*;
  import p2::word;
  localparam p = 7;
  word w = 3'd5;
  initial $display("W|%0d %0d %0d", w, p, p1::f(1));
endmodule
"#
    );
    let sim = simulate(&src, 10).expect("resolved names are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "W|5 7 2"),
        "{:?}",
        sim.output
    );
}
