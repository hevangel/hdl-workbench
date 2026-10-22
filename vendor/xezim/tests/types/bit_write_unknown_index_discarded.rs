//! §11.5.1 — a bit-select WRITE at an unknown or out-of-range index modifies
//! no bits. The write-side sibling of `array_read_unknown_index_is_x`, and it
//! had the same cause: `to_u64().unwrap_or(0)`.
//!
//! `Value::to_u64` returns `Some(val_bits & !xz_bits)` — it masks x bits to
//! ZERO and never returns `None` — so every dynamic bit-store treated an
//! unknown index as bit 0 and wrote there. Out-of-range indices were already
//! handled, because `Value::set_bit` drops them, which is why a constant
//! out-of-range index behaved correctly all along and hid the shape of the
//! bug.
//!
//! An `initial` block and a task got this right, so the defect only showed
//! from inside an always block. That is the shape a wormhole arbiter uses to
//! build its per-input ready:
//!
//! ```systemverilog
//! always_comb begin
//!   ready_o = '0;
//!   ready_o[selected_idx] = (|valid_i) ? ready_i : '0;
//! end
//! ```
//!
//! At reset `valid_i` is x, so `selected_idx` is x and bit 0 of `ready_o`
//! picked up an x that the LRM says must never be written. Downstream that x
//! reached a crossbar's ready and stalled the router forever.
//!
//! Note that Verilator cannot referee this: it is 2-state here, resolves the
//! index to 0 and performs the write. The expected values below are the LRM's.

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 200).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn unknown_index_bit_write_is_discarded() {
    const SRC: &str = r#"
module top;
  logic [2:0] sel;            // never assigned -> stays x
  logic [4:0] a, b, c, d;
  logic clk = 0;
  always #5 clk = ~clk;

  always_comb begin           // fill, then an x-index write
    a = '0;
    a[sel] = 1'b1;
  end
  always_comb begin           // x-index write with no fill: stays all-x
    b[sel] = 1'b1;
  end
  always_ff @(posedge clk) begin
    c <= '0;
    c[sel] <= 1'b1;
  end
  initial begin               // the same write outside an always block
    d = '0;
    d[sel] = 1'b1;
  end

  initial begin
    #12;
    $display("A %b B %b C %b D %b", a, b, c, d);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("A 00000 B xxxxx C 00000 D 00000"),
        "an x index must modify no bits, in always_comb, always_ff and initial \
         alike:\n{}",
        o
    );
}

/// The neighbouring index forms must keep working: a known in-range index
/// still writes, and a constant out-of-range one is still dropped.
#[test]
fn known_and_out_of_range_indices_unchanged() {
    const SRC: &str = r#"
module top;
  logic [4:0] e, f, g;
  logic [2:0] two = 3'd2;
  always_comb begin
    e = '0;
    e[two] = 1'b1;            // dynamic, known, in range
  end
  always_comb begin
    f = '0;
    f[3'd6] = 1'b1;           // constant, out of range
  end
  always_comb begin
    g = '0;
    g[3'd4] = 1'b1;           // constant, in range
  end
  initial begin
    #1 $display("E %b F %b G %b", e, f, g);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("E 00100 F 00000 G 10000"),
        "index forms that already worked:\n{}",
        o
    );
}

/// The arbiter shape the bug was found in: once any input is valid the index
/// resolves and the write lands normally, so the fix must not suppress the
/// real write.
#[test]
fn arbiter_ready_resolves_once_valid_is_known() {
    const SRC: &str = r#"
module top;
  logic [4:0] valid_i;
  logic [2:0] sel;
  logic [4:0] ready_o;
  always_comb begin
    ready_o = '0;
    ready_o[sel] = (|valid_i) ? 1'b1 : '0;
  end
  initial begin
    #1 $display("RESET %b", ready_o);       // valid_i and sel are x
    valid_i = 5'b00100;
    sel     = 3'd2;
    #1 $display("VALID %b", ready_o);
    $finish;
  end
endmodule
"#;
    let o = out(SRC);
    assert!(
        o.contains("RESET 00000"),
        "no bit written while the index is x:\n{}",
        o
    );
    assert!(
        o.contains("VALID 00100"),
        "the real write still lands:\n{}",
        o
    );
}
