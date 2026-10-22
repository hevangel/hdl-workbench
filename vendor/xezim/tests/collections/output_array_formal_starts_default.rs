//! IEEE 1800-2023 §13.5.1: an `output` unpacked-array formal is not copied
//! in. It starts at its element type's default (0 for a 2-state element, x
//! for a 4-state one) and the whole formal is copied out on return, so an
//! element the body leaves alone resets the caller's element. The formal
//! used to start as a copy of the actual, so those elements kept the
//! caller's old values. Expected lines are the reference simulator's output.

use xezim::simulate;

#[test]
fn output_array_formal_starts_at_default() {
    let out: Vec<String> = simulate(
        r#"
class C;
  int M[2][2];
  int F[3];
  function void fm(output int p[2][2]); p[1][0] = 7; endfunction
  function void ff(output int p[3]); p[0] = 1; endfunction
  function void run(); M[0][1] = 3; F[2] = 9; fm(M); ff(F); $display("T| o1 m M %0d %0d F %0d %0d", M[1][0], M[0][1], F[0], F[2]); endfunction
endclass
module top;
  int G[3];
  int H[2][2];
  logic [3:0] L[2];
  function automatic void ff(output int p[3]); p[0] = 1; endfunction
  function automatic void fh(output int p[2][2]); p[0][0] = 1; endfunction
  function automatic void fl(output logic [3:0] p[2]); p[0] = 1; endfunction
  task automatic tf(output int p[3]); p[0] = 1; endtask
  initial begin
    C c = new;
    G = '{5, 6, 7}; ff(G); $display("T| o1 f G %0d %0d %0d", G[0], G[1], G[2]);
    G = '{5, 6, 7}; tf(G); $display("T| o1 t G %0d %0d %0d", G[0], G[1], G[2]);
    H[1][1] = 4; fh(H); $display("T| o1 H %0d %0d", H[0][0], H[1][1]);
    L[1] = 3; fl(L); $display("T| o1 L %b %b", L[0], L[1]);
    c.run();
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed")
    .output
    .iter()
    .map(|o| o.message.clone())
    .filter(|m| m.starts_with("T|"))
    .collect();
    let want = [
        "T| o1 f G 1 0 0",
        "T| o1 t G 1 0 0",
        "T| o1 H 1 0",
        "T| o1 L 0001 xxxx",
        "T| o1 m M 7 0 F 1 0",
    ];
    assert_eq!(out, want, "{out:?}");
}
