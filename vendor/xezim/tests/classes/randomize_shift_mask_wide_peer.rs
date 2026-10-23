//! Shift/mask equality constraints on a rand scalar fail whenever the
//! class also carries a >64-bit rand member (issue #261): the joint CSP
//! refuses the whole class on width, and the solution set of `(x >> 8)==0`
//! / `(x & 'hffff_ff00)==0` is a 2^-24 sliver generate-and-test cannot
//! reach. Forcing the rand scalar directly (shift: uniform in
//! `[c<<k, c<<k + 2^k)`, mask: OR random bits into the holes) covers any
//! width, including a >64-bit constrained target. Reference-verified.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

const SRC: &str = r#"
module top;
  class narrow_only;
    rand bit [31:0] x;
    rand bit [63:0] y;
    constraint k { (x >> 8) == 0; }
  endclass
  class with_wide;
    rand bit [31:0] x;
    rand bit [64:0] y;   // one bit wider than narrow_only::y; never constrained
    constraint k { (x >> 8) == 0; }
  endclass
  class with_wide_mask;
    rand bit [31:0] x;
    rand bit [64:0] y;
    constraint k { (x & 32'hffff_ff00) == 0; }
  endclass
  class wide_x;
    rand bit [127:0] x;
    constraint k { (x >> 8) == 0; }
  endclass

  initial begin
    narrow_only a = new();
    with_wide b = new();
    with_wide_mask c = new();
    wide_x d = new();
    if (a.randomize() == 1 && a.x < 32'h100) $display("A_OK");
    else $display("A_FAIL");
    if (b.randomize() == 1 && b.x < 32'h100) $display("B_OK");
    else $display("B_FAIL");
    if (c.randomize() == 1 && c.x < 32'h100) $display("C_OK");
    else $display("C_FAIL");
    if (d.randomize() == 1 && d.x < 128'h100) $display("D_OK");
    else $display("D_FAIL");
  end
endmodule
"#;

#[test]
fn shift_mask_constraints_with_wide_rand_peer() {
    let msgs = messages(SRC);
    for want in ["A_OK", "B_OK", "C_OK", "D_OK"] {
        assert!(
            msgs.iter().any(|m| m == want),
            "missing {want}, got {msgs:?}"
        );
    }
    assert!(
        !msgs.iter().any(|m| m.ends_with("_FAIL")),
        "no FAIL expected, got {msgs:?}"
    );
}
