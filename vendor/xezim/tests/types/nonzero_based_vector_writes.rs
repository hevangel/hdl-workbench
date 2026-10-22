//! §7.4.1 — writes to part-selects and bit-selects of a NON-ZERO-BASED packed
//! vector (`logic [3:1] w`). Reference-validated.
//!
//! The bytecode compiler's WRITE emission used declared indices as raw
//! physical offsets — `w[2:1] = v` landed at offsets 2:1 (declared 3:2)
//! instead of 1:0, so the write sat one position high and declared bit 1
//! stayed x forever. The READ path had the rebase all along, which made the
//! corruption self-consistent and hard to spot: reading back the same select
//! returned what was written; only the WHOLE vector (or the neighbours)
//! showed the shift.
//!
//! Only COMPILED paths were affected (continuous assigns, and always blocks
//! that compile to bytecode). The AST interpreter normalized correctly —
//! which is why simple procedural probes passed while the same expression in
//! an `assign` failed. The field shape: a submodule's output port connected
//! to `parent_status[2:1]` where the parent declares `logic [3:1]` — the
//! port connection inlines to a continuous assign, and the status register
//! check read x after reset.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

/// Every write form: continuous const range, output-port connection,
/// NBA const range, NBA dynamic bit, NBA indexed-up.
#[test]
fn writes_land_at_declared_positions() {
    let src = r#"
module leaf(output logic [1:0] o2, output logic o1);
  assign o2 = 2'b10;
  assign o1 = 1'b1;
endmodule
module tb;
  logic clk = 0;
  logic [3:1] cont, port, nba_r, nba_b, nba_iu;
  int i;
  always #5 clk = ~clk;
  assign cont[2:1] = 2'b10;
  leaf u(.o2(port[2:1]), .o1(port[3]));
  always @(posedge clk) nba_r[2:1] <= 2'b10;
  always @(posedge clk) nba_b[i]   <= 1'b1;
  always @(posedge clk) nba_iu[i+:2] <= 2'b11;
  int r_cont, r_port, r_nr, r_nb, r_niu;
  initial begin
    i = 2; nba_r = '0; nba_b = '0; nba_iu = '0;
    @(posedge clk); #1;
    // reading [3:1] as a 3-bit value: {bit3, bit2, bit1}
    r_cont = {1'b0, cont[2:1]};
    r_port = port;
    r_nr  = nba_r;
    r_nb  = nba_b;
    r_niu = nba_iu;
  end
endmodule
"#;
    let sim = simulate(src, 30).expect("simulate failed");
    assert_eq!(u(&sim, "r_cont"), 0b010, "continuous const range write");
    assert_eq!(u(&sim, "r_port"), 0b110, "output-port part+bit connection");
    assert_eq!(u(&sim, "r_nr"), 0b010, "NBA const range write");
    assert_eq!(
        u(&sim, "r_nb"),
        0b010,
        "NBA dynamic bit write (declared idx 2 = offset 1)"
    );
    assert_eq!(u(&sim, "r_niu"), 0b110, "NBA indexed-up write");
}
