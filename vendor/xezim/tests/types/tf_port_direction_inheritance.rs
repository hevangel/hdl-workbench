//! §13.5.2: a task/function formal whose direction is omitted takes the
//! direction of the PREVIOUS formal; only the first defaults to `input`.
//!
//! The parser defaulted every direction-less formal to input, so
//! `output logic [7:0] r0, r1, r2, r3` declared r1..r3 as INPUTS. Assignments
//! to them inside the body landed in call-frame locals and the copy-out never
//! happened — the caller saw only the FIRST result of any multi-output
//! helper. Found via an AES core whose MixColumns updated exactly one byte
//! per column: the ciphertext was wrong while every individual step that
//! used single-output helpers checked out.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
module top;
  function automatic void f(input logic [7:0] a, output logic [7:0] x, y, z);
    x = a + 8'd1;
    y = a + 8'd2;
    z = a + 8'd3;
  endfunction

  task automatic t(input logic [7:0] a, output logic [7:0] x, y);
    x = a + 8'd10;
    y = a + 8'd20;
  endtask

  // Direction continues across a TYPE change too: c/d are still inputs.
  function automatic logic [7:0] g(input logic [7:0] a, b, logic [3:0] c, d);
    g = a + b + {4'd0, c} + {4'd0, d};
  endfunction

  logic [7:0] m, n, o, p, q;
  initial begin
    f(8'd5, m, n, o);
    $display("NOTE: fn m=%0d n=%0d o=%0d", m, n, o);
    t(8'd5, p, q);
    $display("NOTE: task p=%0d q=%0d", p, q);
    $display("NOTE: g=%0d", g(8'd1, 8'd2, 4'd3, 4'd4));
    $finish;
  end
endmodule
"#;

#[test]
fn omitted_formal_direction_inherits_from_previous() {
    assert_eq!(
        notes(SRC),
        vec!["NOTE: fn m=6 n=7 o=8", "NOTE: task p=15 q=25", "NOTE: g=10",],
        "r1..rN of a comma-continued output list must be OUTPUTS (§13.5.2)"
    );
}
