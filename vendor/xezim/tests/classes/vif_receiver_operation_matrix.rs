//! §25.8/§25.9: a receiver whose static type is a VIRTUAL INTERFACE names
//! the interface instance it is bound to, whatever the receiver's spelling,
//! and every operation through it reaches that instance.
//!
//! One resolution step serves every consumer: the receiver — the class's own
//! vif property, one reached through a class-handle chain (`cfg.vif`,
//! `a.cfg.vif`, `this.cfg.vif`), through a local or a class-typed formal
//! handle, a vif declared in a BASE class, a vif of a parameterized class, or
//! the same chains at module scope (`d.cfg.vif`) — resolves through the
//! recorded binding to the instance, and the read, part-select, blocking and
//! nonblocking write, `@(...)`, `wait(...)`, clocking-block event and sampled
//! read, interface task and function call, and the handle assignment itself
//! all see that instance. Only the class's own property used to work; the
//! other spellings read 0 (x at module scope), dropped writes, never woke, or
//! called nothing, and a vif inherited from a base class could not even be
//! bound. Every expectation below is the reference simulator's.

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

const SRC: &str = r#"
interface bus_if (input logic clk);
  logic [7:0] data;
  logic flag;
  int cnt;
  clocking cb @(posedge clk);
    input data;
  endclocking
  task automatic bump(input int n); cnt += n; endtask
  function automatic int get2(); return data * 2; endfunction
endinterface
class cfg_base;
  virtual bus_if bvif;
  function int bown(); return bvif.data; endfunction
endclass
class cfg_c extends cfg_base;
  virtual bus_if vif;
  function int own(); return vif.data; endfunction
endclass
class cfg_p #(int W = 8);
  virtual bus_if vif;
  function int own(); return vif.data; endfunction
endclass
class agt;
  cfg_c cfg; cfg_p #(4) cp;
  function new(); cfg = new(); cp = new(); endfunction
endclass
class drv;
  virtual bus_if vif; agt a; cfg_c cfg;
  function new(); a = new(); cfg = a.cfg; endfunction
  function int rd_own();  return vif.data; endfunction
  function int ps_own();  return vif.data[7:4]; endfunction
  function int fc_own();  return vif.get2(); endfunction
  function int rd_two();  return cfg.vif.data; endfunction
  function int ps_two();  return cfg.vif.data[7:4]; endfunction
  function int fc_two();  return cfg.vif.get2(); endfunction
  function int rd_three();  return a.cfg.vif.data; endfunction
  function int ps_three();  return a.cfg.vif.data[7:4]; endfunction
  function int fc_three();  return a.cfg.vif.get2(); endfunction
  function int rd_this();  return this.cfg.vif.data; endfunction
  function int ps_this();  return this.cfg.vif.data[7:4]; endfunction
  function int fc_this();  return this.cfg.vif.get2(); endfunction
  function int rd_local(); cfg_c c; c = a.cfg; return c.vif.data; endfunction
  function int ps_local(); cfg_c c; c = a.cfg; return c.vif.data[7:4]; endfunction
  function int fc_local(); cfg_c c; c = a.cfg; return c.vif.get2(); endfunction
  function int rd_inh();  return cfg.bvif.data; endfunction
  function int ps_inh();  return cfg.bvif.data[7:4]; endfunction
  function int fc_inh();  return cfg.bvif.get2(); endfunction
  function int rd_par();  return a.cp.vif.data; endfunction
  function int ps_par();  return a.cp.vif.data[7:4]; endfunction
  function int fc_par();  return a.cp.vif.get2(); endfunction
  function int rd_formal(cfg_c c);  return c.vif.data; endfunction
  function int ps_formal(cfg_c c);  return c.vif.data[7:4]; endfunction
  function int fc_formal(cfg_c c);  return c.vif.get2(); endfunction
  function void wr_own(byte v);  vif.data = v; endfunction
  task nb_own(byte v);  vif.data <= ~v; endtask
  task cl_own();  vif.bump(1); endtask
  function void wr_two(byte v);  cfg.vif.data = v; endfunction
  task nb_two(byte v);  cfg.vif.data <= ~v; endtask
  task cl_two();  cfg.vif.bump(2); endtask
  function void wr_three(byte v);  a.cfg.vif.data = v; endfunction
  task nb_three(byte v);  a.cfg.vif.data <= ~v; endtask
  task cl_three();  a.cfg.vif.bump(3); endtask
  function void wr_this(byte v);  this.cfg.vif.data = v; endfunction
  task nb_this(byte v);  this.cfg.vif.data <= ~v; endtask
  task cl_this();  this.cfg.vif.bump(4); endtask
  function void wr_local(byte v); cfg_c c; c = a.cfg; c.vif.data = v; endfunction
  task nb_local(byte v); cfg_c c; c = a.cfg; c.vif.data <= ~v; endtask
  task cl_local(); cfg_c c; c = a.cfg; c.vif.bump(5); endtask
  function void wr_inh(byte v);  cfg.bvif.data = v; endfunction
  task nb_inh(byte v);  cfg.bvif.data <= ~v; endtask
  task cl_inh();  cfg.bvif.bump(6); endtask
  function void wr_par(byte v);  a.cp.vif.data = v; endfunction
  task nb_par(byte v);  a.cp.vif.data <= ~v; endtask
  task cl_par();  a.cp.vif.bump(7); endtask
  function void wr_formal(cfg_c c, byte v);  c.vif.data = v; endfunction
  task nb_formal(cfg_c c, byte v);  c.vif.data <= ~v; endtask
  task cl_formal(cfg_c c);  c.vif.bump(8); endtask
  function void bd_own(virtual bus_if b);  vif = b; endfunction
  function void bd_two(virtual bus_if b);  cfg.vif = b; endfunction
  function void bd_three(virtual bus_if b);  a.cfg.vif = b; endfunction
  function void bd_this(virtual bus_if b);  this.cfg.vif = b; endfunction
  function void bd_local(virtual bus_if b); cfg_c c; c = a.cfg; c.vif = b; endfunction
  function void bd_inh(virtual bus_if b);  cfg.bvif = b; endfunction
  function void bd_par(virtual bus_if b);  a.cp.vif = b; endfunction
  function void bd_formal(cfg_c c, virtual bus_if b);  c.vif = b; endfunction
  task ev_own();  @(vif.data); $display("ev own t=%0t", $time); endtask
  task wt_own();  wait (vif.flag == 1); $display("wt own t=%0t", $time); endtask
  task cb_own();  @(vif.cb); $display("cb own t=%0t d=%h", $time, vif.cb.data); endtask
  task ev_two();  @(cfg.vif.data); $display("ev two t=%0t", $time); endtask
  task wt_two();  wait (cfg.vif.flag == 1); $display("wt two t=%0t", $time); endtask
  task cb_two();  @(cfg.vif.cb); $display("cb two t=%0t d=%h", $time, cfg.vif.cb.data); endtask
  task ev_three();  @(a.cfg.vif.data); $display("ev three t=%0t", $time); endtask
  task wt_three();  wait (a.cfg.vif.flag == 1); $display("wt three t=%0t", $time); endtask
  task cb_three();  @(a.cfg.vif.cb); $display("cb three t=%0t d=%h", $time, a.cfg.vif.cb.data); endtask
  task ev_this();  @(this.cfg.vif.data); $display("ev this t=%0t", $time); endtask
  task wt_this();  wait (this.cfg.vif.flag == 1); $display("wt this t=%0t", $time); endtask
  task cb_this();  @(this.cfg.vif.cb); $display("cb this t=%0t d=%h", $time, this.cfg.vif.cb.data); endtask
  task ev_local(); cfg_c c; c = a.cfg; @(c.vif.data); $display("ev local t=%0t", $time); endtask
  task wt_local(); cfg_c c; c = a.cfg; wait (c.vif.flag == 1); $display("wt local t=%0t", $time); endtask
  task cb_local(); cfg_c c; c = a.cfg; @(c.vif.cb); $display("cb local t=%0t d=%h", $time, c.vif.cb.data); endtask
  task ev_inh();  @(cfg.bvif.data); $display("ev inh t=%0t", $time); endtask
  task wt_inh();  wait (cfg.bvif.flag == 1); $display("wt inh t=%0t", $time); endtask
  task cb_inh();  @(cfg.bvif.cb); $display("cb inh t=%0t d=%h", $time, cfg.bvif.cb.data); endtask
  task ev_par();  @(a.cp.vif.data); $display("ev par t=%0t", $time); endtask
  task wt_par();  wait (a.cp.vif.flag == 1); $display("wt par t=%0t", $time); endtask
  task cb_par();  @(a.cp.vif.cb); $display("cb par t=%0t d=%h", $time, a.cp.vif.cb.data); endtask
  task ev_formal(cfg_c c);  @(c.vif.data); $display("ev formal t=%0t", $time); endtask
  task wt_formal(cfg_c c);  wait (c.vif.flag == 1); $display("wt formal t=%0t", $time); endtask
  task cb_formal(cfg_c c);  @(c.vif.cb); $display("cb formal t=%0t d=%h", $time, c.vif.cb.data); endtask
endclass
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus(clk);
  bus_if bus2(clk);
  drv d;
  byte w;
  initial begin
    bus.flag = 0; bus.cnt = 0;
    d = new();
    d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    bus.data = 8'ha5;
    $display("rd own=%h ps=%h fc=%h", d.rd_own(), d.ps_own(), d.fc_own());
    $display("rd two=%h ps=%h fc=%h", d.rd_two(), d.ps_two(), d.fc_two());
    $display("rd three=%h ps=%h fc=%h", d.rd_three(), d.ps_three(), d.fc_three());
    $display("rd this=%h ps=%h fc=%h", d.rd_this(), d.ps_this(), d.fc_this());
    $display("rd local=%h ps=%h fc=%h", d.rd_local(), d.ps_local(), d.fc_local());
    $display("rd inh=%h ps=%h fc=%h", d.rd_inh(), d.ps_inh(), d.fc_inh());
    $display("rd par=%h ps=%h fc=%h", d.rd_par(), d.ps_par(), d.fc_par());
    $display("rd formal=%h ps=%h fc=%h", d.rd_formal(d.cfg), d.ps_formal(d.cfg), d.fc_formal(d.cfg));
    $display("rd m_own=%h ps=%h fc=%h", d.vif.data, d.vif.data[7:4], d.vif.get2());
    $display("rd m_two=%h ps=%h fc=%h", d.cfg.vif.data, d.cfg.vif.data[7:4], d.cfg.vif.get2());
    $display("rd m_three=%h ps=%h fc=%h", d.a.cfg.vif.data, d.a.cfg.vif.data[7:4], d.a.cfg.vif.get2());
    $display("rd m_inh=%h ps=%h fc=%h", d.cfg.bvif.data, d.cfg.bvif.data[7:4], d.cfg.bvif.get2());
    $display("rd m_par=%h ps=%h fc=%h", d.a.cp.vif.data, d.a.cp.vif.data[7:4], d.a.cp.vif.get2());
    d.wr_own(8'h11); w = bus.data; d.nb_own(8'h11); #1;
    d.cl_own();
    $display("wr own=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.wr_two(8'h22); w = bus.data; d.nb_two(8'h22); #1;
    d.cl_two();
    $display("wr two=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.wr_three(8'h33); w = bus.data; d.nb_three(8'h33); #1;
    d.cl_three();
    $display("wr three=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.wr_this(8'h44); w = bus.data; d.nb_this(8'h44); #1;
    d.cl_this();
    $display("wr this=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.wr_local(8'h55); w = bus.data; d.nb_local(8'h55); #1;
    d.cl_local();
    $display("wr local=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.wr_inh(8'h66); w = bus.data; d.nb_inh(8'h66); #1;
    d.cl_inh();
    $display("wr inh=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.wr_par(8'h77); w = bus.data; d.nb_par(8'h77); #1;
    d.cl_par();
    $display("wr par=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.wr_formal(d.cfg, 8'h88); w = bus.data; d.nb_formal(d.cfg, 8'h88); #1;
    d.cl_formal(d.cfg);
    $display("wr formal=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.vif.data = 8'hc0; w = bus.data; d.vif.data <= 8'hd0; #1; d.vif.bump(10);
    $display("wr m_own=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.cfg.vif.data = 8'hc1; w = bus.data; d.cfg.vif.data <= 8'hd1; #1; d.cfg.vif.bump(10);
    $display("wr m_two=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.a.cfg.vif.data = 8'hc2; w = bus.data; d.a.cfg.vif.data <= 8'hd2; #1; d.a.cfg.vif.bump(10);
    $display("wr m_three=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.cfg.bvif.data = 8'hc3; w = bus.data; d.cfg.bvif.data <= 8'hd3; #1; d.cfg.bvif.bump(10);
    $display("wr m_inh=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    d.a.cp.vif.data = 8'hc4; w = bus.data; d.a.cp.vif.data <= 8'hd4; #1; d.a.cp.vif.bump(10);
    $display("wr m_par=%h nb=%h cnt=%0d", w, bus.data, bus.cnt);
    bus2.data = 8'h77;
    d.bd_own(bus2);
    $display("bind own=%h", d.rd_own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.bd_two(bus2);
    $display("bind two=%h", d.cfg.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.bd_three(bus2);
    $display("bind three=%h", d.cfg.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.bd_this(bus2);
    $display("bind this=%h", d.cfg.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.bd_local(bus2);
    $display("bind local=%h", d.cfg.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.bd_inh(bus2);
    $display("bind inh=%h", d.cfg.bown()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.bd_par(bus2);
    $display("bind par=%h", d.a.cp.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.bd_formal(d.cfg, bus2);
    $display("bind formal=%h", d.cfg.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.vif = bus2;
    $display("bind m_own=%h", d.rd_own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.cfg.vif = bus2;
    $display("bind m_two=%h", d.cfg.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.a.cfg.vif = bus2;
    $display("bind m_three=%h", d.cfg.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.cfg.bvif = bus2;
    $display("bind m_inh=%h", d.cfg.bown()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    d.a.cp.vif = bus2;
    $display("bind m_par=%h", d.a.cp.own()); d.vif = bus; d.a.cfg.vif = bus; d.a.cfg.bvif = bus; d.a.cp.vif = bus;
    #(100 - $time);
    bus.flag = 0;
    fork
      d.ev_own(); d.wt_own(); d.cb_own();
      d.ev_two(); d.wt_two(); d.cb_two();
      d.ev_three(); d.wt_three(); d.cb_three();
      d.ev_this(); d.wt_this(); d.cb_this();
      d.ev_local(); d.wt_local(); d.cb_local();
      d.ev_inh(); d.wt_inh(); d.cb_inh();
      d.ev_par(); d.wt_par(); d.cb_par();
      d.ev_formal(d.cfg); d.wt_formal(d.cfg); d.cb_formal(d.cfg);
      begin @(d.vif.data); $display("ev m_own t=%0t", $time); end
      begin wait (d.vif.flag == 1); $display("wt m_own t=%0t", $time); end
      begin @(d.vif.cb); $display("cb m_own t=%0t d=%h", $time, d.vif.cb.data); end
      begin @(d.cfg.vif.data); $display("ev m_two t=%0t", $time); end
      begin wait (d.cfg.vif.flag == 1); $display("wt m_two t=%0t", $time); end
      begin @(d.cfg.vif.cb); $display("cb m_two t=%0t d=%h", $time, d.cfg.vif.cb.data); end
      begin @(d.a.cfg.vif.data); $display("ev m_three t=%0t", $time); end
      begin wait (d.a.cfg.vif.flag == 1); $display("wt m_three t=%0t", $time); end
      begin @(d.a.cfg.vif.cb); $display("cb m_three t=%0t d=%h", $time, d.a.cfg.vif.cb.data); end
      begin @(d.cfg.bvif.data); $display("ev m_inh t=%0t", $time); end
      begin wait (d.cfg.bvif.flag == 1); $display("wt m_inh t=%0t", $time); end
      begin @(d.cfg.bvif.cb); $display("cb m_inh t=%0t d=%h", $time, d.cfg.bvif.cb.data); end
      begin @(d.a.cp.vif.data); $display("ev m_par t=%0t", $time); end
      begin wait (d.a.cp.vif.flag == 1); $display("wt m_par t=%0t", $time); end
      begin @(d.a.cp.vif.cb); $display("cb m_par t=%0t d=%h", $time, d.a.cp.vif.cb.data); end
    join_none
    #2 bus.data = 8'h3c;
    #10 bus.flag = 1;
    #20 $finish;
  end
endmodule
"#;

/// The reference simulator's output (the forked ev/cb/wt lines print in
/// scheduling order, so each line is matched on its own).
const EXPECT: &[&str] = &[
    "rd own=000000a5 ps=0000000a fc=0000014a",
    "rd two=000000a5 ps=0000000a fc=0000014a",
    "rd three=000000a5 ps=0000000a fc=0000014a",
    "rd this=000000a5 ps=0000000a fc=0000014a",
    "rd local=000000a5 ps=0000000a fc=0000014a",
    "rd inh=000000a5 ps=0000000a fc=0000014a",
    "rd par=000000a5 ps=0000000a fc=0000014a",
    "rd formal=000000a5 ps=0000000a fc=0000014a",
    "rd m_own=a5 ps=a fc=0000014a",
    "rd m_two=a5 ps=a fc=0000014a",
    "rd m_three=a5 ps=a fc=0000014a",
    "rd m_inh=a5 ps=a fc=0000014a",
    "rd m_par=a5 ps=a fc=0000014a",
    "wr own=11 nb=ee cnt=1",
    "wr two=22 nb=dd cnt=3",
    "wr three=33 nb=cc cnt=6",
    "wr this=44 nb=bb cnt=10",
    "wr local=55 nb=aa cnt=15",
    "wr inh=66 nb=99 cnt=21",
    "wr par=77 nb=88 cnt=28",
    "wr formal=88 nb=77 cnt=36",
    "wr m_own=c0 nb=d0 cnt=46",
    "wr m_two=c1 nb=d1 cnt=56",
    "wr m_three=c2 nb=d2 cnt=66",
    "wr m_inh=c3 nb=d3 cnt=76",
    "wr m_par=c4 nb=d4 cnt=86",
    "bind own=00000077",
    "bind two=00000077",
    "bind three=00000077",
    "bind this=00000077",
    "bind local=00000077",
    "bind inh=00000077",
    "bind par=00000077",
    "bind formal=00000077",
    "bind m_own=00000077",
    "bind m_two=00000077",
    "bind m_three=00000077",
    "bind m_inh=00000077",
    "bind m_par=00000077",
    "ev m_par t=102",
    "ev m_inh t=102",
    "ev m_three t=102",
    "ev m_two t=102",
    "ev m_own t=102",
    "ev formal t=102",
    "ev par t=102",
    "ev inh t=102",
    "ev local t=102",
    "ev this t=102",
    "ev three t=102",
    "ev two t=102",
    "ev own t=102",
    "cb m_par t=105 d=3c",
    "cb m_inh t=105 d=3c",
    "cb m_three t=105 d=3c",
    "cb m_two t=105 d=3c",
    "cb m_own t=105 d=3c",
    "cb formal t=105 d=3c",
    "cb par t=105 d=3c",
    "cb inh t=105 d=3c",
    "cb local t=105 d=3c",
    "cb this t=105 d=3c",
    "cb three t=105 d=3c",
    "cb two t=105 d=3c",
    "cb own t=105 d=3c",
    "wt m_par t=112",
    "wt m_inh t=112",
    "wt m_three t=112",
    "wt m_two t=112",
    "wt m_own t=112",
    "wt formal t=112",
    "wt par t=112",
    "wt inh t=112",
    "wt local t=112",
    "wt this t=112",
    "wt three t=112",
    "wt two t=112",
    "wt own t=112",
];

#[test]
fn every_vif_receiver_spelling_reaches_the_bound_instance_for_every_operation() {
    let o = out(SRC);
    let missing: Vec<&str> = EXPECT
        .iter()
        .copied()
        .filter(|e| !o.lines().any(|l| l.trim_end() == *e))
        .collect();
    assert!(
        missing.is_empty(),
        "missing lines:\n{}\n--- output:\n{o}",
        missing.join("\n")
    );
}
