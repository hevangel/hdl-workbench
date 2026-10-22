//! IEEE 1800 §27.6: a continuous assignment that reads a variable of an
//! UNNAMED generate block through its implicit `genblk<N>` name re-evaluates
//! when that variable changes.

fn check(source: &str) {
    let result = xezim::simulate(source, 100_000).unwrap();
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .collect();
    assert!(
        !lines.iter().any(|line| line.starts_with("FAIL")),
        "{lines:?}"
    );
    assert!(lines.contains(&"PASS"), "{lines:?}");
}

/// §27.6: `u_mod.genblk1.x` on a continuous-assign RHS. The unnamed branch
/// keeps its declaration under the bare name (`u_mod.x`); the continuous
/// assignment must depend on that same signal (not the `u_mod` instance),
/// or it evaluates once and stays X when `x` later changes.
#[test]
fn cont_assign_reads_genblk_signal() {
    check(
        r#"module sub;
  if (1) begin            // unnamed: genblk1
    logic [3:0] x;
    initial begin x = 5; #10 x = 9; end
  end
  if (1) begin : g        // named
    logic [3:0] y;
    initial begin y = 5; #10 y = 9; end
  end
endmodule

module top;
  sub u ();
  wire [3:0] vx = u.genblk1.x;
  wire [3:0] vy = u.g.y;
  initial begin
    #5;
    if (vx !== 4'd5 || vy !== 4'd5) $display("FAIL t=5 vx=%0d vy=%0d (direct %0d %0d)", vx, vy, u.genblk1.x, u.g.y);
    #10;
    if (vx !== 4'd9 || vy !== 4'd9) $display("FAIL t=15 vx=%0d vy=%0d (direct %0d %0d)", vx, vy, u.genblk1.x, u.g.y);
    else $display("PASS");
    $finish;
  end
endmodule"#,
    );
}
