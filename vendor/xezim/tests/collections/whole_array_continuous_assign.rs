//! §10.6.2 — a whole-ARRAY continuous assignment between unpacked arrays.
//!
//! ```systemverilog
//! logic [9:0] src [4];
//! logic [9:0] dst [4];
//! assign dst = src;
//! ```
//!
//! An unpacked array has no single backing signal — its ELEMENTS are the
//! signals — so this was pushed as one scalar assignment that matched no
//! target and did nothing at all: `dst` stayed x for the whole run and never
//! responded to a change in `src`. The per-element spelling
//! (`assign dst[i] = src[i];`) always worked, as did procedural writes and the
//! packed-2D form (`logic [3:0][9:0]`), which is what made this so quiet — the
//! shape looks ordinary and only the whole-array spelling fails.
//!
//! Found while debugging a CDC design where an array output port driven this
//! way left the parent's array x, which in turn made four downstream
//! struct-member assigns propagate x. The struct assigns looked like the
//! culprit; they were faithfully forwarding an x source.
//!
//! The SUB-MODULE case needed a second fix. An inlined body's assigns reach
//! neither elaborate-side pending-drain — the simulator drains
//! `pending_cont_assign` itself — and on the way there the array names looked
//! undeclared (an unpacked array has no signal under its own name), so each
//! side got a 1-BIT implicit net and the assignment drove a phantom scalar.
//!
//! Arrays of DIFFERENT element counts are left on their existing path rather
//! than expanded (assigning between differently-sized unpacked arrays is
//! illegal per §10.9 anyway, so there is nothing useful to pin about it).
//!
//! Expectations below are byte-identical to a reference simulator.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Every element is driven, and the assignment stays live: a later change to
/// the source propagates.
#[test]
fn whole_array_assign_drives_every_element_and_tracks_updates() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic [9:0] src [4];
  logic [9:0] dst [4];
  assign dst = src;
  int d0, d1, d2, d3, d1_after;
  initial begin
    src[0] = 10'h011; src[1] = 10'h022; src[2] = 10'h033; src[3] = 10'h044;
    #1;
    d0 = dst[0]; d1 = dst[1]; d2 = dst[2]; d3 = dst[3];
    src[1] = 10'h077;
    #1 d1_after = dst[1];
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "d0"), 0x011);
    assert_eq!(u(&sim, "d1"), 0x022);
    assert_eq!(u(&sim, "d2"), 0x033);
    assert_eq!(u(&sim, "d3"), 0x044);
    assert_eq!(
        u(&sim, "d1_after"),
        0x077,
        "the assign stays live after the source changes"
    );
}

/// An array whose element type is a packed struct — the shape the CDC design
/// used — and index ranges that do not start at 0.
#[test]
fn whole_array_assign_of_struct_elements_and_offset_ranges() {
    let src = r#"
`timescale 1ns/1ns
module top;
  typedef struct packed { logic [9:0] f; } hf_t;
  hf_t hsrc [4];
  hf_t hdst [4];
  logic [7:0] osrc [1:4];
  logic [7:0] odst [1:4];
  assign hdst = hsrc;
  assign odst = osrc;
  int h0, h3, o1, o4;
  initial begin
    hsrc[0] = 10'h055; hsrc[3] = 10'h0AA;
    osrc[1] = 8'h5A;   osrc[4] = 8'hC3;
    #1;
    h0 = hdst[0]; h3 = hdst[3]; o1 = odst[1]; o4 = odst[4];
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "h0"), 0x055, "struct-element array, first");
    assert_eq!(u(&sim, "h3"), 0x0AA, "struct-element array, last");
    assert_eq!(u(&sim, "o1"), 0x5A, "1-based range, first");
    assert_eq!(u(&sim, "o4"), 0xC3, "1-based range, last");
}

/// The guard: forms that already worked must be untouched — per-element
/// assigns, a packed 2D whole assign, and a plain scalar assign.
#[test]
fn existing_assign_forms_are_unchanged() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic [9:0] src [4];
  logic [9:0] per [4];
  logic [3:0][9:0] psrc, pdst;
  logic [7:0] a, b;
  assign per[0] = src[0];
  assign per[1] = src[1];
  assign pdst = psrc;
  assign b = a;
  int p0, p1, pk, sc;
  initial begin
    src[0] = 10'h011; src[1] = 10'h022;
    psrc = {10'h004, 10'h003, 10'h002, 10'h001};
    a = 8'h5A;
    #1;
    p0 = per[0]; p1 = per[1]; pk = pdst[2]; sc = b;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "p0"), 0x011, "per-element assign");
    assert_eq!(u(&sim, "p1"), 0x022);
    assert_eq!(u(&sim, "pk"), 0x003, "packed 2D whole assign");
    assert_eq!(u(&sim, "sc"), 0x5A, "plain scalar assign");
}

/// The reported shape: a sub-module drives an unpacked-array OUTPUT PORT with
/// a whole-array assign, and the parent reads the elements. This is what left
/// a CDC design's pointer array x for an entire run.
#[test]
fn whole_array_assign_through_a_submodule_output_port() {
    let src = r#"
`timescale 1ns/1ns
module producer (output logic [9:0] o [4]);
  logic [9:0] internal [4];
  initial begin
    internal[0] = 10'h011; internal[1] = 10'h022;
    internal[2] = 10'h033; internal[3] = 10'h044;
  end
  assign o = internal;
endmodule
module top;
  logic [9:0] got [4];
  producer u (.o(got));
  int g0, g1, g2, g3;
  initial begin
    #1;
    g0 = got[0]; g1 = got[1]; g2 = got[2]; g3 = got[3];
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "g0"), 0x011);
    assert_eq!(u(&sim, "g1"), 0x022);
    assert_eq!(u(&sim, "g2"), 0x033);
    assert_eq!(u(&sim, "g3"), 0x044);
}

/// The same assign entirely LOCAL to a sub-module — proves the sub-module gap
/// was not about the port, and that no phantom implicit net is created.
#[test]
fn whole_array_assign_local_to_a_submodule() {
    let src = r#"
`timescale 1ns/1ns
module inner (output logic [9:0] probe);
  logic [9:0] a [4];
  logic [9:0] b [4];
  initial begin a[0] = 10'h011; a[1] = 10'h022; a[2] = 10'h033; a[3] = 10'h044; end
  assign b = a;
  assign probe = b[1];
endmodule
module top;
  logic [9:0] p;
  inner u (.probe(p));
  int seen;
  initial #1 seen = p;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "seen"),
        0x022,
        "a sub-module-local whole-array assign drives"
    );
}

/// End-to-end in the shape that surfaced it: an array output port feeds
/// per-member continuous assigns into a packed struct, clocked.
#[test]
fn array_output_port_feeding_struct_member_assigns() {
    let src = r#"
`timescale 1ns/1ns
package pk;
  typedef struct packed { logic [9:0] f; } hf_t;
  typedef struct packed { hf_t w3; hf_t w2; hf_t w1; hf_t w0; } ps_t;
endpackage
module counters #(parameter int N = 4, parameter int W = 10)
  (input logic clk, output logic [W-1:0] ptr_next [N]);
  logic [W-1:0] ptr [N];
  initial for (int i = 0; i < N; i++) ptr[i] = '0;
  always @(posedge clk) for (int i = 0; i < N; i++) ptr[i] <= ptr[i] + 1;
  assign ptr_next = ptr;
endmodule
module top;
  import pk::*;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [9:0] pos [4];
  ps_t sync_client;
  counters #(.N(4), .W(10)) u (.clk(clk), .ptr_next(pos));
  assign sync_client.w0 = pos[0];
  assign sync_client.w1 = pos[1];
  assign sync_client.w2 = pos[2];
  assign sync_client.w3 = pos[3];
  int unk_at_1, pos0_at_21, w0_at_21;
  initial begin
    #1  unk_at_1 = $isunknown(sync_client);
    #20 pos0_at_21 = pos[0];
        w0_at_21   = sync_client.w0;
  end
endmodule
"#;
    let sim = simulate(src, 400).expect("simulate failed");
    assert_eq!(
        u(&sim, "unk_at_1"),
        0,
        "the struct resolves as soon as the array does"
    );
    assert_eq!(u(&sim, "pos0_at_21"), 2, "two posedges elapsed");
    assert_eq!(u(&sim, "w0_at_21"), 2, "and the member assign forwards it");
}
