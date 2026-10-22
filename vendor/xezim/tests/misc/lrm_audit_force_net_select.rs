//! IEEE 1800-2023 §10.6.2: `force` / `release` of a bit- or part-select of
//! a net. The forced bits hold against the net's drivers (also against a
//! driver change in the same time step), and a release hands just those
//! bits back to the drivers. Such a force used to degrade to a plain write.
//! Expected values come from the reference simulator.
use xezim::simulate;

fn t_lines(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

/// The audit repro.
#[test]
fn force_release_net_bits_audit_repro() {
    const SRC: &str = r#"
module rfb;
  logic [3:0] a = 4'hf, b = 4'h0;
  wire [7:0] w;
  assign w = {a, b};
  initial begin
    #1 force w[0] = 1'b1;
    force w[7:6] = 2'b00;
    #1 $display("T|r1|w=%b", w);
    release w[0];
    #1 $display("T|r2|w=%b", w);
    release w[7:6];
    #1 $display("T|r3|w=%b", w);
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        ["T|r1|w=00110001", "T|r2|w=00110000", "T|r3|w=11110000",],
    );
}

/// The audit repro for the same time step.
#[test]
fn force_net_bit_against_same_step_driver_change() {
    const SRC: &str = r#"
module ffb2;
  logic [3:0] a = 0, b = 0;
  wire [7:0] w;
  assign w = {a, b};
  initial begin
    #1 force w[0] = 1'b1;
    a = 4'hf;
    #1 $display("T|10.6.2|w=%b", w);
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(t_lines(&sim), ["T|10.6.2|w=11110001",],);
}

/// An expression right-hand side that keeps tracking, non-zero-based and
/// ascending ranges, an indexed part-select, a whole-net force over a bit
/// force, overlapping forces and partial releases.
#[test]
fn force_net_select_shapes() {
    const SRC: &str = r#"
module frc1;
  logic [3:0] a = 4'h0, b = 4'h0;
  wire [7:0] w; assign w = {a, b};
  wire [15:8] h; assign h = {a, b};
  wire [0:7] u; assign u = {a, b};
  logic [1:0] s = 2'b10;
  wire [7:0] m; assign m = {a, b};
  initial begin
    #1 force w[3:2] = s;          // expression RHS
    force h[9] = 1'b1;            // non-zero LSB
    force u[0:1] = 2'b11;         // ascending: MSB end
    force m[2 +: 3] = 3'b101;     // indexed part-select
    #1 $display("T|r1|w=%b h=%b u=%b m=%b", w, h, u, m);
    s = 2'b01; a = 4'h5;
    #1 $display("T|r2|w=%b h=%b u=%b m=%b", w, h, u, m);
    force w = 8'hff;              // whole force replaces the bit force
    #1 $display("T|r3|w=%b", w);
    release w;
    #1 $display("T|r4|w=%b", w);
    force w[0] = 1'b1;
    force w[1:0] = 2'b00;         // overlapping later force wins
    #1 $display("T|r5|w=%b", w);
    release w[1];
    b = 4'hf;
    #1 $display("T|r6|w=%b", w);
    release h[9]; release u[0:1]; release m[2 +: 3]; release w[3:2];
    #1 $display("T|r7|w=%b h=%b u=%b m=%b", w, h, u, m);
    release w[0];
    #1 $display("T|r8|w=%b", w);
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|r1|w=00001000 h=00000010 u=11000000 m=00010100",
            "T|r2|w=01010100 h=01010010 u=11010000 m=01010100",
            "T|r3|w=11111111",
            "T|r4|w=01010000",
            "T|r5|w=01010000",
            "T|r6|w=01011110",
            "T|r7|w=01011110 h=01011111 u=01011111 m=01011111",
            "T|r8|w=01011111",
        ],
    );
}
