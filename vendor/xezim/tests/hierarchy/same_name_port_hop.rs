//! A port hop whose actual shares the SUBMODULE PORT's name silently
//! self-assigned instead of driving the parent signal.
//!
//!     module holder (output some_t [1:0] t);  ...
//!     module tb;
//!       some_t [1:0] t;              // same name as the port
//!       holder u_h (.t(t[1:0]));
//!
//! Inlining builds `t[1:0] = u_h.t`, and scope inference (correctly, for
//! everything else) gave the entry scope `u_h` — so the LHS resolved
//! scope-first to `u_h.t`, a self-assign, and the testbench signal stayed X
//! forever. The `lhs_is_absolute` guard already forced "no scope" for a BARE
//! `t = …` LHS in exactly this situation; it now looks through one
//! Index/RangeSelect layer so `t[1:0] = …` and `t[i] = …` anchor the same way.
//!
//! Found via a customer testbench with a three-deep pass-through hierarchy,
//! where every inner hop was fine and only the final testbench hop (the only
//! one whose actual shared the port name) lost the data. Reference-simulator
//! verified.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

const SRC: &str = r#"
package cargo_pkg;
  typedef struct packed {
    logic [63:0] pd;
    logic [7:0]  tag;
    logic        v;
    logic        e;
  } line_t;                                   // 74 bits
endpackage

module maker (output cargo_pkg::line_t [1:0] t);
  always_comb begin
    t[0].pd  = 64'hA5A5_A5A5_B4B4_B4B4;
    t[0].tag = 8'h11;
    t[0].v   = 1'b1;
    t[0].e   = 1'b0;
    t[1].pd  = 64'h5A5A_5A5A_C3C3_C3C3;
    t[1].tag = 8'h22;
    t[1].v   = 1'b0;
    t[1].e   = 1'b1;
  end
endmodule

module passer (output cargo_pkg::line_t [1:0] t);
  maker u_mk (.t(t[1:0]));                    // inner hop, same-named too
endmodule

module tb;
  cargo_pkg::line_t [1:0] t;                  // same name as the port
  passer u_p (.t(t[1:0]));
  logic [63:0] pd0, pd1;
  logic [7:0]  tag0, tag1;
  logic [3:0]  flags;                          // {v0,e0,v1,e1}
  initial begin
    #1;
    pd0   = t[0].pd;
    pd1   = t[1].pd;
    tag0  = t[0].tag;
    tag1  = t[1].tag;
    flags = {t[0].v, t[0].e, t[1].v, t[1].e};
  end
endmodule
"#;

#[test]
fn same_named_port_actual_drives_the_parent_signal() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(get(&sim, "pd0"), 0xA5A5_A5A5_B4B4_B4B4);
    assert_eq!(get(&sim, "pd1"), 0x5A5A_5A5A_C3C3_C3C3);
    assert_eq!(get(&sim, "tag0") & 0xFF, 0x11);
    assert_eq!(get(&sim, "tag1") & 0xFF, 0x22);
    // {v0,e0,v1,e1} = 1,0,0,1 — the single-bit members were the visible
    // symptom in the customer test.
    assert_eq!(get(&sim, "flags") & 0xF, 0b1001);
}
