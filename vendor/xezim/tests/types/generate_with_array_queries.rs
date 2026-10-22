//! IEEE 1800-2023 §27.5 / §27.4: a generate `if` / `case` condition and a
//! generate `for` loop's bounds are constant expressions, and §20.6.2 / §20.7
//! `$bits` and the array queries of a fixed-size variable are constant. A
//! condition such as `if ($size(fx) - 5 < 0)` was rejected with "Generate if
//! condition must be a constant expression", which the reference simulator
//! accepts.
//!
//! The genvar initializer is a constant expression too, but only a plain
//! literal was folded: `for (genvar i = P; ...)`, `i = -2` and `i = $low(a)`
//! all started at 0. Every expected value below comes from the reference
//! simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let mut v: Vec<String> = simulate(src, 100)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect();
    v.sort();
    v
}

fn sorted(lines: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

/// The reproducer, plus `if`, `else if`, `case` and `for` over the queries.
#[test]
fn generate_conditions_and_bounds_over_array_queries() {
    let got = t_lines(
        r#"
module top;
  int fx[4];
  int fx2[2][3];
  logic [7:0] pu [0:2][5:1];
  typedef byte bt [3];
  bt tb;
  byte dd [3:-2];
  generate if ($size(fx) - 5 < 0) begin : g1
    initial $display("T|G1 neg");
  end else begin : g2
    initial $display("T|G1 nonneg");
  end endgenerate
  if ($bits(fx) == 128) begin : g3
    initial $display("T|G3 yes");
  end else begin : g4
    initial $display("T|G3 no");
  end
  if ($unpacked_dimensions(fx2) != 2) begin : g5
    initial $display("T|G5 bad");
  end else if ($left(pu, 2) == 5) begin : g6
    initial $display("T|G5 left5");
  end
  case ($size(fx2, 2))
    2: begin : c2 initial $display("T|C 2"); end
    3: begin : c3 initial $display("T|C 3"); end
    default: begin : cd initial $display("T|C dflt"); end
  endcase
  case ($bits(tb))
    24: begin : d24 initial $display("T|D 24"); end
    default: begin : dd0 initial $display("T|D dflt"); end
  endcase
  for (genvar i = $right(pu, 2); i <= $left(pu, 2); i++) begin : fl
    initial $display("T|F %0d", i);
  end
  for (genvar j = 0; j < $dimensions(pu); j++) begin : fl2
    initial $display("T|F2 %0d", j);
  end
  genvar gi;
  for (gi = 0; gi < $size(fx); gi++) begin : gl
    initial $display("T|GL %0d", gi);
  end
  for (gi = $low(dd); gi <= $high(dd); gi++) begin : gl2
    localparam int K = gi;
    initial $display("T|GL2 %0d", K);
  end
  generate if ($increment(pu, 2) > 0) begin : ia initial $display("T|INC up"); end
  else begin : ib initial $display("T|INC down"); end endgenerate
  if (1) begin : outer
    localparam K = $size(fx2, 2);
    if ($bits(fx2) == 192) begin : inner
      initial $display("T|NEST %0d", K);
    end
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        sorted(&[
            "T|G1 neg",
            "T|G3 yes",
            "T|G5 left5",
            "T|C 3",
            "T|D 24",
            "T|F 1",
            "T|F 2",
            "T|F 3",
            "T|F 4",
            "T|F 5",
            "T|F2 0",
            "T|F2 1",
            "T|F2 2",
            "T|GL 0",
            "T|GL 1",
            "T|GL 2",
            "T|GL 3",
            "T|GL2 -2",
            "T|GL2 -1",
            "T|GL2 0",
            "T|GL2 1",
            "T|GL2 2",
            "T|GL2 3",
            "T|INC up",
            "T|NEST 3",
        ]),
    );
}

/// Generate constructs over an instance's own arrays, each instance sized by
/// its own parameter.
#[test]
fn generate_over_array_queries_in_instances() {
    let got = t_lines(
        r#"
module sub #(parameter int N = 4) ();
  typedef struct packed { logic [2:0] a; logic [4:0] b; } st_t;
  int arr [N][2];
  st_t sarr [N:1];
  if ($size(arr) > 4) begin : big
    initial $display("T|SUB%0d big", N);
  end else begin : notbig
    initial $display("T|SUB%0d notbig", N);
  end
  for (genvar k = $low(sarr); k <= $high(sarr); k++) begin : lp
    initial $display("T|SUB%0d k %0d", N, k);
  end
endmodule
module top;
  sub #(.N(3)) u3 ();
  sub #(.N(6)) u6 ();
endmodule
"#,
    );
    assert_eq!(
        got,
        sorted(&[
            "T|SUB3 notbig",
            "T|SUB3 k 1",
            "T|SUB3 k 2",
            "T|SUB3 k 3",
            "T|SUB6 big",
            "T|SUB6 k 1",
            "T|SUB6 k 2",
            "T|SUB6 k 3",
            "T|SUB6 k 4",
            "T|SUB6 k 5",
            "T|SUB6 k 6",
        ]),
    );
}

/// §27.4: the genvar initializer — a parameter, an expression, a negative
/// number — at module scope and inside an instance.
#[test]
fn genvar_initializer_is_a_constant_expression() {
    let got = t_lines(
        r#"
module sub #(parameter int S = 1) ();
  for (genvar gi = S; gi < 3; gi++) begin : g
    initial $display("T|SUB %0d", gi);
  end
endmodule
module top;
  localparam int P = 1;
  localparam int D = -2;
  for (genvar gi = P; gi < 3; gi++) begin : g1
    initial $display("T|P %0d", gi);
  end
  for (genvar gi = P + 0; gi < 3; gi++) begin : g2
    initial $display("T|P2 %0d", gi);
  end
  for (genvar gi = -2; gi < 0; gi++) begin : g3
    initial $display("T|N %0d", gi);
  end
  for (genvar gi = D; gi < 0; gi++) begin : g4
    initial $display("T|D %0d", gi);
  end
  sub #(.S(2)) u ();
endmodule
"#,
    );
    assert_eq!(
        got,
        sorted(&[
            "T|P 1", "T|P 2", "T|P2 1", "T|P2 2", "T|N -2", "T|N -1", "T|D -2", "T|D -1",
            "T|SUB 2",
        ]),
    );
}
