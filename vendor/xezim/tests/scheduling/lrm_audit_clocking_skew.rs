//! IEEE 1800-2023 §14.3 / §14.4 / §14.13 / §14.16: clocking-block input
//! sampling and output skews. Inputs sample with `#1step` (the default) in
//! the Preponed region, with `#0` in the Observed region and with `#N` the
//! value N time units before the clocking event; `1step` used to read as
//! `#1` (post-edge value) and swallowed the rest of a `default` item, so an
//! output skew after it was lost. Expected values come from the reference
//! simulator.
use xezim::simulate;

fn t_lines(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

/// The audit repro (§14.4 / §14.13): `cb.d` after `@(cb)` is the pre-edge
/// sample, and `default output #2` drives 2 units after the edge.
#[test]
fn input_sampling_audit_repro() {
    const SRC: &str = r#"
`timescale 1ns/1ps
module rcb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] d = 0, dout;
  always @(posedge clk) d <= d + 1;
  clocking cb @(posedge clk);
    default input #1step output #2;
    input d;
    input #3 dl = d;
    output dout;
  endclocking
  initial begin
    repeat (4) begin
      @(cb);
      $display("T|r1|t=%0t cb.d=%0d cb.dl=%0d d=%0d", $time, cb.d, cb.dl, d);
    end
    repeat (2) begin
      @(posedge clk);
      $display("T|r2|t=%0t cb.d=%0d d=%0d", $time, cb.d, d);
    end
    cb.dout <= 8'hab;
    #1 $display("T|r3|t=%0t dout=%h", $time, dout);
    #2 $display("T|r4|t=%0t dout=%h", $time, dout);
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|r1|t=5000 cb.d=0 cb.dl=0 d=1",
            "T|r1|t=15000 cb.d=1 cb.dl=1 d=2",
            "T|r1|t=25000 cb.d=2 cb.dl=2 d=3",
            "T|r1|t=35000 cb.d=3 cb.dl=3 d=4",
            "T|r2|t=45000 cb.d=3 d=4",
            "T|r2|t=55000 cb.d=4 d=5",
            "T|r3|t=56000 dout=xx",
            "T|r4|t=58000 dout=ab",
        ],
    );
}

/// Default, `#1step`, `#0`, `#N` (also beyond a period), per-signal overrides,
/// `#3ns` and `#(1+2)`: a value changing exactly N units before the edge is not
/// yet visible to an `input #N` sample.
#[test]
fn input_skew_kinds() {
    const SRC: &str = r#"
`timescale 1ns/1ns
module cbin1;
  logic clk = 0;
  always #5 clk = ~clk;
  int d = 0, e = 0, f = 0;
  always @(posedge clk) d <= d + 1;
  always @(posedge clk) f = f + 1;
  initial forever begin #2 e = e + 1; #8; end
  clocking c_def @(posedge clk); input d, e, f; endclocking
  clocking c_1s @(posedge clk); default input #1step; input d, e, f; endclocking
  clocking c_0 @(posedge clk); default input #0; input d, e, f; endclocking
  clocking c_2 @(posedge clk); default input #2; input d, e, f; endclocking
  clocking c_3 @(posedge clk); default input #3; input d, e, f; endclocking
  clocking c_4 @(posedge clk); default input #4; input d, e, f; endclocking
  clocking c_12 @(posedge clk); default input #12; input d, e, f; endclocking
  clocking c_mix @(posedge clk); default input #0; input #1step d; input #3 e; input f; endclocking
  clocking c_ns @(posedge clk); default input #3ns output #1; input d, e, f; endclocking
  clocking c_par @(posedge clk); default input #(1+2); input d, e, f; endclocking
  initial begin
    repeat (3) begin
      @(c_def);
      $display("T|at t=%0t def=%0d/%0d/%0d 1s=%0d/%0d/%0d z=%0d/%0d/%0d", $time,
        c_def.d, c_def.e, c_def.f, c_1s.d, c_1s.e, c_1s.f, c_0.d, c_0.e, c_0.f);
      #1;
      $display("T|p1 t=%0t def=%0d/%0d/%0d 1s=%0d/%0d/%0d z=%0d/%0d/%0d", $time,
        c_def.d, c_def.e, c_def.f, c_1s.d, c_1s.e, c_1s.f, c_0.d, c_0.e, c_0.f);
      $display("T|p2 t=%0t s2=%0d/%0d/%0d s3=%0d/%0d/%0d s4=%0d/%0d/%0d s12=%0d/%0d/%0d", $time,
        c_2.d, c_2.e, c_2.f, c_3.d, c_3.e, c_3.f, c_4.d, c_4.e, c_4.f, c_12.d, c_12.e, c_12.f);
      $display("T|p3 t=%0t mix=%0d/%0d/%0d ns=%0d/%0d/%0d par=%0d/%0d/%0d", $time,
        c_mix.d, c_mix.e, c_mix.f, c_ns.d, c_ns.e, c_ns.f, c_par.d, c_par.e, c_par.f);
    end
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|at t=5 def=0/1/0 1s=0/1/0 z=1/1/1",
            "T|p1 t=6 def=0/1/0 1s=0/1/0 z=1/1/1",
            "T|p2 t=6 s2=0/1/0 s3=0/0/0 s4=0/0/0 s12=0/0/0",
            "T|p3 t=6 mix=0/0/1 ns=0/0/0 par=0/0/0",
            "T|at t=15 def=1/2/1 1s=1/2/1 z=2/2/2",
            "T|p1 t=16 def=1/2/1 1s=1/2/1 z=2/2/2",
            "T|p2 t=16 s2=1/2/1 s3=1/1/1 s4=1/1/1 s12=0/1/0",
            "T|p3 t=16 mix=1/1/2 ns=1/1/1 par=1/1/1",
            "T|at t=25 def=2/3/2 1s=2/3/2 z=3/3/3",
            "T|p1 t=26 def=2/3/2 1s=2/3/2 z=3/3/3",
            "T|p2 t=26 s2=2/3/2 s3=2/2/2 s4=2/2/2 s12=1/2/1",
            "T|p3 t=26 mix=2/2/3 ns=2/2/2 par=2/2/2",
        ],
    );
}

/// A clockvar read before the first clocking event is x; `#1ps` / `#2ps` input
/// skews against a change 1ps before the edge; `@(cb)` reached right after
/// `@(posedge clk)` in the same slot still sees the clocking event of that slot;
/// an `input #0` of a negedge block.
#[test]
fn input_skew_precision_and_ordering() {
    const SRC: &str = r#"
`timescale 1ns/1ps
module cbin2;
  // input skew vs timescale precision; edge read order; negedge block
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] d = 0;
  always @(posedge clk) d <= d + 1;
  logic [7:0] g = 0;
  initial forever begin #4.999 g = g + 1; #5.001; end // changes 1ps before posedge
  clocking cb @(posedge clk); default input #1step; input d, g; input #1ps gp = g; input #2ps gq = g; endclocking
  clocking cn @(negedge clk); input d; input #0 d0 = d; endclocking
  initial begin
    @(posedge clk);
    $display("T|pe t=%0t cb.d=%0d d=%0d", $time, cb.d, d);
    @(cb);
    $display("T|cb t=%0t cb.d=%0d g=%0d gp=%0d gq=%0d", $time, cb.d, cb.g, cb.gp, cb.gq);
    #0 $display("T|cb0 t=%0t cb.d=%0d", $time, cb.d);
    @(cn);
    $display("T|cn t=%0t cn.d=%0d cn.d0=%0d d=%0d", $time, cn.d, cn.d0, d);
    @(cb);
    $display("T|cb2 t=%0t cb.d=%0d g=%0d gp=%0d gq=%0d", $time, cb.d, cb.g, cb.gp, cb.gq);
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|pe t=5000 cb.d=x d=0",
            "T|cb t=5000 cb.d=0 g=1 gp=0 gq=0",
            "T|cb0 t=5000 cb.d=0",
            "T|cn t=10000 cn.d=1 cn.d0=1 d=1",
            "T|cb2 t=15000 cb.d=1 g=2 gp=1 gq=1",
        ],
    );
}

/// `default input #1` read through a virtual interface handle samples the
/// pre-edge value as the direct and the default-skew block do.
#[test]
fn input_skew_through_vif() {
    const SRC: &str = r#"
`timescale 1ns/1ns
interface bif(input logic clk);
  logic [7:0] gnt, req;
  clocking cb @(posedge clk);
    default input #1 output #1;
    input gnt;
    output req;
  endclocking
  clocking cs @(posedge clk);
    input gnt;
  endclocking
endinterface
class mon;
  virtual bif vif;
  function new(virtual bif v); vif = v; endfunction
  task run();
    repeat (3) begin
      @(vif.cb);
      $display("T|vif t=%0t cb.gnt=%h cs.gnt=%h gnt=%h", $time, vif.cb.gnt, vif.cs.gnt, vif.gnt);
    end
  endtask
endclass
module cbin3;
  logic clk = 0;
  always #5 clk = ~clk;
  bif b(clk);
  always @(posedge clk) b.gnt <= b.req + 1;
  initial begin
    mon m = new(b);
    b.req = 0;
    b.gnt = 0;
    m.run();
    @(b.cb);
    $display("T|dir t=%0t cb.gnt=%h cs.gnt=%h", $time, b.cb.gnt, b.cs.gnt);
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|vif t=5 cb.gnt=00 cs.gnt=00 gnt=01",
            "T|vif t=15 cb.gnt=01 cs.gnt=01 gnt=01",
            "T|vif t=25 cb.gnt=01 cs.gnt=01 gnt=01",
            "T|dir t=35 cb.gnt=01 cs.gnt=01",
        ],
    );
}

/// Output skews `#0`, `#2` (block default), `#1step` (one precision unit), `#3`,
/// and drives that wait for the next edge (`##2`, issued between edges).
#[test]
fn output_skews() {
    const SRC: &str = r#"
`timescale 1ns/1ps
module cbout1r;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] o0, o2, o1s, o3, oc, ob, od;
  clocking cb @(posedge clk);
    default input #1step output #2;
    output o2, oc, ob;
    output #0 o0;
    output #1step o1s;
    output #3 o3;
  endclocking
  clocking cd @(posedge clk);
    output od;
  endclocking
  always @(o0) $display("T|o0 t=%0.3f v=%h", $realtime, o0);
  always @(o2) $display("T|o2 t=%0.3f v=%h", $realtime, o2);
  always @(o1s) $display("T|o1s t=%0.3f v=%h", $realtime, o1s);
  always @(o3) $display("T|o3 t=%0.3f v=%h", $realtime, o3);
  always @(oc) $display("T|oc t=%0.3f v=%h", $realtime, oc);
  always @(ob) $display("T|ob t=%0.3f v=%h", $realtime, ob);
  always @(od) $display("T|od t=%0.3f v=%h", $realtime, od);
  initial begin
    @(cb);
    cb.o0 <= 1; cb.o2 <= 2; cb.o1s <= 3; cb.o3 <= 4; cd.od <= 9;
    cb.oc <= ##2 8'hc1;
    @(cb);
    #1 cb.ob <= 8'hb1;   // between edges: next edge + skew
    @(cb); @(cb); @(cb);
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    let mut got: Vec<String> = t_lines(&sim);
    got.sort();
    let mut exp: Vec<&str> = vec![
        "T|od t=5.000 v=09",
        "T|o0 t=5.000 v=01",
        "T|o1s t=5.001 v=03",
        "T|o2 t=7.000 v=02",
        "T|o3 t=8.000 v=04",
        "T|oc t=27.000 v=c1",
        "T|ob t=27.000 v=b1",
    ];
    exp.sort();
    assert_eq!(got, exp);
}

/// §14.16.2: an `inout` clockvar on an interface wire is driven through the
/// virtual interface and the instance with the output skew (audit repro).
#[test]
fn inout_clockvar_on_interface_wire() {
    const SRC: &str = r#"
`timescale 1ns/1ns
interface cif(input logic clk);
  logic [7:0] req;
  wire [7:0] bidir;
  clocking cb @(posedge clk);
    default input #1 output #1;
    output req;
    inout bidir;
  endclocking
endinterface
module rvd;
  logic clk = 0;
  always #5 clk = ~clk;
  cif ifc(clk);
  virtual cif vif;
  initial begin
    vif = ifc;
    @(vif.cb);
    vif.cb.req <= 8'h11;
    vif.cb.bidir <= 8'h5a;
    #3 $display("T|r1|t=%0t req=%h bidir=%h", $time, ifc.req, ifc.bidir);
    @(ifc.cb);
    ifc.cb.bidir <= 8'h77;
    #3 $display("T|r2|t=%0t bidir=%h", $time, ifc.bidir);
    $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    assert_eq!(
        t_lines(&sim),
        ["T|r1|t=8 req=11 bidir=5a", "T|r2|t=18 bidir=77",],
    );
}

/// The same through a vif, with the sampled value read back; the time-0
/// `always @(ifc.bidir)` event (x to z) is not modeled and is left out.
#[test]
fn inout_clockvar_drive_and_sample() {
    const SRC: &str = r#"
`timescale 1ns/1ns
interface oif(input logic clk);
  logic [7:0] req;
  wire [7:0] bidir;
  logic [7:0] gnt;
  clocking cb @(posedge clk);
    default input #1 output #2;
    output req;
    inout bidir;
    input gnt;
  endclocking
endinterface
module cbout2;
  logic clk = 0;
  always #5 clk = ~clk;
  oif ifc(clk);
  virtual oif vif;
  always @(ifc.req) $display("T|req t=%0t v=%h", $time, ifc.req);
  always @(ifc.bidir) $display("T|bidir t=%0t v=%h", $time, ifc.bidir);
  initial begin
    vif = ifc;
    @(vif.cb);
    vif.cb.req <= 8'h11;
    vif.cb.bidir <= 8'h5a;
    @(vif.cb);
    $display("T|rd t=%0t cb.bidir=%h", $time, vif.cb.bidir);
    ifc.cb.bidir <= 8'h77;
    @(vif.cb);
    $display("T|rd2 t=%0t cb.bidir=%h", $time, vif.cb.bidir);
    #3 $finish;
  end
endmodule
"#;
    let sim = simulate(SRC, 1000000).expect("sim");
    let mut got = t_lines(&sim);
    got.retain(|l| !l.starts_with("T|bidir t=0 "));
    assert_eq!(
        got,
        [
            "T|bidir t=7 v=5a",
            "T|req t=7 v=11",
            "T|rd t=15 cb.bidir=5a",
            "T|bidir t=17 v=77",
            "T|rd2 t=25 cb.bidir=77",
        ],
    );
}
