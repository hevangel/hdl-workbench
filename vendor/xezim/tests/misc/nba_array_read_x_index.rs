//! §11.5.1: a non-blocking array read with an x/z index (`q <= mem[addr]`
//! while `addr` is x, the idle state of a synchronous RAM's address input)
//! queues an all-x element. The 4-state executor read element 0 (its index
//! went through `to_u64`, which masks x to 0) and the two-state path bailed
//! the whole block back to the interpreter on every clock edge.
use xezim::simulate;

const SRC: &str = r#"
module top;
  logic clk = 0; always #5 clk = ~clk;
  logic [7:0] mem [0:3]; logic [1:0] addr; logic cen = 0; logic [7:0] d = 8'h5A, q;
  logic [7:0] q_x, q_2, q_xagain;
  always @(posedge clk) begin if (cen) mem[addr] <= d; q <= mem[addr]; end
  initial begin
    mem[0] = 8'h11; mem[1] = 8'h22; mem[2] = 8'h33; mem[3] = 8'h44;
    @(posedge clk); #1 q_x = q;
    addr = 2; @(posedge clk); #1 q_2 = q;
    addr = 'x; @(posedge clk); #1 q_xagain = q;
  end
endmodule
"#;

fn bits(sim: &xezim::compiler::Simulator, n: &str) -> String {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("{n}"));
    (0..v.width as usize)
        .rev()
        .map(|i| match v.get_bit(i) {
            xezim_core::value::LogicBit::Zero => '0',
            xezim_core::value::LogicBit::One => '1',
            xezim_core::value::LogicBit::X => 'x',
            xezim_core::value::LogicBit::Z => 'z',
        })
        .collect()
}

#[test]
fn nba_array_read_with_x_index_is_x() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        bits(&sim, "q_x"),
        "xxxxxxxx",
        "x address before any assignment"
    );
    assert_eq!(
        bits(&sim, "q_2"),
        "00110011",
        "known address reads the element"
    );
    assert_eq!(bits(&sim, "q_xagain"), "xxxxxxxx", "x address again");
}
