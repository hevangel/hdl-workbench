//! A continuous assign to a packed-struct MEMBER inside an instantiated
//! submodule (`assign s.m0 = v;`) must splice the driven bits into the whole
//! struct signal and re-trigger when the (scoped) input changes.
//!
//! After submodule inlining the member LHS becomes a scope-qualified 2-segment
//! `Ident(["dut.s", "m0"])` that resolves to no leaf signal, so it used to fall
//! back to the AST interpreter. There its read dependency mis-resolved
//! bare-first to the top-scope `v` instead of the port `dut.v`, so the assign
//! ran once at time 0 with `v` still X and never re-triggered — the whole
//! struct read back all-X. The member cont-assign now compiles to a bit-range
//! splice into the container signal, matching a plain vector partial drive.

use xezim::simulate;

const SRC: &str = r#"
module leaf (input logic [9:0] v, output logic [20:0] whole);
  typedef struct packed { logic e; logic [9:0] m1; logic [9:0] m0; } t;
  t s;
  assign s.m0 = v;      // partial member drive -> low 10 bits
  assign whole = s;     // whole-struct read-back
endmodule

module tb;
  logic [9:0]  v;
  logic [20:0] whole;
  leaf dut (.v(v), .whole(whole));
  initial begin
    v = 10'h2AA;   // 1010101010
    #1;
  end
endmodule
"#;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

#[test]
fn submodule_packed_struct_member_contassign() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // The driven member (m0, bits [9:0]) must equal v through the whole struct.
    assert_eq!(get(&sim, "whole") & 0x3FF, 0x2AA);
}
