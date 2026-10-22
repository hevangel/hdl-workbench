//! §10.3 vs §28.4 — a continuous assignment passes `z` through unchanged; a
//! gate PRIMITIVE's truth table maps a `z` input to `x`.
//!
//! Both lower to the same one-bit copy, and the optimizer fuses that copy into
//! a buffer op whose execution applied the GATE rule unconditionally ("Z
//! treated as X when used as a wire value") — so every scalar `assign y = x;`
//! silently ate `z`. A z-propagation testbench failed on the very first
//! sample: the DUT input chain turned all-z stimulus into x one hop in.
//!
//! The fix records which nets a lowered gate primitive drives
//! (`ElaboratedModule::gate_driven_nets` → `signal_gate_driven`) and applies
//! the z→x mapping only to those, in all three fused-buffer execution paths
//! (serial, isolated/parallel, and the shared-source fanout group).

use xezim::simulate;

fn bits(sim: &xezim::compiler::Simulator, n: &str) -> String {
    let v = sim
        .get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n));
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

/// The same z-valued variable driven through an `assign` and through `buf` /
/// `not` primitives — only the primitives may produce `x`.
#[test]
fn continuous_assign_passes_z_but_gates_map_it_to_x() {
    let src = r#"
module top;
  logic v;
  wire  as_copy;  assign as_copy = v;
  wire  buf_out;  buf b1(buf_out, v);
  wire  not_out;  not n1(not_out, v);
  initial begin
    v = 1'bz;
    #2;
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(
        bits(&sim, "as_copy"),
        "z",
        "assign must pass z through (§10.3)"
    );
    assert_eq!(
        bits(&sim, "buf_out"),
        "x",
        "buf primitive maps z to x (§28.4)"
    );
    assert_eq!(
        bits(&sim, "not_out"),
        "x",
        "not primitive maps z to x (§28.4)"
    );
}

/// Chained scalar assigns — the shape the buf-fanout fusion groups — keep `z`
/// end to end, and recover a driven value when the source leaves `z`.
#[test]
fn z_survives_a_chain_of_scalar_assigns_and_recovers() {
    let src = r#"
module top;
  logic v;
  wire a; assign a = v;
  wire b; assign b = a;
  wire c; assign c = b;
  wire d; assign d = v;   // 4+ copies of one source triggers fanout grouping
  wire e; assign e = v;
  wire f; assign f = v;
  wire g; assign g = v;
  initial begin
    v = 1'bz;
    #2 v = 1'b1;
    #2 v = 1'bz;
    #2;
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    for n in ["a", "b", "c", "d", "e", "f", "g"] {
        assert_eq!(
            bits(&sim, n),
            "z",
            "{n} must read z after the source returns to z"
        );
    }
}

/// Known values keep flowing through gates — the gate rule is only narrowed,
/// not removed.
#[test]
fn gates_still_drive_known_values() {
    let src = r#"
module top;
  logic v;
  wire  buf_out;  buf b1(buf_out, v);
  wire  not_out;  not n1(not_out, v);
  initial begin
    v = 1'b1;
    #2;
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(bits(&sim, "buf_out"), "1");
    assert_eq!(bits(&sim, "not_out"), "0");
}
