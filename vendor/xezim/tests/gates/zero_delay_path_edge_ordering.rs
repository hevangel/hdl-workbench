//! §4.4, §30.4: a module path whose delay rounds to zero ticks is no delay.
//!
//! A library clock cell (`buf` plus a specify path `(posedge A => (Y:1'b1)) =
//! (0.01, 100.0)` at 1ps precision) passes the clock through with a zero rise
//! delay. The edge it raises on `Y` belongs to the same Active region as the
//! clock edge, so a flop clocked by `Y` samples the value a flop clocked by the
//! clock is about to change (pre-NBA). xezim queued the zero-delay update and
//! applied it on the next pass of the time step, after that step's NBAs had
//! committed, so the capture flop sampled the new value.
//!
//! A non-zero rise delay still moves the edge to a later time slot (control).
//! Expected values are the reference simulator's. Lines from the same time step
//! are checked as a set: IEEE 1800 leaves the order of same-time `always`
//! blocks open.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn has_all(o: &[String], want: &[&str]) {
    for w in want {
        assert!(o.iter().any(|l| l == w), "missing `{w}`: {o:?}");
    }
    assert!(!o.iter().any(|l| l.contains("FAIL")), "{o:?}");
}

const USER: &str = r##"`timescale 1ps/1ps

// Library-style clock hop: edge-sensitive module path (posedge A => Y),
// rise delay 0.01 -> 0 ticks at 1ps precision.
module cell_hop (Y, A);
  output Y;
  input  A;
  buf u0 (Y, A);
  specify
    (posedge A => (Y:1'b1)) = (0.01, 100.0);
  endspecify
endmodule

module tb_top;
  reg clk;
  reg q1;   // launch flop: toggles via NBA at each clk edge
  reg q;    // capture flop, clocked through the specify-path hop (starts x)
  wire y;

  initial begin
    clk = 1'b0;
    forever #5 clk = ~clk;          // base clock: blocking toggle (Active region)
  end
  always @(posedge clk) q1 <= ~q1;  // q1 commits via NBA at the same timestamp
  cell_hop hop (.A(clk), .Y(y));
  always @(posedge y) q <= q1;      // must sample the pre-NBA value of q1

  initial begin
    q1 = 1'b0;
    #6;
    if (q === 1'b0) $display("TEST_PASS");
    else begin
      $display("FAIL @%0t : capture must sample pre-NBA q1 (q=%b)", $time, q);
      $display("TEST_FAIL count=1");
      $fatal(1);
    end
    $finish;
  end
endmodule
"##;
const TRACED: &str = r##"`timescale 1ps/1ps

// Library-style clock hop: edge-sensitive module path (posedge A => Y),
// rise delay 0.01 -> 0 ticks at 1ps precision.
module cell_hop (Y, A);
  output Y;
  input  A;
  buf u0 (Y, A);
  specify
    (posedge A => (Y:1'b1)) = (0.01, 100.0);
  endspecify
endmodule

module tb_top;
  reg clk;
  reg q1;   // launch flop: toggles via NBA at each clk edge
  reg q;    // capture flop, clocked through the specify-path hop (starts x)
  wire y;

  initial begin
    clk = 1'b0;
    forever #5 clk = ~clk;          // base clock: blocking toggle (Active region)
  end
  always @(posedge clk) q1 <= ~q1;  // q1 commits via NBA at the same timestamp
  cell_hop hop (.A(clk), .Y(y));
  always @(posedge y) begin q <= q1; $display("T| posedge y at %0t q1=%b", $time, q1); end
  always @(posedge clk) $display("T| posedge clk at %0t q1=%b", $time, q1);
  always @(q) $display("T| q=%b at %0t", q, $time);

  initial begin
    q1 = 1'b0;
    #6;
    if (q === 1'b0) $display("TEST_PASS");
    else begin
      $display("FAIL @%0t : capture must sample pre-NBA q1 (q=%b)", $time, q);
      $display("TEST_FAIL count=1");
      $fatal(1);
    end
    $finish;
  end
endmodule
"##;
const EXPLICIT_ZERO: &str = r##"`timescale 1ps/1ps

// Library-style clock hop: edge-sensitive module path (posedge A => Y),
// rise delay 0.01 -> 0 ticks at 1ps precision.
module cell_hop (Y, A);
  output Y;
  input  A;
  buf u0 (Y, A);
  specify
    (posedge A => (Y:1'b1)) = (0, 100);
  endspecify
endmodule

module tb_top;
  reg clk;
  reg q1;   // launch flop: toggles via NBA at each clk edge
  reg q;    // capture flop, clocked through the specify-path hop (starts x)
  wire y;

  initial begin
    clk = 1'b0;
    forever #5 clk = ~clk;          // base clock: blocking toggle (Active region)
  end
  always @(posedge clk) q1 <= ~q1;  // q1 commits via NBA at the same timestamp
  cell_hop hop (.A(clk), .Y(y));
  always @(posedge y) begin q <= q1; $display("T| posedge y at %0t q1=%b", $time, q1); end
  always @(posedge clk) $display("T| posedge clk at %0t q1=%b", $time, q1);
  always @(q) $display("T| q=%b at %0t", q, $time);

  initial begin
    q1 = 1'b0;
    #6;
    if (q === 1'b0) $display("TEST_PASS");
    else begin
      $display("FAIL @%0t : capture must sample pre-NBA q1 (q=%b)", $time, q);
      $display("TEST_FAIL count=1");
      $fatal(1);
    end
    $finish;
  end
endmodule
"##;
const PLAIN_PATH: &str = r##"`timescale 1ps/1ps

// Library-style clock hop: edge-sensitive module path (posedge A => Y),
// rise delay 0.01 -> 0 ticks at 1ps precision.
module cell_hop (Y, A);
  output Y;
  input  A;
  buf u0 (Y, A);
  specify
    (A => Y) = (0.01, 100.0);
  endspecify
endmodule

module tb_top;
  reg clk;
  reg q1;   // launch flop: toggles via NBA at each clk edge
  reg q;    // capture flop, clocked through the specify-path hop (starts x)
  wire y;

  initial begin
    clk = 1'b0;
    forever #5 clk = ~clk;          // base clock: blocking toggle (Active region)
  end
  always @(posedge clk) q1 <= ~q1;  // q1 commits via NBA at the same timestamp
  cell_hop hop (.A(clk), .Y(y));
  always @(posedge y) begin q <= q1; $display("T| posedge y at %0t q1=%b", $time, q1); end
  always @(posedge clk) $display("T| posedge clk at %0t q1=%b", $time, q1);
  always @(q) $display("T| q=%b at %0t", q, $time);

  initial begin
    q1 = 1'b0;
    #6;
    if (q === 1'b0) $display("TEST_PASS");
    else begin
      $display("FAIL @%0t : capture must sample pre-NBA q1 (q=%b)", $time, q);
      $display("TEST_FAIL count=1");
      $fatal(1);
    end
    $finish;
  end
endmodule
"##;
const TWO_HOPS: &str = r##"`timescale 1ps/1ps

// Library-style clock hop: edge-sensitive module path (posedge A => Y),
// rise delay 0.01 -> 0 ticks at 1ps precision.
module cell_hop (Y, A);
  output Y;
  input  A;
  buf u0 (Y, A);
  specify
    (posedge A => (Y:1'b1)) = (0.01, 100.0);
  endspecify
endmodule

module tb_top;
  reg clk;
  reg q1;   // launch flop: toggles via NBA at each clk edge
  reg q;    // capture flop, clocked through the specify-path hop (starts x)
  wire y;

  initial begin
    clk = 1'b0;
    forever #5 clk = ~clk;          // base clock: blocking toggle (Active region)
  end
  always @(posedge clk) q1 <= ~q1;  // q1 commits via NBA at the same timestamp
  wire ym;
  cell_hop hop0 (.A(clk), .Y(ym));
  cell_hop hop (.A(ym), .Y(y));
  always @(posedge y) begin q <= q1; $display("T| posedge y at %0t q1=%b", $time, q1); end
  always @(posedge clk) $display("T| posedge clk at %0t q1=%b", $time, q1);
  always @(q) $display("T| q=%b at %0t", q, $time);

  initial begin
    q1 = 1'b0;
    #6;
    if (q === 1'b0) $display("TEST_PASS");
    else begin
      $display("FAIL @%0t : capture must sample pre-NBA q1 (q=%b)", $time, q);
      $display("TEST_FAIL count=1");
      $fatal(1);
    end
    $finish;
  end
endmodule
"##;
const RISE_ONE: &str = r##"`timescale 1ps/1ps

// Library-style clock hop: edge-sensitive module path (posedge A => Y),
// rise delay 0.01 -> 0 ticks at 1ps precision.
module cell_hop (Y, A);
  output Y;
  input  A;
  buf u0 (Y, A);
  specify
    (posedge A => (Y:1'b1)) = (1, 100);
  endspecify
endmodule

module tb_top;
  reg clk;
  reg q1;   // launch flop: toggles via NBA at each clk edge
  reg q;    // capture flop, clocked through the specify-path hop (starts x)
  wire y;

  initial begin
    clk = 1'b0;
    forever #5 clk = ~clk;          // base clock: blocking toggle (Active region)
  end
  always @(posedge clk) q1 <= ~q1;  // q1 commits via NBA at the same timestamp
  cell_hop hop (.A(clk), .Y(y));
  always @(posedge y) begin q <= q1; $display("T| posedge y at %0t q1=%b", $time, q1); end
  always @(posedge clk) $display("T| posedge clk at %0t q1=%b", $time, q1);
  always @(q) $display("T| q=%b at %0t", q, $time);

  initial begin
    q1 = 1'b0;
    #8;
    if (q === 1'b1) $display("TEST_PASS");
    else begin
      $display("FAIL @%0t : capture must sample pre-NBA q1 (q=%b)", $time, q);
      $display("TEST_FAIL count=1");
      $fatal(1);
    end
    $finish;
  end
endmodule
"##;

#[test]
fn zero_delay_edge_path_clock_samples_pre_nba() {
    has_all(&out(USER), &["TEST_PASS"]);
}

#[test]
fn zero_delay_hop_edge_runs_in_the_clock_edge_active_region() {
    has_all(
        &out(TRACED),
        &[
            "T| posedge clk at 5 q1=0",
            "T| posedge y at 5 q1=0",
            "T| q=0 at 5",
            "TEST_PASS",
        ],
    );
}

#[test]
fn explicit_zero_and_plain_path_delays() {
    for src in [EXPLICIT_ZERO, PLAIN_PATH] {
        has_all(&out(src), &["T| posedge y at 5 q1=0", "TEST_PASS"]);
    }
}

#[test]
fn two_zero_delay_hops_in_series() {
    has_all(
        &out(TWO_HOPS),
        &["T| posedge y at 5 q1=0", "T| q=0 at 5", "TEST_PASS"],
    );
}

#[test]
fn non_zero_rise_delay_moves_the_edge_to_a_later_slot() {
    has_all(
        &out(RISE_ONE),
        &["T| posedge y at 6 q1=1", "T| q=1 at 6", "TEST_PASS"],
    );
}
