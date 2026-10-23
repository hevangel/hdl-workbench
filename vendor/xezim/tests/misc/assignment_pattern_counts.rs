//! §10.9: an ordered assignment pattern has one item per struct member, or
//! per element of a packed array's outer dimension. The reference simulator
//! rejects the mismatched counts at elaboration and runs the legal module
//! with the same output.

use xezim::simulate;

#[test]
fn pattern_count_mismatch() {
    for decl in [
        "bit [2:0][3:0] x = '{1, 2};",
        "bit [2:0][3:0] x = '{1, 2, 3, 4};",
        "bit [3:0] x = '{1, 0, 1};",
        "struct packed { int x; shortint y; byte z; } x = '{1, 2};",
        "struct packed { int x; shortint y; byte z; } x = '{1, 2, 3, 4};",
        "struct { int a; int b; int c; } x = '{1, 2};",
        "typedef struct packed { int a; shortint b; } s_t;\n  s_t x = '{1, 2, 3};",
    ] {
        let src = format!("module test;\n  {decl}\n  initial $display(\"x\");\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn pattern_counts_matching() {
    let src = r#"
module test;
  bit [3:0] x = '{1, 0, 1, 1};
  bit [1:0][3:0] y = '{4'h1, 4'h2};
  struct packed { int a, b; byte c; } s = '{1, 2, 3};
  initial $display("A|%b %h %0d %0d %0d", x, y, s.a, s.b, s.c);
endmodule
"#;
    let sim = simulate(src, 10).expect("matching patterns are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "A|1011 12 1 2 3"),
        "{:?}",
        sim.output
    );
}
