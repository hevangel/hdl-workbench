//! §11.5.1: in an indexed part-select `[base +: width]` / `[base -: width]`
//! the RIGHT operand is a WIDTH, not a second index.
//!
//! The compiler's `arr[i][hi:lo] = val` arm emitted `left`/`right` as hi/lo
//! REGARDLESS of the range kind, so `arr[n][64 +: 32]` became the 33-bit
//! window `[64:32]`: the payload landed 32 bits low and the top bit was
//! clipped. The flat-signal arm and the nonblocking array arm both convert
//! base+width to hi/lo — only the blocking array arm did not, so the bug
//! needed all three of: an unpacked-array ELEMENT, an INDEXED part-select,
//! and a BLOCKING (continuous-assign) target.
//!
//! This is what wedged the C910 SoC's PLIC. `plic_hreg_busif.v` builds a
//! prefix-OR chain out of continuous assigns to successive slices of one
//! array element:
//!
//! ```verilog
//! assign mie_lst_read_tmp[n][31:0] = 32'b0;
//! assign mie_lst_read_tmp[n][32*(m+1)+:32] = mie_lst_read_tmp[n][32*m+:32] | (...);
//! ```
//!
//! Every stage therefore read back the previous stage's clipped/misplaced
//! bits, the settle loop never converged, and the run died in X-churn with a
//! dead-clock watchdog. All expectations below are reference-simulator
//! verified.

use xezim::simulate;

fn msgs(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn indexed_part_select_target_on_array_element() {
    let out = msgs(
        r#"
module top;
  wire [95:0] s [1:0];
  wire [95:0] t [1:0];
  assign s[1][32 +: 32] = 32'h44444444;
  assign t[0][64 +: 32] = 32'hDEADBEEF;
  initial begin
    #1;
    $display("A_%h", s[1]);
    $display("B_%h", t[0]);
    $display("C_%h", s[0]);
  end
endmodule
"#,
    );
    // The window is [63:32] and [95:64] respectively — NOT [32:32] / [64:32].
    assert!(
        out.contains(&"A_zzzzzzzz44444444zzzzzzzz".to_string()),
        "{out:?}"
    );
    assert!(
        out.contains(&"B_deadbeefzzzzzzzzzzzzzzzz".to_string()),
        "{out:?}"
    );
    // The untouched sibling element stays fully undriven.
    assert!(
        out.contains(&"C_zzzzzzzzzzzzzzzzzzzzzzzz".to_string()),
        "{out:?}"
    );
}

#[test]
fn downward_indexed_part_select_target_on_array_element() {
    let out = msgs(
        r#"
module top;
  wire [95:0] d [1:0];
  assign d[0][95 -: 32] = 32'hFEEDFACE;
  initial begin
    #1;
    $display("D_%h", d[0]);
  end
endmodule
"#,
    );
    assert!(
        out.contains(&"D_feedfacezzzzzzzzzzzzzzzz".to_string()),
        "{out:?}"
    );
}

#[test]
fn prefix_or_chain_across_array_element_slices() {
    // The C910 PLIC shape, reduced: each generate stage drives the next slice
    // of the SAME array element from the previous one.
    let out = msgs(
        r#"
module top;
  localparam INT_NUM  = 64;
  localparam HART_NUM = 2;
  wire [INT_NUM-1:0] data = 64'hAAAABBBBCCCCDDDD;
  wire [1:0]         rd   = 2'b11;
  wire [INT_NUM+31:0] arr [HART_NUM-1:0];
  assign arr[0][31:0] = 32'b0;
  assign arr[1][31:0] = 32'b0;
  genvar m, n;
  generate
    for (n = 0; n < HART_NUM; n = n + 1) begin : H
      for (m = 0; m < INT_NUM/32; m = m + 1) begin : I
        assign arr[n][32*(m+1)+:32] = arr[n][32*m+:32] | ({32{rd[m]}} & data[32*m+:32]);
      end
    end
  endgenerate
  initial begin
    #1;
    $display("E_%h", arr[0]);
    $display("F_%h", arr[1]);
  end
endmodule
"#,
    );
    assert!(
        out.contains(&"E_eeeeffffccccdddd00000000".to_string()),
        "{out:?}"
    );
    assert!(
        out.contains(&"F_eeeeffffccccdddd00000000".to_string()),
        "{out:?}"
    );
}
