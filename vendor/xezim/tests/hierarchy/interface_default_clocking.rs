//! §14.12 — a standalone `default clocking <name>;` inside an INTERFACE.
//! Reference-validated.
//!
//! The designation parses as an empty, clock-less `ClockingDeclaration` that
//! reuses the block's name. Both module-level elaboration paths fold it into
//! the real same-named block; the interface-INSTANCE path had no such guard and
//! inserted it unconditionally, overwriting the real block with the marker.
//! With no clock signal the simulator then skipped it entirely, so every
//! clocking access through that interface — drives and samples alike — became
//! x. One innocuous and very common line disabled the whole block, while the
//! identical construct in a module was fine.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn default_clocking_designation_keeps_the_block_alive() {
    let src = r#"
interface bus_if(input logic clk);
  logic [7:0] data;
  logic [7:0] q;
  clocking cb @(posedge clk);
    default input #1step output #0;
    output data;
    input  q;
  endclocking
  default clocking cb;          // §14.12 designation — must not clobber `cb`
endinterface
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if a0(clk);
  int drove, sampled;
  initial begin
    a0.q = 8'hA5;
    @(posedge clk);
    a0.cb.data <= 8'h6D;
    @(posedge clk);
    #1;
    drove   = a0.data;
    sampled = a0.cb.q;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "drove"),
        0x6D,
        "a clocking-block drive still reaches the signal"
    );
    assert_eq!(
        u(&sim, "sampled"),
        0xA5,
        "and a clocking-block input still samples"
    );
}
