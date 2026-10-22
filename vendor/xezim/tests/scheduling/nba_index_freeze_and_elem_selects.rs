//! Three lvalue/select gaps found by one pipeline testbench, all in the
//! `q[i][j]` family:
//!
//! 1. §10.4.2 — a non-blocking assignment's TARGET indices are evaluated when
//!    the assignment is SCHEDULED. An lvalue the fast path couldn't reduce to
//!    a signal id was stored as an expression and re-evaluated in the NBA
//!    region — by which time a `for` loop's variable is gone, so
//!    `for (int j…) q[j] <= …;` landed on the wrong bit or nowhere.
//!    Fixed by freezing the evaluated indices into literals at schedule time
//!    (`freeze_lvalue_indices`).
//!
//! 2. §7.4.1 — a bit/lane select on an ELEMENT of a 1-D unpacked array
//!    (`logic [1:0] q [0:2]; q[i][j] = b`). Each element is its own signal,
//!    so this is a sub-select into that signal — but it matched neither the
//!    `arrays_2d` arm nor any other, and the write was silently DROPPED: a
//!    pipeline written as `vld_q[i+1][j] <= vld_q[i][j]` never advanced and
//!    its output stayed x forever.
//!
//! 3. §7.4.1 — `arr[i].field[j]` where `arr` is an unpacked array of structs
//!    and `field` is a multi-dim packed array. The field's element width was
//!    registered only for NON-array declarators (an array of structs reached
//!    `continue` first), and the read path only built candidates for Ident
//!    bases — so the select degraded to a 1-BIT read and a reference model
//!    compared one bit against a 64-bit lane.
//!
//! All three verified against a reference simulator; the originating
//! testbench (2-stage valid-gated pipeline, X/Z/mixed/LFSR phases) passes
//! byte-identically on both.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// §10.4.2: the loop variable indexing the NBA target must be captured at
/// schedule time — for a flat vector, an unpacked element's bit, and a packed
/// 2-D element.
#[test]
fn nba_target_indices_freeze_at_schedule_time() {
    let src = r#"
module top;
  bit clk;
  logic rst_n;
  always #5 clk = ~clk;
  logic [3:0] flat;
  logic [1:0] unp [0:1];
  logic [1:0][1:0] pk;
  int flat_r, unp_r, pk_r;
  // Reset value driven from INSIDE the same block: every variable has exactly
  // one driver, so the shape is legal SV (§9.2.2.4) and portable to any
  // simulator rather than only running here.
  always_ff @(posedge clk) begin
    if (!rst_n) begin
      flat <= '0; unp[0] <= 2'b11; unp[1] <= 2'b00; pk <= 4'b0011;
    end
    else begin
      for (int j = 0; j < 4; j++) flat[j] <= 1'b1;
      for (int j = 0; j < 2; j++) unp[1][j] <= unp[0][j];
      for (int j = 0; j < 2; j++) pk[1][j] <= pk[0][j];
    end
  end
  initial begin
    rst_n = 0;
    repeat (2) @(posedge clk);
    #1 rst_n = 1;
    repeat (4) @(posedge clk);
    #1 flat_r = flat; unp_r = unp[1]; pk_r = pk[1];
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "flat_r"), 0b1111, "flat[j] <= … with loop-var j");
    assert_eq!(
        u(&sim, "unp_r"),
        0b11,
        "unp[1][j] <= unp[0][j] with loop-var j"
    );
    assert_eq!(
        u(&sim, "pk_r"),
        0b11,
        "pk[1][j] <= pk[0][j] with loop-var j"
    );
}

/// Constant-index writes into an unpacked element, blocking and NBA — the
/// case that previously matched NO lvalue arm at all.
#[test]
fn unpacked_element_bit_write_lands() {
    let src = r#"
module top;
  bit clk;
  logic rst_n;
  always #5 clk = ~clk;
  logic [1:0] q1 [0:1];
  logic [1:0] q2 [0:1];
  int q1_r, q2_r;
  always_ff @(posedge clk) begin
    if (!rst_n) begin
      q1[0] <= 2'b11; q1[1] <= 2'b00;
      q2[0] <= 2'b11; q2[1] <= 2'b00;
    end
    else begin
      q1[1][0] <= q1[0][0];  q1[1][1] <= q1[0][1];   // NBA
      q2[1][0]  = q2[0][0];  q2[1][1]  = q2[0][1];   // blocking
    end
  end
  initial begin
    rst_n = 0;
    repeat (2) @(posedge clk);
    #1 rst_n = 1;
    repeat (4) @(posedge clk);
    #1 q1_r = q1[1]; q2_r = q2[1];
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "q1_r"),
        0b11,
        "NBA bit writes into an unpacked element"
    );
    assert_eq!(
        u(&sim, "q2_r"),
        0b11,
        "blocking bit writes into an unpacked element"
    );
}

/// The element itself may be a multi-dim packed array — `[j]` then selects a
/// whole LANE, sized from `packed_signal_elem_widths`, not one bit.
#[test]
fn unpacked_element_lane_write_uses_element_width() {
    let src = r#"
module top;
  bit clk;
  logic rst_n;
  always #5 clk = ~clk;
  logic [1:0][7:0] q [0:1];
  int lane0, lane1;
  always_ff @(posedge clk) begin
    if (!rst_n) begin
      q[0][0] <= 8'hA5; q[0][1] <= 8'h3C;
      q[1][0] <= 8'h00; q[1][1] <= 8'h00;
    end
    else begin
      for (int j = 0; j < 2; j++) q[1][j] <= q[0][j];
    end
  end
  initial begin
    rst_n = 0;
    repeat (2) @(posedge clk);
    #1 rst_n = 1;
    repeat (4) @(posedge clk);
    #1 lane0 = q[1][0]; lane1 = q[1][1];
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "lane0"), 0xA5, "lane 0 carries all 8 bits");
    assert_eq!(u(&sim, "lane1"), 0x3C, "lane 1 carries all 8 bits");
}

/// `arr[i].field[j]` — read AND write of a lane of a packed multi-dim field
/// inside an unpacked array of structs.
#[test]
fn struct_array_member_lane_select_reads_and_writes() {
    let src = r#"
module top;
  typedef struct {
    logic [1:0][7:0] wdata;
    logic [7:0]      other;
  } st_t;
  st_t arr [0:2];
  int rd0, rd1, wr, bits_n, other_r;
  initial begin
    arr[2].wdata = 16'h3CA5;
    arr[1].wdata = 16'h0000;
    arr[1].other = 8'hFF;
    #1;
    rd0 = arr[2].wdata[0];
    rd1 = arr[2].wdata[1];
    bits_n = $bits(arr[2].wdata[0]);
    arr[1].wdata[0] = 8'hA5;
    for (int j = 1; j < 2; j++) arr[1].wdata[j] = 8'h3C;
    #1;
    wr = arr[1].wdata;
    other_r = arr[1].other;
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "rd0"), 0xA5, "lane 0 read, not bit 0");
    assert_eq!(u(&sim, "rd1"), 0x3C, "lane 1 read, not bit 1");
    assert_eq!(
        u(&sim, "bits_n"),
        8,
        "$bits of a lane select is the lane width"
    );
    assert_eq!(u(&sim, "wr"), 0x3CA5, "lane writes assemble the full field");
    assert_eq!(
        u(&sim, "other_r"),
        0xFF,
        "the neighbouring member is untouched"
    );
}
