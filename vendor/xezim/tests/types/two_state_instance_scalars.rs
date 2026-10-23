//! §6.11.1: a 2-state variable declared in an INSTANTIATED module drops x/z
//! on every write, like the same declaration at top level. The instance
//! path registered 2-state arrays and ports but not scalars, so `int`,
//! `bit`, `byte`, `shortint` and `longint` variables of a submodule held
//! x/z; a write through a hierarchical name from a task kept it too.
//! Reference-validated.

use xezim::simulate;

fn bits(sim: &xezim::compiler::Simulator, n: &str) -> String {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_bin_string()
}

#[test]
fn instance_two_state_scalars_drop_xz() {
    let src = r#"
module sub;
  int iv; bit [3:0] b4; logic [3:0] l4; byte by; shortint si; longint li;
  initial begin
    #1; iv = 'x; b4 = 4'bx1z0; l4 = 4'bx1z0; by = 8'hzz; si = 'x; li = 'z;
  end
endmodule
module tb;
  sub s();
  task w(); s.by = 8'bzx11_0000; endtask
  initial begin
    #2 s.iv = 8'hxz;
    #1 w();
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    for (n, want) in [
        ("s.iv", "00000000000000000000000000000000"),
        ("s.b4", "0100"),
        ("s.by", "00110000"),
        ("s.si", "0000000000000000"),
    ] {
        assert_eq!(bits(&sim, n), want, "{}", n);
    }
    assert_eq!(bits(&sim, "s.li"), "0".repeat(64), "longint holds no z");
    assert_eq!(bits(&sim, "s.l4"), "x1z0", "a 4-state variable keeps x/z");
}

/// §6.8: an `output bit` port of an unconnected interface instance is a
/// 2-state variable, 0 at time 0 — a clock toggled from it runs. It came up
/// x (the 4-state default of an output), so `~clk` stayed x forever.
/// Reference-validated.
#[test]
fn unconnected_output_bit_port_starts_at_zero() {
    let src = r#"
interface adder_if(output bit clk);
endinterface
module tb;
  adder_if dif();
  bit s0, s60, s110;
  initial begin
    s0 = dif.clk;
    #60 s60 = dif.clk;
    #50 s110 = dif.clk;
    $finish;
  end
  initial forever #50 dif.clk = ~dif.clk;
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    assert_eq!(bits(&sim, "s0"), "0");
    assert_eq!(bits(&sim, "s60"), "1");
    assert_eq!(bits(&sim, "s110"), "0");
}
