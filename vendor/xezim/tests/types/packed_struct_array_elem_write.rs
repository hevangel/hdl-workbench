//! §7.4.2 — writing a packed ELEMENT of a struct member inside a PACKED array
//! of packed structs: `arr[i].field[k] = v`. Reference-validated.
//!
//! `arr[i]` is not a signal of its own here — a packed array of packed structs
//! is one backing vector — so the write must splice at
//! `slot*struct_w + field_off + k*member_elem_w`. The existing member path
//! handled `arr[i].field = v`, but this form's outermost AST node is an Index,
//! so it never reached that code and the write was silently DROPPED.
//!
//! The read path resolved the same expression correctly all along, which is
//! what made the value look like it was never driven rather than never stored:
//! reading back gave 0 with no error anywhere.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

const SRC: &str = r#"
package P;
  typedef struct packed {
    logic [1:0][63:0] wdata;
    logic [1:0][7:0]  mask;
    logic [1:0]       amask;
  } t;
endpackage

module tb;
  P::t            s;    // plain struct, for contrast
  P::t [1:0]      c1;   // 1-D packed array of packed structs
  P::t [0:0][1:0] c2;   // 2-D — the reported shape

  logic [63:0]  r_plain, r_e1w0, r_e1w1, r_e0w0;
  logic [1:0]   r_amask;
  logic [63:0]  r_c2a, r_c2b;
  logic [127:0] r_c2m;
  logic [1:0]   r_c2am;
  int           b_elem, b_memb, b_sub;

  initial begin
    s = '0; c1 = '0;
    s.wdata[0]     = 64'h11;
    c1[1].wdata[0] = 64'h22;    // the form that was dropped
    c1[1].wdata[1] = 64'h33;
    c1[0].wdata[0] = 64'h44;    // a different element must not collide
    c1[0].amask    = 2'b11;     // member with no trailing index still works

    c2 = '0;
    c2[0][1].wdata[0] = 64'h55;   // 2-D: two indices before the member
    c2[0][0].wdata[1] = 64'h66;
    c2[0][0].amask    = 2'b10;
    #1;
    r_plain  = s.wdata[0];
    r_e1w0   = c1[1].wdata[0];
    r_e1w1   = c1[1].wdata[1];
    r_e0w0   = c1[0].wdata[0];
    r_amask  = c1[0].amask;
    r_c2a    = c2[0][1].wdata[0];
    r_c2b    = c2[0][0].wdata[1];
    r_c2m    = c2[0][0].wdata;
    r_c2am   = c2[0][0].amask;
    b_elem   = $bits(c2[0][0]);
    b_memb   = $bits(c2[0][0].wdata);
    b_sub    = $bits(c2[0][0].wdata[0]);
  end
endmodule
"#;

#[test]
fn packed_array_of_struct_member_element_writes_land() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "r_plain"),
        0x11,
        "plain struct member element (control)"
    );
    assert_eq!(
        u(&sim, "r_e1w0"),
        0x22,
        "c1[1].wdata[0] — this write used to be dropped"
    );
    assert_eq!(
        u(&sim, "r_e1w1"),
        0x33,
        "c1[1].wdata[1] — second element of the same member"
    );
    assert_eq!(
        u(&sim, "r_e0w0"),
        0x44,
        "c1[0].wdata[0] — a different array element must not alias c1[1]"
    );
    assert_eq!(
        u(&sim, "r_amask"),
        0b11,
        "member with no trailing index (control)"
    );
}

/// The reported shape: TWO indices before the member. Both halves needed
/// generalising — `struct_w` must come from the field layout, since
/// `packed_signal_elem_widths[root]` is the width of an element of the
/// OUTERMOST dimension (for `t [0:0][1:0]` that is the whole `[1:0]`
/// sub-array, not one struct).
#[test]
fn two_dim_packed_array_of_struct_round_trips() {
    let sim = simulate(SRC, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "r_c2a"),
        0x55,
        "c2[0][1].wdata[0] through two indices"
    );
    assert_eq!(
        u(&sim, "r_c2b"),
        0x66,
        "c2[0][0].wdata[1] — different element and lane"
    );
    assert_eq!(
        u(&sim, "r_c2am"),
        0b10,
        "c2[0][0].amask — member with no trailing index"
    );
    assert_eq!(
        u(&sim, "r_c2m") & 0xFFFF_FFFF_FFFF_FFFF,
        0,
        "c2[0][0].wdata low lane is untouched by the writes above"
    );
    // A failed member resolution used to fall back to the 32-bit default.
    assert_eq!(u(&sim, "b_elem"), 146, "$bits(c2[0][0])");
    assert_eq!(
        u(&sim, "b_memb"),
        128,
        "$bits(c2[0][0].wdata) — 32 meant unresolved"
    );
    assert_eq!(
        u(&sim, "b_sub"),
        64,
        "$bits(c2[0][0].wdata[0]) — 1 meant a bit-select"
    );
}
