//! §23.3.2: an instance connects its ports all by position or all by name.
//! The reference simulator rejects the mixed lists too and runs the legal
//! instances with the same output.

use xezim::simulate;

#[test]
fn positional_and_named_connections_mixed() {
    for conns in ["(.a(w1), w2)", "(w1, .b(w2))", "(w1, .*)"] {
        let src = format!(
            "module m(input a, b); endmodule\n\
             module test; wire w1, w2; m u{conns}; endmodule\n"
        );
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn uniform_connection_lists() {
    let src = r#"
module m(input a, b, output y);
  assign y = a & b;
endmodule
module test;
  reg a = 1, b = 1;
  wire y1, y2, y3, y4;
  m u1(a, b, y1);
  m u2(.a(a), .b(b), .y(y2));
  m u3(.a(a), .*, .y(y3));
  m u4(a, , y4);
  initial #1 $display("C|%b %b %b %b", y1, y2, y3, y4);
endmodule
"#;
    let sim = simulate(src, 10).expect("uniform connection lists are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "C|1 1 1 x"),
        "{:?}",
        sim.output
    );
}
