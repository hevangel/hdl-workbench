//! §7.2/§23.2.2 — a module PORT whose type is an unpacked struct.
//!
//! An unpacked struct is stored one signal per member, but the port paths
//! (the module's own ANSI port list and the inlined instance's `<inst>.<port>`
//! signal) registered only the flat container. Two consequences, both in
//! xezim#121:
//!
//! 1. a member read through the port fell back to slicing the container and
//!    lost `is_real` — `u1.o.a` came back as the raw f64 BIT PATTERN
//!    (9.25 read as 4621396905123905536);
//! 2. the parent never saw any member value at all, because the connection is
//!    emitted as a whole-struct continuous assign (`assign s = u1.o;`) during
//!    inlining — i.e. AFTER the in-module member-wise expansion pass had
//!    already run.
//!
//! Fixed by registering the member leaves on both port paths and re-running
//! the whole-struct CA expansion after inlining. Both port directions are
//! pinned here: the output half was the reported symptom, the input half rides
//! the same connection machinery.

use xezim::simulate;

/// Output port: the submodule drives members, the parent reads them, and a
/// `real` member keeps its type through the port.
const OUT_PORT: &str = r#"
typedef struct { real a; real b; } S;
module ch #(parameter real P = 9.25) (output S o, output real direct);
  assign o.a    = P;
  assign o.b    = 1.0;
  assign direct = P;
endmodule
module tb;
  S s; real d;
  ch #(.P(9.25)) u1 (.o(s), .direct(d));
  real got_a, got_b, got_inst_a;
  initial begin
    #1;
    got_a      = s.a;
    got_b      = s.b;
    got_inst_a = u1.o.a;   // read through the instance's own port signal
  end
endmodule
"#;

/// Input port with mixed member types, plus an output struct fed from it —
/// the parent→instance direction of the same machinery.
const IN_PORT: &str = r#"
typedef struct { real a; bit [7:0] b; } S;
module sink (input S i, output real seen_a, output bit [7:0] seen_b);
  assign seen_a = i.a;
  assign seen_b = i.b;
endmodule
module tb;
  S s; real ra; bit [7:0] rb;
  sink u2 (.i(s), .seen_a(ra), .seen_b(rb));
  initial begin
    s.a = 3.5;
    s.b = 8'hA5;
  end
endmodule
"#;

fn real_of(sim: &xezim::compiler::Simulator, name: &str) -> f64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_f64()
}

fn u64_of(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

#[test]
fn unpacked_struct_output_port_reaches_parent_with_member_types() {
    let sim = simulate(OUT_PORT, 1000).expect("simulate failed");
    // The plain `real` port already worked; it pins that nothing regressed.
    assert_eq!(real_of(&sim, "d"), 9.25);
    // The parent must see the driven members (was 0.0).
    assert_eq!(real_of(&sim, "got_a"), 9.25);
    assert_eq!(real_of(&sim, "got_b"), 1.0);
    // Reading through the instance port must keep `is_real` (was the raw
    // f64 bit pattern 4621396905123905536).
    assert_eq!(real_of(&sim, "got_inst_a"), 9.25);
}

#[test]
fn unpacked_struct_input_port_delivers_members() {
    let sim = simulate(IN_PORT, 1000).expect("simulate failed");
    assert_eq!(real_of(&sim, "ra"), 3.5);
    assert_eq!(u64_of(&sim, "rb"), 0xA5);
}
