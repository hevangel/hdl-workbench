//! §9.4.2.3 / §14.13 — the `iff` guard of a CLOCKING event reads the
//! clockvars sampled at THAT clocking event (#317, reported by urbanij).
//!
//! The clockvars update when the clocking event occurs, before the
//! processes waiting on it resume, so `@(cb iff (cb.cs_n === 0))` completes
//! on the first edge at which `cb.cs_n` SAMPLES 0. The guard used to be
//! judged with the raw clock edge, before the block sampled, so it read the
//! previous edge's value and the wait completed one cycle late.
//!
//! A raw `@(posedge clk iff cb.x)` is NOT a clocking event: its guard is
//! judged at the edge and still reads the previous sample. Expected values
//! are the reference simulator's.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

const IFACE: &str = r#"
`timescale 1ns/1ns
interface bus_if (input logic clk);
  logic        cs_n;
  logic [15:0] data;
  logic        o;
  int          t_cls, t_cls_d;
  logic [15:0] d_cls;
  clocking cb @(posedge clk);
    default input #1step output #0;
    input cs_n;
    input data;
    output o;
  endclocking
  clocking cb0 @(posedge clk);
    input #0 cs_n;
    input #0 data;
  endclocking
  clocking cb2 @(posedge clk);
    input #2 cs_n;
    input #2 data;
  endclocking
endinterface

class mon_c;
  virtual bus_if vif;
  task run();
    @(vif.cb iff (vif.cb.cs_n === 1'b0));
    vif.t_cls = $time;
    vif.d_cls = vif.cb.data;
  endtask
  task run_d();
    @(vif.cb iff (vif.cb.data == 16'h0001));
    vif.t_cls_d = $time;
  endtask
endclass
"#;

/// The issue's shape (a class task waiting through a virtual interface)
/// and its neighbours: module scope, `#0` / `#2` input skews, a guard on a
/// plain signal written by a same-edge NBA, a raw-edge guard reading a
/// clockvar, a clocking drive issued after the wait, and a repeat count.
#[test]
fn clocking_event_iff_reads_this_edge_samples() {
    let src = format!(
        "{IFACE}{}",
        r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus (clk);
  logic nb = 0, pre = 0;
  int t_mod, t_mod_d, t_skew0, t_skew2, t_pre, t_nba, t_raw_cb, t_raw_sig, t_out, t_rep;
  logic [15:0] d_mod, d_skew0, d_skew2, d_rep;
  always @(posedge clk) if ($time == 25) nb <= 1;
  initial begin
    bus.cs_n = 1'b1;
    bus.data = 16'h0000;
    bus.o = 0;
    #20;
    bus.cs_n = 1'b0;
    bus.data = 16'hf000;
    pre = 1;
    #4 bus.data = 16'hf00f;
    #6 bus.data = 16'h0001;
    #10 bus.data = 16'h0002;
    #10 bus.cs_n = 1'b1;
  end
  initial begin
    @(bus.cb iff (bus.cb.cs_n === 1'b0));
    t_mod = $time; d_mod = bus.cb.data;
  end
  initial begin
    @(bus.cb iff (bus.cb.data == 16'h0001));
    t_mod_d = $time;
  end
  initial begin
    @(bus.cb0 iff (bus.cb0.cs_n === 1'b0));
    t_skew0 = $time; d_skew0 = bus.cb0.data;
  end
  initial begin
    @(bus.cb2 iff (bus.cb2.data === 16'hf000));
    t_skew2 = $time; d_skew2 = bus.cb2.data;
  end
  initial begin
    @(bus.cb iff pre);
    t_pre = $time;
  end
  initial begin
    @(bus.cb iff nb);
    t_nba = $time;
  end
  initial begin
    @(posedge clk iff (bus.cb.cs_n === 1'b0));
    t_raw_cb = $time;
  end
  initial begin
    @(posedge clk iff (bus.cs_n === 1'b0));
    t_raw_sig = $time;
  end
  initial begin
    @(bus.cb iff (bus.cb.cs_n === 1'b0));
    bus.cb.o <= 1'b1;
    @(bus.o);
    t_out = $time;
  end
  initial begin
    repeat (2) @(bus.cb iff (bus.cb.cs_n === 1'b0));
    t_rep = $time; d_rep = bus.cb.data;
  end
  initial begin
    automatic mon_c m = new();
    automatic mon_c m2 = new();
    m.vif = bus;
    m2.vif = bus;
    fork
      m.run();
      m2.run_d();
    join_none
    #100 $finish;
  end
endmodule
"#
    );
    let sim = simulate(&src, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "bus.t_cls"),
        25,
        "class task via vif wakes on the first sampled 0"
    );
    assert_eq!(u(&sim, "bus.d_cls"), 0xf00f, "and reads that edge's sample");
    assert_eq!(u(&sim, "bus.t_cls_d"), 35);
    assert_eq!(u(&sim, "t_mod"), 25, "module initial");
    assert_eq!(u(&sim, "d_mod"), 0xf00f);
    assert_eq!(u(&sim, "t_mod_d"), 35);
    assert_eq!(u(&sim, "t_skew0"), 25, "#0 input skew");
    assert_eq!(u(&sim, "d_skew0"), 0xf00f);
    assert_eq!(u(&sim, "t_skew2"), 25, "#2 input skew samples at 23");
    assert_eq!(u(&sim, "d_skew2"), 0xf000);
    assert_eq!(u(&sim, "t_pre"), 25, "guard on a plain signal");
    assert_eq!(u(&sim, "t_nba"), 25, "guard sees the same-edge NBA");
    assert_eq!(
        u(&sim, "t_raw_cb"),
        35,
        "raw-edge guard reads the previous sample"
    );
    assert_eq!(u(&sim, "t_raw_sig"), 25);
    assert_eq!(u(&sim, "t_out"), 25, "clocking drive after the wait");
    assert_eq!(u(&sim, "t_rep"), 35, "repeat (2) counts guarded events");
    assert_eq!(u(&sim, "d_rep"), 0x0001);
}

/// A wait reached in the same slot after the raw edge (`@(posedge clk)`
/// then `@(cb iff g)`) still sees this slot's clocking event — and its
/// guard is still judged; a guard that never holds never completes.
#[test]
fn clocking_event_iff_due_now_is_still_guarded() {
    let src = format!(
        "{IFACE}{}",
        r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus (clk);
  int t_due = -1, t_never = -1;
  logic [15:0] d_due;
  initial begin
    bus.cs_n = 1'b1;
    bus.data = 16'h0000;
    #20;
    bus.cs_n = 1'b0;
    bus.data = 16'hf000;
    #10 bus.data = 16'h0001;
  end
  initial begin
    @(posedge clk iff (bus.cs_n === 1'b0));
    @(bus.cb iff (bus.cb.data == 16'hf000));
    t_due = $time; d_due = bus.cb.data;
    @(posedge clk);
    @(bus.cb iff (bus.cb.data == 16'hffff));
    t_never = $time;
  end
  initial #100 $finish;
endmodule
"#
    );
    let sim = simulate(&src, 200).expect("simulate failed");
    assert_eq!(u(&sim, "t_due"), 25);
    assert_eq!(u(&sim, "d_due"), 0xf000);
    assert_eq!(
        u(&sim, "t_never") as i32,
        -1,
        "a false guard never completes the wait"
    );
}

fn sorted_t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    let mut v: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect();
    v.sort();
    v
}

/// Sibling shapes of the clocking-event `iff`: a negedge clocking block, an
/// interface task, a guard mixing a clockvar and a plain signal, default
/// clocking with `##N` after the wait, a `forever` loop, a `##1` clocking
/// drive, a guard true at the first edge and an `or` list.
#[test]
fn clocking_event_iff_sibling_shapes() {
    assert_eq!(sorted_t_lines(SRC_SIBLINGS), EXPECT_SIBLINGS);
}

/// §14.13: a clockvar's first sample (x to its value) is a change, so
/// `@(cb.sig)` completes at the first clocking event. The block lives in an
/// interface instance, whose clockvars are named by their inlined path
/// (`bus.data`); `@(bus.cb.data)` used to miss their sample mirror and wait
/// on nothing.
#[test]
fn clockvar_first_sample_event_and_join_any_item() {
    let want: Vec<&str> = EXPECT_FIRST_SAMPLE
        .iter()
        .copied()
        .filter(|l| !l.contains("fork_any"))
        .collect();
    assert_eq!(
        sorted_t_lines(&SRC_FIRST_SAMPLE.replace(FORK_ANY_BLOCK, "")),
        want
    );
}

/// Deliberate divergence (§9.4.2.3): the reference simulator completes a
/// `join_any` whose item is a bare `@(cb iff g);` at the next clocking
/// event even though `g` is false there — it drops the guard of a
/// null-statement event item in a fork of two or more items (an item with a
/// statement, or a lone item, keeps it). xezim follows the standard and
/// completes the item when the guard holds (t=35).
#[test]
#[ignore = "deliberate divergence: the reference simulator ignores the iff guard of a bare @(cb iff g); item of a multi-item fork (§9.4.2.3)"]
fn join_any_bare_clocking_iff_item() {
    assert_eq!(sorted_t_lines(SRC_FIRST_SAMPLE), EXPECT_FIRST_SAMPLE);
}

/// §24.3 / §4.4: program processes resume in the Reactive region, after
/// the slot's clocking event, so `@(posedge clk); @(cb);` in a program
/// waits for the NEXT clocking event (a module process sees this one).
#[test]
fn program_clocking_wait_after_raw_edge() {
    assert_eq!(sorted_t_lines(SRC_PROGRAM), EXPECT_PROGRAM);
}

/// §24.3 / §4.4 neighbours: a program process resumed by an edge reads
/// the slot's NBA results, a `#1` / `#0` / `wait` / `@(cb iff)` in a
/// program, and a program output port driving a module `always`.
#[test]
fn program_reactive_region_shapes() {
    assert_eq!(sorted_t_lines(SRC_PROGRAM2), EXPECT_PROGRAM2);
}

/// §14.13: `@(bus.cb.sig)` of an interface's clocking block, armed after
/// the first sample, and an `always @(bus.cb.sig)`.
#[test]
fn interface_clockvar_event_shapes() {
    assert_eq!(sorted_t_lines(SRC_IFACE_CBVAR), EXPECT_IFACE_CBVAR);
}

const SRC_SIBLINGS: &str = r#"
`timescale 1ns/1ns
interface bus_if (input logic clk);
  logic        cs_n;
  logic [15:0] data;
  logic        o;
  clocking cb @(posedge clk);
    default input #1step output #0;
    input cs_n;
    input data;
    output o;
  endclocking
  clocking ncb @(negedge clk);
    input cs_n;
    input data;
  endclocking
  clocking cb1 @(posedge clk);
    input #1 cs_n;
    input #1 data;
  endclocking
  task automatic wait_low();
    @(cb iff (cb.cs_n === 1'b0));
  endtask
endinterface

module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus (clk);
  logic plain = 0;
  default clocking dcb @(posedge clk);
    input cs_n = bus.cs_n;
    input data = bus.data;
  endclocking
  initial begin
    bus.cs_n = 1'b1;
    bus.data = 16'h0000;
    bus.o = 0;
    #20;
    bus.cs_n = 1'b0;
    bus.data = 16'hf000;
    #4 bus.data = 16'hf00f;
    #6 bus.data = 16'h0001;
    plain = 1;
    #10 bus.data = 16'h0002;
    #10 bus.cs_n = 1'b1;
  end
  // 1 negedge clocking block iff
  initial begin
    @(bus.ncb iff (bus.ncb.cs_n === 1'b0));
    $display("T| neg_cb t=%0t d=%h", $time, bus.ncb.data);
  end
  // 2 #1 input skew
  initial begin
    @(bus.cb1 iff (bus.cb1.data === 16'hf00f));
    $display("T| skew1 t=%0t", $time);
  end
  // 3 interface task with clocking iff
  initial begin
    bus.wait_low();
    $display("T| iface_task t=%0t", $time);
  end
  // 4 iff mixing clockvar and plain signal
  initial begin
    @(bus.cb iff (bus.cb.cs_n === 1'b0 && plain));
    $display("T| mixed t=%0t", $time);
  end
  // 5 default clocking ##1 after wait
  initial begin
    @(dcb iff (dcb.cs_n === 1'b0));
    $display("T| dcb t=%0t d=%h", $time, dcb.data);
    ##1;
    $display("T| dcb_cycle t=%0t d=%h", $time, dcb.data);
    ##2;
    $display("T| dcb_cycle2 t=%0t d=%h", $time, dcb.data);
  end
  // 8 forever loop counting guarded clocking events
  int cnt = 0;
  initial begin
    forever begin
      @(bus.cb iff (bus.cb.cs_n === 1'b0));
      cnt++;
      if (cnt == 3) $display("T| loop3 t=%0t", $time);
    end
  end
  // 9 output drive with cycle delay after wait
  initial begin
    @(bus.cb iff (bus.cb.cs_n === 1'b0));
    bus.cb.o <= ##1 1'b1;
    @(bus.o);
    $display("T| out_cycle t=%0t", $time);
  end
  // 10 negated guard that is true at first edge
  initial begin
    @(bus.cb iff (bus.cb.cs_n !== 1'b0));
    $display("T| first_true t=%0t", $time);
  end
  // 11 or-list with clocking event
  initial begin
    @(bus.cb iff (bus.cb.cs_n === 1'b0) or negedge plain);
    $display("T| or_list t=%0t", $time);
  end
  initial #120 $finish;
endmodule
"#;

const EXPECT_SIBLINGS: [&str; 10] = [
    "T| dcb t=25 d=f00f",
    "T| dcb_cycle t=35 d=0001",
    "T| dcb_cycle2 t=55 d=0002",
    "T| first_true t=5",
    "T| iface_task t=25",
    "T| loop3 t=45",
    "T| mixed t=35",
    "T| neg_cb t=30 d=f00f",
    "T| or_list t=25",
    "T| out_cycle t=35",
];

const SRC_FIRST_SAMPLE: &str = r#"
`timescale 1ns/1ns
interface bus_if (input logic clk);
  logic [15:0] data;
  logic cs_n;
  clocking cb @(posedge clk);
    input data;
    input cs_n;
  endclocking
endinterface
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus (clk);
  initial begin
    bus.data = 0; bus.cs_n = 1;
    #30 bus.data = 16'h0001;
  end
  initial begin
    fork
      @(bus.cb iff (bus.cb.data == 16'h0001));
      #100;
    join_any
    $display("T| fork_any t=%0t", $time);
  end
  initial begin
    fork
      begin @(bus.cb iff (bus.cb.data == 16'h0001)); end
    join
    $display("T| fork_join t=%0t", $time);
  end
  initial begin
    @(bus.cb iff (bus.cb.data == 16'h0001));
    $display("T| plain_wait t=%0t", $time);
  end
  initial begin
    @(bus.cb.cs_n);
    $display("T| cbvar_first t=%0t cs=%b", $time, bus.cb.cs_n);
  end
  initial begin
    @(bus.cb.data);
    $display("T| cbvar_data t=%0t d=%h", $time, bus.cb.data);
  end
  initial #100 $finish;
endmodule
"#;

/// The `join_any` block of `SRC_FIRST_SAMPLE` (see
/// `join_any_bare_clocking_iff_item`).
const FORK_ANY_BLOCK: &str = r#"  initial begin
    fork
      @(bus.cb iff (bus.cb.data == 16'h0001));
      #100;
    join_any
    $display("T| fork_any t=%0t", $time);
  end
"#;

const EXPECT_FIRST_SAMPLE: [&str; 5] = [
    "T| cbvar_data t=5 d=0000",
    "T| cbvar_first t=5 cs=1",
    "T| fork_any t=5",
    "T| fork_join t=35",
    "T| plain_wait t=35",
];

const SRC_PROGRAM: &str = r#"
`timescale 1ns/1ns
interface bus_if (input logic clk);
  logic cs_n;
  clocking cb @(posedge clk);
    default input #1step;
    input cs_n;
  endclocking
endinterface
program automatic prg(bus_if b);
  initial begin
    @(posedge b.clk);
    @(b.cb);
    $display("T| prog_plain t=%0t", $time);
    @(posedge b.clk);
    @(b.cb iff (b.cb.cs_n === 1'b1));
    $display("T| prog_iff t=%0t", $time);
    #50;
  end
endprogram
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus (clk);
  prg p (bus);
  initial bus.cs_n = 1;
  initial begin
    @(posedge clk);
    @(bus.cb);
    $display("T| mod_plain t=%0t", $time);
  end
endmodule
"#;

const EXPECT_PROGRAM: [&str; 3] = ["T| mod_plain t=5", "T| prog_iff t=35", "T| prog_plain t=15"];

const SRC_PROGRAM2: &str = r#"`timescale 1ns/1ns
interface bus_if (input logic clk);
  logic cs_n;
  clocking cb @(posedge clk);
    default input #1step;
    input cs_n;
  endclocking
endinterface
program automatic prg(bus_if b, input logic [7:0] q, output logic [7:0] d, output logic pr);
  initial begin
    @(posedge b.clk);
    $display("T| p1 t=%0t q=%0d", $time, q);
    @(b.cb);
    $display("T| p2 t=%0t q=%0d", $time, q);
    d = 8'd7;
    #1 $display("T| p3 t=%0t q=%0d", $time, q);
    @(posedge b.clk);
    $display("T| p4 t=%0t q=%0d", $time, q);
    pr = 1;
    #0 $display("T| p5 t=%0t q=%0d", $time, q);
    @(b.cb iff (b.cb.cs_n === 1'b1));
    $display("T| p6 t=%0t", $time);
    wait (q == 8'd7);
    $display("T| p7 t=%0t", $time);
    #50;
  end
endprogram
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus (clk);
  logic [7:0] q = 0, cnt = 0; logic [7:0] d;
  logic pr;
  always @(posedge clk) begin cnt <= cnt + 1; q <= cnt + 1; end
  always @(posedge pr) $display("T| m_pr t=%0t", $time);
  prg p (bus, q, d, pr);
  initial bus.cs_n = 1;
  initial begin
    @(posedge clk);
    $display("T| m1 t=%0t q=%0d", $time, q);
    @(bus.cb);
    $display("T| m2 t=%0t q=%0d", $time, q);
    #1 $display("T| m3 t=%0t d=%0d", $time, d);
  end
endmodule
"#;

const EXPECT_PROGRAM2: [&str; 11] = [
    "T| m1 t=5 q=0",
    "T| m2 t=5 q=1",
    "T| m3 t=6 d=x",
    "T| m_pr t=25",
    "T| p1 t=5 q=1",
    "T| p2 t=15 q=2",
    "T| p3 t=16 q=2",
    "T| p4 t=25 q=3",
    "T| p5 t=25 q=3",
    "T| p6 t=35",
    "T| p7 t=65",
];

const SRC_IFACE_CBVAR: &str = r#"`timescale 1ns/1ns
interface bus_if (input logic clk);
  logic [15:0] data;
  logic cs_n;
  clocking cb @(posedge clk);
    input data;
    input cs_n;
  endclocking
endinterface
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus (clk);
  initial begin
    bus.data = 0; bus.cs_n = 1;
    #30 bus.data = 16'h0001;
  end
  initial begin
    #7 @(bus.cb.data);
    $display("T| late t=%0t d=%h", $time, bus.cb.data);
  end
  initial begin
    @(bus.cb.data);
    $display("T| early t=%0t d=%h", $time, bus.cb.data);
  end
  always @(bus.cb.cs_n) $display("T| alw t=%0t cs=%b", $time, bus.cb.cs_n);
  initial #50 $finish;
endmodule
"#;

const EXPECT_IFACE_CBVAR: [&str; 3] = [
    "T| alw t=5 cs=1",
    "T| early t=5 d=0000",
    "T| late t=35 d=0001",
];
