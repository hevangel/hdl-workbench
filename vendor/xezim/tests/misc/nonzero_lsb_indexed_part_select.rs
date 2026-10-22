//! Regression test for INDEXED part-selects (`[base +: W]` / `[base -: W]`) on
//! packed vectors declared with a NON-zero lower bound (IEEE 1800-2023
//! §7.4.1 / §11.5.1).
//!
//! For `logic [hi:lo] v`, user index `i` maps to storage bit `i - lo`. The
//! declared-lower-bound offset must be applied to the BASE index only: in
//! `[base +: W]` / `[base -: W]` the right operand is a WIDTH, not an index, so
//! shifting it corrupts the selection. (The constant `[msb:lsb]` form, where
//! both operands are indices, shifts both — covered by
//! `nonzero_lsb_part_select_write.rs`.)
//!
//! Verified byte-for-byte against reference simulators:
//!   logic [3:1] w / logic [7:4] v
//!     v[5 +: 2] read     -> 01
//!     w[3 -: 2] = 2'b00  -> 001
//!     w[1 +: 2] = 2'b11  -> 011
//!     w[2 +: 2] read     -> 11
//!     w[3 -: 2] read     -> 11

use xezim::simulate;

const SRC: &str = r#"
module top;
  logic [3:1] w;   // 3-bit vector, lower bound 1
  logic [7:4] v;   // 4-bit vector, lower bound 4
  initial begin
    integer bad;
    bad = 0;

    // IndexedUp READ: v = 4'b1010 (v[7]=1,v[6]=0,v[5]=1,v[4]=0); v[5 +: 2] is
    // bits idx 5,6 = 2'b01 = 1.
    v = 4'b1010;
    if (v[5 +: 2] != 2'b01) begin
      $display("FAIL v[5+:2]=%b expected 01", v[5 +: 2]);
      bad = bad + 1;
    end

    // IndexedDown WRITE: w = 7 (3'b111); w[3 -: 2] = 2'b00 clears idx 3,2 ->
    // w = 3'b001 = 1.
    w = 3'b111;
    w[3 -: 2] = 2'b00;
    if (w != 3'd1) begin
      $display("FAIL w[3-:2]=0 -> w=%b expected 001", w);
      bad = bad + 1;
    end

    // IndexedUp WRITE: w = 0; w[1 +: 2] = 2'b11 sets idx 1,2 -> w = 3'b011 = 3.
    w = 3'b000;
    w[1 +: 2] = 2'b11;
    if (w != 3'd3) begin
      $display("FAIL w[1+:2]=3 -> w=%b expected 011", w);
      bad = bad + 1;
    end

    // IndexedUp READ on w: w = 6 (3'b110); w[2 +: 2] is idx 2,3 = w[3:2] = 11.
    w = 3'b110;
    if (w[2 +: 2] != 2'b11) begin
      $display("FAIL w[2+:2]=%b expected 11", w[2 +: 2]);
      bad = bad + 1;
    end

    // IndexedDown READ on w: w = 6; w[3 -: 2] is idx 3,2 = 11.
    if (w[3 -: 2] != 2'b11) begin
      $display("FAIL w[3-:2]=%b expected 11", w[3 -: 2]);
      bad = bad + 1;
    end

    if (bad == 0)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL bad=%0d", bad);
  end
endmodule
"#;

#[test]
fn test_nonzero_lsb_indexed_part_select() {
    let sim = simulate(SRC, 10_000).expect("simulation failed");
    assert!(
        sim.output
            .iter()
            .any(|line| line.message.contains("TAG_PASS")),
        "expected TAG_PASS in output, got: {:?}",
        sim.output
    );
}
