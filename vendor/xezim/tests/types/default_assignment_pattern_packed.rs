//! §10.9.2 — `'{default: expr}` on a PACKED target assigns one item per
//! ELEMENT. A plain packed vector is a packed array of 1-bit elements, so
//! `logic [15:0] v = '{default:1'b1};` is `16'hffff`, and an integer atom
//! behaves the same way (`int i = '{default:1'b1};` is `-1`).
//!
//! xezim only reached the per-element expansion when the target had a
//! registered packed ELEMENT width — i.e. a packed ARRAY. A plain vector fell
//! through to the generic "concatenate the items at their own widths"
//! fallback, which turned the single item into a ONE-BIT value and
//! zero-extended it: `16'h0001`. In a bank tracker whose reset did
//! `bank_active <= '{default:1'b1};` that armed only bank 0 and left the other
//! fifteen dead, which then propagated into every downstream ready/grant.
//!
//! The expansion also had to be wired into the BLOCKING assignment path — the
//! continuous and nonblocking paths already called it, so the same
//! declaration behaved differently depending on which operator wrote it.
//! Reference-validated.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Every assignment operator must expand the pattern the same way.
#[test]
fn default_pattern_fills_a_packed_vector() {
    let src = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [15:0] nb, bl;
  logic [15:0] ca;
  assign ca = '{default:1'b1};
  always_ff @(posedge clk) nb <= '{default:1'b1};
  int b_i, n_i, c_i;
  initial begin
    bl = '{default:1'b1};
    @(posedge clk); #1;
    b_i = bl; n_i = nb; c_i = ca;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "b_i") & 0xFFFF, 0xFFFF, "blocking assign");
    assert_eq!(u(&sim, "n_i") & 0xFFFF, 0xFFFF, "nonblocking assign");
    assert_eq!(u(&sim, "c_i") & 0xFFFF, 0xFFFF, "continuous assign");
}

/// Integer atoms fill too; aggregates that own their own pattern path keep it.
#[test]
fn default_pattern_across_target_shapes() {
    let src = r#"
module tb;
  typedef struct packed { logic [3:0] a; logic [3:0] b; } ps_t;
  logic [3:0]        vz;
  int                ii;
  byte               bb;
  logic              s1;
  ps_t               ps;
  logic [15:0][15:0] p2;
  logic [7:0]        ua [4];
  logic [1:0]        ord;
  int e_vz, e_ii, e_bb, e_s1, e_ps, e_p2, e_u0, e_u3, e_ord;
  initial begin
    vz  = '{default:1'b0};
    ii  = '{default:1'b1};
    bb  = '{default:1'b1};
    s1  = '{default:1'b1};
    ps  = '{default:1'b1};
    p2  = '{default:'0};
    ua  = '{default:8'hAB};
    ord = '{1'b1, 1'b0};
    #1;
    e_vz = vz; e_ii = ii; e_bb = bb; e_s1 = s1; e_ps = ps;
    e_p2 = p2[3]; e_u0 = ua[0]; e_u3 = ua[3]; e_ord = ord;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "e_vz") & 0xF, 0x0);
    assert_eq!(u(&sim, "e_ii") as i32, -1, "int fills all 32 bits");
    assert_eq!(u(&sim, "e_bb") as i32, -1, "byte fills all 8 bits");
    assert_eq!(u(&sim, "e_s1") & 1, 1, "a 1-bit target is unaffected");
    // A STRUCT's `default:` applies per MEMBER, not per bit: each 4-bit
    // member takes 1'b1 zero-extended, so `0001_0001`. Reference-confirmed.
    assert_eq!(
        u(&sim, "e_ps") & 0xFF,
        0b0001_0001,
        "packed struct fills per member"
    );
    assert_eq!(u(&sim, "e_p2") & 0xFFFF, 0, "packed 2-D keeps its own path");
    assert_eq!(
        u(&sim, "e_u0") & 0xFF,
        0xAB,
        "unpacked array keeps its own path"
    );
    assert_eq!(u(&sim, "e_u3") & 0xFF, 0xAB);
    assert_eq!(
        u(&sim, "e_ord") & 0x3,
        0b10,
        "ordered items still positional"
    );
}
