//! §9.2.2.2 — an `always_comb` whose only input is a MEMBER of a packed
//! struct must re-fire when the struct changes. Reference-validated.
//!
//! A packed struct is ONE signal (`st`), but a member read collects as
//! `st.m` — a name no signal matches. The dependency resolver simply dropped
//! it, so the block computed once at time 0 and FROZE. The values were stale
//! but fully known — no X anywhere to flag the miss — which is what made the
//! field report ("wrong but clean outputs after the input changed") so
//! confusing. The fix resolves a read name by stripping member segments from
//! the right until a real signal matches (`u.bus.m` → `u.bus`), i.e. the
//! §9.2.2.2 longest static prefix.
//!
//! The guard the first fix version got wrong: an UNPACKED array of structs
//! (`e_t arr [3]`) stores its element leaves as separate signals, and its
//! reads already collect per-leaf — resolution must not shortcut those to the
//! never-written base cell (that broke `comb_refires_on_struct_array_field_write`).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

/// Top level: constant and dynamic member bit-selects both track changes.
#[test]
fn member_bitselect_refires_at_top_level() {
    let src = r#"
module tb;
  typedef struct packed { logic [3:0] a; logic [31:0] m; logic [1:0] q; } s_t;
  s_t st;
  logic bc, bd; int k;
  int r_bc, r_bd;
  always_comb bc = st.m[8];
  always_comb bd = st.m[k];
  initial begin
    k = 8; st = '0;
    #10 st.m = 32'h100;
    #10 r_bc = bc; r_bd = bd;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r_bc"), 1, "constant member bit-select re-fired");
    assert_eq!(u(&sim, "r_bd"), 1, "dynamic member bit-select re-fired");
}

/// The field shape: a struct-typed input port fed by a concat, its member
/// bit-selected in a loop, two modules deep — and the input CHANGES after
/// time 0 (every probe that only assigns at t=0 hides this bug).
#[test]
fn struct_port_member_loop_tracks_input_changes() {
    let src = r#"
package p;
   typedef struct packed {
      logic [3:0]   a;
      logic [31:0]  m;
      logic [255:0] d;
      logic [1:0]   q;
   } s_t;
endpackage
module expander (
   input  p::s_t st,
   output logic [255:0] o
);
   always_comb begin
      for (int i = 0; i < 32; ++i)
         o[i*8 +: 8] = {8{st.m[i]}};
   end
endmodule
module mid (
   input  logic [31:0] m_in,
   output logic [255:0] o
);
   import p::*;
   s_t bus;
   assign bus = {4'b1010, m_in, {4{64'hDEADBEEFCAFEBABE}}, 2'b11};
   expander u(.st(bus), .o(o));
endmodule
module tb;
   logic [31:0] m;
   logic [255:0] o;
   mid um(.m_in(m), .o(o));
   int b0, b8, b8_before;
   initial begin
      m = 32'h0;
      #10 b8_before = o[71:64];
      m = 32'h0000_0100;
      #10 b0 = o[7:0]; b8 = o[71:64];
   end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "b8_before"), 0);
    assert_eq!(u(&sim, "b0"), 0);
    assert_eq!(
        u(&sim, "b8"),
        0xFF,
        "the expander re-fired on the post-t0 input change"
    );
}
