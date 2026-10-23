//! `@(arr[i])` waits for a change of one unpacked-array element.
//!
//! An array large enough to be stored without per-element names (here
//! 200,000 entries, which puts it in the packed memory arena) used to drop the
//! event control: the element had no signal name, so the waiter armed on
//! nothing and never woke, for a constant and a variable index alike. The
//! same code on a small array works. §9.4.2 makes `@(expr)` an event on the
//! VALUE of `expr`: the waiter now watches the cell the index selects, the
//! index operands too, and re-evaluates the index when they move.
//!
//! Every case runs on a small array and on a 200,000-entry one. Expected values
//! are the reference simulator's.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// The source with `N` replaced by each array size.
fn each_size(src: &str, expect: &[&str]) {
    for n in ["16", "200000"] {
        let o = out(&src.replace("N-1", &format!("{}-1", n)));
        assert_eq!(o, expect, "N={n}: {o:?}");
    }
}

#[test]
fn small_array_element_event_wakes() {
    let o = out(r#"
module top;
  logic [7:0] sm [0:3];
  int hits = 0;
  initial forever @(sm[2]) begin hits++; $display("T|sm t=%0t", $time); end
  initial begin
    #1 sm[1] = 8'h11;
    #1 sm[2] = 8'h44;
    #1 $display("T|hits %0d", hits);
    $finish;
  end
endmodule
"#);
    assert_eq!(o, ["T|sm t=2", "T|hits 1"], "{o:?}");
}

#[test]
fn large_array_element_events_wake() {
    let o = out(r#"
module top;
  logic [7:0] big [0:199999];
  int i = 7;
  int hits_c = 0, hits_v = 0;
  initial forever @(big[5]) begin hits_c++; $display("T|const t=%0t v=%h", $time, big[5]); end
  initial forever @(big[i]) begin hits_v++; $display("T|var t=%0t v=%h", $time, big[i]); end
  initial begin
    #1 big[5] = 8'h11;
    #1 big[7] = 8'h22;
    #1 big[6] = 8'h33;
    #1 $display("T|hits %0d %0d", hits_c, hits_v);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|const t=1 v=11", "T|var t=2 v=22", "T|hits 1 1"],
        "{o:?}"
    );
}

/// §9.4.2: `@(mem[i])` is an event on the expression's value. A change of `i`
/// that selects an element with a different value is itself an event (t=5);
/// one that selects an equal value is not (t=2), but from then on the wait
/// watches the newly selected element (t=4) and no longer the old one (t=3).
#[test]
fn element_event_follows_its_index() {
    each_size(
        r#"
module top;
  logic [7:0] mem [0:N-1];
  int i = 7;
  int hits = 0;
  initial forever @(mem[i]) begin hits++; $display("T|t=%0t i=%0d v=%h", $time, i, mem[i]); end
  initial begin
    #1 mem[5] = 8'h11; mem[7] = 8'h11;
    #1 i = 5;
    #1 mem[7] = 8'h22;
    #1 mem[5] = 8'h33;
    #1 i = 7;
    #1 mem[5] = 8'h44;
    #1 mem[7] = 8'h55;
    #1 $display("T|hits %0d", hits);
    $finish;
  end
endmodule
"#,
        &[
            "T|t=1 i=7 v=11",
            "T|t=4 i=5 v=33",
            "T|t=5 i=7 v=22",
            "T|t=7 i=7 v=55",
            "T|hits 4",
        ],
    );
}

/// Edges and part-selects of an element: the edge is judged on the selected
/// bit, not on the element's LSB; `wait` on an element compares its value.
#[test]
fn element_bit_and_part_select_events() {
    each_size(
        r#"
module top;
  logic [7:0] mem [0:N-1];
  int i = 7, j = 9;
  initial forever @(posedge mem[i][0]) $display("T|pos t=%0t v=%h", $time, mem[i]);
  initial forever @(negedge mem[3][1]) $display("T|neg t=%0t v=%h", $time, mem[3]);
  initial forever @(mem[2][5:4]) $display("T|part t=%0t v=%h", $time, mem[2]);
  initial begin wait (mem[j] == 8'h22); $display("T|wait t=%0t", $time); end
  initial begin
    #1 mem[7] = 8'h01;
    #1 mem[7] = 8'h03;
    #1 mem[7] = 8'h02;
    #1 mem[7] = 8'h03;
    #1 mem[9] = 8'h22;
    #1 mem[3] = 8'h02;
    #1 mem[3] = 8'h00;
    #1 mem[2] = 8'h0f;
    #1 mem[2] = 8'h1f;
    #1 mem[2] = 8'h10;
    #1 $display("T|done");
    $finish;
  end
endmodule
"#,
        &[
            "T|pos t=1 v=01",
            "T|pos t=4 v=03",
            "T|wait t=5",
            "T|neg t=7 v=00",
            "T|part t=8 v=0f",
            "T|part t=9 v=1f",
            "T|done",
        ],
    );
}

/// The same event lists heading `always` blocks. On a large array these were
/// dropped with a "DROPPED unresolvable sensitivity term" warning; `@(mem[i])`
/// was dropped on a small array too.
#[test]
fn always_blocks_on_array_elements() {
    each_size(
        r#"
module top;
  logic [7:0] mem [0:N-1];
  int i = 7;
  int ha = 0, hb = 0, hc = 0;
  always @(mem[5]) begin ha++; $display("T|a5 t=%0t v=%h", $time, mem[5]); end
  always @(mem[i]) begin hb++; $display("T|ai t=%0t v=%h", $time, mem[i]); end
  always @(posedge mem[6][2]) begin hc++; $display("T|p62 t=%0t v=%h", $time, mem[6]); end
  initial begin
    #1 mem[5] = 8'h11;
    #1 mem[7] = 8'h22;
    #1 mem[6] = 8'h04;
    #1 mem[6] = 8'h01;
    #1 mem[6] = 8'h05;
    #1 i = 6;
    #1 mem[6] = 8'h06;
    #1 $display("T|hits %0d %0d %0d", ha, hb, hc);
    $finish;
  end
endmodule
"#,
        &[
            "T|a5 t=1 v=11",
            "T|ai t=2 v=22",
            "T|p62 t=3 v=04",
            "T|p62 t=5 v=05",
            "T|ai t=6 v=05",
            "T|ai t=7 v=06",
            "T|hits 1 3 2",
        ],
    );
}

/// Continuous assignments and combinational blocks reading an element follow
/// writes to it, for a constant and a variable index.
#[test]
fn combinational_readers_of_elements() {
    each_size(
        r#"
module top;
  logic [7:0] mem [0:N-1];
  int i = 7;
  logic [7:0] ya, yb, yc, yd, ye;
  assign ya = mem[5] ^ 8'hff;
  assign yb = mem[i];
  always @* yc = mem[i];
  always_comb yd = mem[5] + mem[6];
  always_comb ye = mem[i] ^ 8'h0f;
  initial begin
    #1 mem[5] = 8'h11; mem[6] = 8'h01;
    #1 $display("T|a %h %h %h %h %h", ya, yb, yc, yd, ye);
    mem[7] = 8'h22;
    #1 $display("T|b %h %h %h %h %h", ya, yb, yc, yd, ye);
    i = 5;
    #1 $display("T|c %h %h %h %h %h", ya, yb, yc, yd, ye);
    mem[5] = 8'h33;
    #1 $display("T|d %h %h %h %h %h", ya, yb, yc, yd, ye);
    $finish;
  end
endmodule
"#,
        &[
            "T|a ee xx xx 12 xx",
            "T|b ee 22 22 12 2d",
            "T|c ee 11 11 12 1e",
            "T|d cc 33 33 34 3c",
        ],
    );
}

/// Writers other than a blocking assignment: an NBA, `$readmemh`, and a task
/// `ref` argument all wake the element's waiter.
#[test]
fn element_events_from_every_writer() {
    let dir = std::env::temp_dir().join(format!("xezim_elem_writers_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let hex = dir.join("mem.hex");
    std::fs::write(&hex, "ab\n").unwrap();
    let src = r#"
module top;
  logic [7:0] mem [0:N-1];
  int h5 = 0, h8 = 0, h9 = 0;
  initial forever @(mem[5]) begin h5++; $display("T|w5 t=%0t v=%h", $time, mem[5]); end
  initial forever @(mem[8]) begin h8++; $display("T|w8 t=%0t v=%h", $time, mem[8]); end
  initial forever @(mem[9]) begin h9++; $display("T|w9 t=%0t v=%h", $time, mem[9]); end
  task automatic setr(ref logic [7:0] r, input logic [7:0] v);
    r = v;
  endtask
  initial begin
    #1 mem[5] <= 8'h11;
    #1 $readmemh("HEX", mem, 8, 8);
    #1 setr(mem[9], 8'h99);
    #1 $display("T|hits %0d %0d %0d", h5, h8, h9);
    $finish;
  end
endmodule
"#
    .replace("HEX", hex.to_str().unwrap());
    each_size(
        &src,
        &[
            "T|w5 t=1 v=11",
            "T|w8 t=2 v=ab",
            "T|w9 t=3 v=99",
            "T|hits 1 1 1",
        ],
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A large memory inside instances (non-ANSI and ANSI ports), waited on from
/// the instance itself and hierarchically from the top.
#[test]
fn large_array_element_events_in_instances() {
    let o = out(r#"
module mem_nonansi(i);
  input int i;
  logic [7:0] big [0:199999];
  initial forever @(big[5]) $display("T|sub_c t=%0t v=%h", $time, big[5]);
  initial forever @(big[i]) $display("T|sub_v t=%0t v=%h", $time, big[i]);
endmodule
module mem_ansi(input int i);
  logic [15:0] big [0:199999];
  always @(big[i]) $display("T|ansi_v t=%0t v=%h", $time, big[i]);
endmodule
module top;
  int i = 7;
  mem_nonansi u(i);
  mem_ansi a(i);
  initial forever @(u.big[9]) $display("T|hier t=%0t v=%h", $time, u.big[9]);
  initial begin
    #1 u.big[5] = 8'h11;
    #1 u.big[7] = 8'h22;
    #1 u.big[9] = 8'h99;
    #1 a.big[7] = 16'h7777;
    #1 i = 9;
    #1 a.big[9] = 16'h9999;
    #1 $display("T|done");
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|sub_c t=1 v=11",
            "T|sub_v t=2 v=22",
            "T|hier t=3 v=99",
            "T|ansi_v t=4 v=7777",
            "T|ansi_v t=5 v=xxxx",
            "T|sub_v t=5 v=99",
            "T|ansi_v t=6 v=9999",
            "T|done",
        ],
        "{o:?}"
    );
}

/// A large array of 128-bit elements is too wide for the packed arena and
/// stays a bulk array without element names; it used to panic a waiter on an
/// array declared after it (`sm`), whose ids then ran past the edge snapshot.
#[test]
fn wide_large_array_and_the_arrays_after_it() {
    let o = out(r#"
module top;
  logic [127:0] big [0:199999];
  logic [7:0] sm [0:3];
  int i = 7;
  int hits_c = 0, hits_v = 0, hits_s = 0, hits_a = 0;
  initial forever @(big[5]) begin hits_c++; $display("T|const t=%0t v=%h", $time, big[5]); end
  initial forever @(big[i]) begin hits_v++; $display("T|var t=%0t v=%h", $time, big[i]); end
  initial forever @(sm[2]) begin hits_s++; $display("T|sm t=%0t", $time); end
  always @(posedge sm[1][0]) begin hits_a++; $display("T|asm t=%0t", $time); end
  initial begin
    #1 big[5] = 128'h11;
    #1 big[7] = 128'h22;
    #1 big[6] = 128'h33;
    #1 sm[2] = 8'h44;
    #1 sm[1] = 8'h01;
    #1 $display("T|hits %0d %0d %0d %0d", hits_c, hits_v, hits_s, hits_a);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|const t=1 v=00000000000000000000000000000011",
            "T|var t=2 v=00000000000000000000000000000022",
            "T|sm t=4",
            "T|asm t=5",
            "T|hits 1 1 1 1",
        ],
        "{o:?}"
    );
}

/// The same value rule on a plain vector: an edge on `v[3]` is an edge of bit
/// 3 (not of `v`'s LSB), `@(v[5:4])` ignores changes outside bits 5:4, and a
/// moved index re-selects the bit.
#[test]
fn vector_select_events() {
    let o = out(r#"
module top;
  logic [7:0] v = 0;
  int i = 3;
  initial forever @(posedge v[3]) $display("T|pos3 t=%0t v=%h", $time, v);
  initial forever @(v[5:4]) $display("T|rng t=%0t v=%h", $time, v);
  initial forever @(negedge v[i]) $display("T|negi t=%0t v=%h", $time, v);
  initial begin
    #1 v = 8'h01;
    #1 v = 8'h08;
    #1 v = 8'h04;
    #1 v = 8'h10;
    #1 i = 4;
    #1 v = 8'h00;
    #1 v = 8'h30;
    #1 $display("T|done");
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "T|pos3 t=2 v=08",
            "T|negi t=3 v=04",
            "T|rng t=4 v=10",
            "T|rng t=6 v=00",
            "T|negi t=6 v=00",
            "T|rng t=7 v=30",
            "T|done",
        ],
        "{o:?}"
    );
}

/// Two-state elements (no x/z plane in the arena).
#[test]
fn two_state_large_array_element_events() {
    let o = out(r#"
module top;
  bit [7:0] big [0:199999];
  int i = 7;
  int hits_c = 0, hits_v = 0;
  initial forever @(big[5]) begin hits_c++; $display("T|const t=%0t v=%h", $time, big[5]); end
  initial forever @(big[i]) begin hits_v++; $display("T|var t=%0t v=%h", $time, big[i]); end
  initial begin
    #1 big[5] = 8'h11;
    #1 big[7] = 8'h22;
    #1 big[6] = 8'h33;
    #1 $display("T|hits %0d %0d", hits_c, hits_v);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|const t=1 v=11", "T|var t=2 v=22", "T|hits 1 1"],
        "{o:?}"
    );
}

/// The index is a task formal or an automatic loop variable. These waits used
/// to return at once, on either array size.
#[test]
fn element_event_with_a_local_index() {
    each_size(
        r#"
module top;
  logic [7:0] mem [0:N-1];
  task automatic wait_elem(input int a);
    @(mem[a]);
    $display("T|task t=%0t a=%0d v=%h", $time, a, mem[a]);
  endtask
  initial begin
    for (int k = 2; k < 4; k++) begin
      @(posedge mem[k][0]);
      $display("T|loop t=%0t k=%0d v=%h", $time, k, mem[k]);
    end
  end
  initial begin
    #1 wait_elem(9);
    wait_elem(10);
  end
  initial begin
    #2 mem[9] = 8'h09;
    #1 mem[2] = 8'h01;
    #1 mem[10] = 8'h0a;
    #1 mem[3] = 8'h02;
    #1 mem[3] = 8'h03;
    #1 $display("T|done");
    $finish;
  end
endmodule
"#,
        &[
            "T|task t=2 a=9 v=09",
            "T|loop t=3 k=2 v=01",
            "T|task t=4 a=10 v=0a",
            "T|loop t=6 k=3 v=03",
            "T|done",
        ],
    );
}

/// A select whose index is a parameter expression (`v[P-1]`, §6.20) is a
/// constant bit: the edge is that bit's, in an always block and a process.
/// The always block used to watch `v`'s LSB instead.
#[test]
fn parameter_indexed_select_events() {
    let o = out(r#"
module sub(input logic [3:0] v);
  localparam P = 2;
  int ha = 0, hp = 0;
  always @(posedge v[P-1]) begin ha++; $display("T|a t=%0t v=%h", $time, v); end
  initial forever @(posedge v[P]) begin hp++; $display("T|p t=%0t v=%h", $time, v); end
endmodule
module top;
  logic [3:0] v = 0;
  sub u(v);
  initial begin
    #1 v = 4'h1;
    #1 v = 4'h2;
    #1 v = 4'h0;
    #1 v = 4'h4;
    #1 v = 4'h6;
    #1 v = 4'h7;
    #1 $display("T|hits %0d %0d", u.ha, u.hp);
    $finish;
  end
endmodule
"#);
    assert_eq!(
        o,
        ["T|a t=2 v=2", "T|p t=4 v=4", "T|a t=5 v=6", "T|hits 2 1"],
        "{o:?}"
    );
}
