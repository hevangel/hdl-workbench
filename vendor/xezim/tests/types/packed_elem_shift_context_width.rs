//! §11.6: a shift's LEFT operand is sized by the ASSIGNMENT context, which may
//! only WIDEN it — never narrow it below its own width.
//!
//! The compiled-block path sized it from the assignment width alone whenever
//! the operand was an ELEMENT SELECT of a packed multi-dimensional array. The
//! compiler's `expr_max_width` reported 1 bit for every `Index` expression —
//! right for a bit-select of a plain vector (`v[3]`), wrong for `s[1]` on
//! `logic [1:0][11:0] s`, which is 12 bits. The shift context is computed as
//! `max(assignment_width, left_operand_width)`, so the 1 lost the max and the
//! source was truncated to the DESTINATION's width BEFORE shifting — silently
//! dropping N extra high bits.
//!
//! The procedural interpreter computed it correctly, so the two disagreed and
//! the corruption appeared only inside `always_comb`/`always @*`.

use xezim::simulate;

/// Every `NOTE:` line the run printed, in order.
fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

/// A 12-bit element shifted into a 10-bit element: the source keeps its own
/// width through the shift, so only the ASSIGNMENT truncates.
#[test]
fn packed_element_keeps_its_width_through_a_shift_in_a_compiled_block() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic [1:0][11:0] wide;
  logic [3:0][9:0]  narrow;
  always_comb narrow[2] = wide[1] >> 2;
  initial begin
    wide[0] = '0;
    wide[1] = 12'hFFF;
    #1;
    $display("NOTE: %0h", narrow[2]);
    $finish;
  end
endmodule
"#;
    // 12'hFFF >> 2 = 10'h3FF, which fills the 10-bit destination exactly.
    // The buggy path computed (10 - 2) = 8 significant bits => 0h0FF.
    assert_eq!(notes(src), vec!["NOTE: 3ff"]);
}

/// The shift amount must not scale the loss: sweep it and check each result is
/// `min(src_width - n, dst_width)` ones, not `dst_width - n`.
#[test]
fn shift_amount_does_not_narrow_the_packed_source() {
    for (n, want) in [(0u32, "3ff"), (1, "3ff"), (2, "3ff"), (3, "1ff"), (4, "ff")] {
        let src = format!(
            r#"
`timescale 1ns/1ns
module top;
  logic [1:0][11:0] wide;
  logic [3:0][9:0]  narrow;
  always_comb narrow[2] = wide[1] >> {n};
  initial begin
    wide[0] = '0;
    wide[1] = 12'hFFF;
    #1;
    $display("NOTE: %0h", narrow[2]);
    $finish;
  end
endmodule
"#
        );
        assert_eq!(notes(&src), vec![format!("NOTE: {want}")], "shift by {n}");
    }
}

/// The procedural path was always right — it must stay right.
#[test]
fn procedural_path_agrees_with_the_compiled_one() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic [1:0][11:0] wide;
  logic [3:0][9:0]  proc_dst;
  logic [3:0][9:0]  comb_dst;
  always_comb comb_dst[2] = wide[1] >> 2;
  initial begin
    wide[0] = '0;
    wide[1] = 12'hFFF;
    #1;
    proc_dst[2] = wide[1] >> 2;
    $display("NOTE: %0h", proc_dst[2]);
    $display("NOTE: %0h", comb_dst[2]);
    $finish;
  end
endmodule
"#;
    let n = notes(src);
    assert_eq!(n, vec!["NOTE: 3ff", "NOTE: 3ff"]);
}

/// A bit-select of a PLAIN vector really is 1 bit — widening it would be just
/// as wrong in the other direction.
#[test]
fn plain_vector_bit_select_is_still_one_bit() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic [11:0] v;
  logic [3:0]  w;
  always_comb w = {3'b0, v[9]};
  initial begin
    v = 12'b0000_0010_0000;
    #1;
    $display("NOTE: %0h", w);
    v = 12'b0010_0000_0000;
    #1;
    $display("NOTE: %0h", w);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: 0", "NOTE: 1"]);
}

/// The element select must carry its full width through a CONCATENATION too —
/// the same `expr_max_width` feeds concat sizing.
#[test]
fn packed_element_carries_full_width_into_a_concatenation() {
    let src = r#"
`timescale 1ns/1ns
module top;
  logic [1:0][11:0] wide;
  logic [23:0]      joined;
  always_comb joined = {wide[1], wide[0]};
  initial begin
    wide[0] = 12'h0AB;
    wide[1] = 12'hCDE;
    #1;
    $display("NOTE: %0h", joined);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: cde0ab"]);
}
