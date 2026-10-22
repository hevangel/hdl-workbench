//! §25.9/§25.10: neighbouring shapes of the virtual-interface receiver
//! resolution — a vif array element reached through a chain (constant and
//! variable index), a modport vif, a vif declared through a TYPEDEF of a
//! virtual interface, a vif inherited from a PARAMETERIZED base, a chain
//! through an element of an object array (`e.ag[0].cfg.vif`), a static vif
//! property through its class scope, bit and indexed part-selects, a vif
//! copied into a local, `== null` tests through chains, an interface function
//! with arguments, an NBA to a bit, an edge event, `super.vif`, clocking-block
//! drives through chains; the same resolution in a SUB-module's scope; and
//! vifs held in a QUEUE (class property and module variable) and a STATIC
//! property. Every expectation is the reference simulator's.

use xezim::simulate;

fn out(src: &str, t: u64) -> String {
    let sim = simulate(src, t).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn check(o: &str, expect: &[&str]) {
    for e in expect {
        assert!(
            o.lines().any(|l| l.trim_end() == *e),
            "expected `{e}` in:\n{o}"
        );
    }
}

#[test]
fn vif_receiver_neighbour_shapes_in_methods_and_at_module_scope() {
    let o = out(
        r#"
interface bus_if (input logic clk);
  logic [7:0] data;
  logic flag;
  int cnt;
  modport mp (input clk, output data, output flag, output cnt, import bump);
  clocking cb @(posedge clk); input data; endclocking
  task automatic bump(input int n); cnt += n; endtask
  function automatic int add(int a, int b); return a + b + data; endfunction
endinterface
typedef virtual bus_if vif_t;
class cfg_base #(int W = 1); virtual bus_if pvif; endclass
class cfg_c extends cfg_base #(4);
  virtual bus_if vif;
  virtual bus_if vifs[2];
  virtual bus_if.mp mvif;
  vif_t tvif;
  static virtual bus_if svif;
endclass
class agt; cfg_c cfg; function new(); cfg = new(); endfunction endclass
class env; agt ag[2]; agt a; function new(); a = new(); ag[0] = a; ag[1] = new(); endfunction endclass
class drv;
  env e; cfg_c cfg;
  function new(); e = new(); cfg = e.a.cfg; endfunction
  function int rd_arr_var(int i); return cfg.vifs[i].data; endfunction
  function int rd_arr_c(); return cfg.vifs[1].data; endfunction
  function int rd_mp(); return cfg.mvif.data; endfunction
  function int rd_typedef(); return cfg.tvif.data; endfunction
  function int rd_pinh(); return cfg.pvif.data; endfunction
  function int rd_deep(); return e.a.cfg.vif.data; endfunction
  function int rd_idx_owner(); return e.ag[0].cfg.vif.data; endfunction
  function int rd_stat(); return cfg_c::svif.data; endfunction
  function int rd_bit(int i); return cfg.vif.data[i]; endfunction
  function int rd_ips(int i); return cfg.vif.data[i +: 4]; endfunction
  function int rd_copy(); virtual bus_if v; v = cfg.pvif; return v.data; endfunction
  function int nullchk(); return (cfg.vif == null) + 2*(cfg.pvif != null) + 4*(cfg.tvif == null); endfunction
  function int fadd(); return cfg.vif.add(2, 3); endfunction
  task nb_bit(); cfg.vif.data[0] <= 1'b0; endtask
  task edge_wait(); @(posedge cfg.pvif.clk); $display("edge t=%0t", $time); endtask
endclass
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus(clk);
  bus_if bus2(clk);
  drv d;
  initial begin
    bus.data = 8'ha5; bus.cnt = 0; bus2.data = 8'h5a;
    d = new();
    d.cfg.vif = bus; d.cfg.vifs[0] = bus2; d.cfg.vifs[1] = bus; d.cfg.mvif = bus;
    d.cfg.tvif = bus; d.cfg.pvif = bus; cfg_c::svif = bus2;
    $display("arr_var0=%h arr_var1=%h arr_c=%h mp=%h", d.rd_arr_var(0), d.rd_arr_var(1), d.rd_arr_c(), d.rd_mp());
    $display("typedef=%h pinh=%h deep=%h idx_owner=%h stat=%h", d.rd_typedef(), d.rd_pinh(), d.rd_deep(), d.rd_idx_owner(), d.rd_stat());
    $display("bit7=%0d bit6=%0d ips=%h copy=%h null=%0d fadd=%0d", d.rd_bit(7), d.rd_bit(6), d.rd_ips(4), d.rd_copy(), d.nullchk(), d.fadd());
    $display("mod arr=%h mp=%h typedef=%h pinh=%h deep=%h idx=%h stat=%h", d.cfg.vifs[0].data, d.cfg.mvif.data, d.cfg.tvif.data, d.cfg.pvif.data, d.e.a.cfg.vif.data, d.e.ag[0].cfg.vif.data, cfg_c::svif.data);
    d.nb_bit(); #1 $display("nb_bit data=%h", bus.data);
    fork d.edge_wait(); join_none
    #20 $finish;
  end
endmodule
"#,
        100,
    );
    check(
        &o,
        &[
            "arr_var0=0000005a arr_var1=000000a5 arr_c=000000a5 mp=000000a5",
            "typedef=000000a5 pinh=000000a5 deep=000000a5 idx_owner=000000a5 stat=0000005a",
            "bit7=1 bit6=0 ips=0000000a copy=000000a5 null=2 fadd=170",
            "mod arr=5a mp=a5 typedef=a5 pinh=a5 deep=a5 idx=a5 stat=5a",
            "nb_bit data=a4",
            "edge t=5",
        ],
    );
}

#[test]
fn vif_receiver_chain_in_a_submodule_scope() {
    let o = out(
        r#"
interface bus_if; logic [7:0] data; int cnt; task automatic bump(input int n); cnt += n; endtask endinterface
class cfg_c; virtual bus_if vif; endclass
class holder; cfg_c cfg; function new(); cfg = new(); endfunction endclass
module sub (output int o1, output int o2);
  bus_if sbus();
  holder h;
  initial begin
    sbus.data = 8'h3c; sbus.cnt = 0;
    h = new();
    h.cfg.vif = sbus;
    #1;
    o1 = h.cfg.vif.data;
    h.cfg.vif.data = 8'h44;
    h.cfg.vif.bump(3);
    o2 = sbus.data + sbus.cnt;
    $display("sub o1=%h o2=%h", o1, o2);
  end
endmodule
module tb;
  int a, b;
  sub u(.o1(a), .o2(b));
  initial #5 $display("top a=%h b=%h", a, b);
endmodule
"#,
        100,
    );
    check(
        &o,
        &["sub o1=0000003c o2=00000047", "top a=0000003c b=00000047"],
    );
}

#[test]
fn vifs_held_in_a_queue_and_a_static_property() {
    let o = out(
        r#"
interface bus_if (input logic clk);
  logic [7:0] data;
  logic flag;
  int cnt;
  task automatic bump(input int n); cnt += n; endtask
endinterface
class dq;
  virtual bus_if vq[$];
  function int rd(); return vq[1].data; endfunction
  function void wr(byte v); vq[1].data = v; endfunction
  task wt(); wait (vq[0].flag == 1); $display("q wt t=%0t", $time); endtask
  task cl(); vq[1].bump(1); endtask
endclass
class ds;
  static virtual bus_if svif;
  function int rd(); return svif.data; endfunction
  function void wr(byte v); svif.data = v; endfunction
  task wt(); wait (svif.flag == 1); $display("s wt t=%0t", $time); endtask
  task cl(); svif.bump(2); endtask
endclass
module tb;
  logic clk = 0;
  bus_if bus(clk);
  bus_if bus2(clk);
  dq q; ds s;
  virtual bus_if lq[$];
  initial begin
    bus.data = 8'ha5; bus.flag = 0; bus.cnt = 0; bus2.data = 8'h11;
    q = new(); s = new();
    q.vq.push_back(bus2);
    q.vq.push_back(bus);
    ds::svif = bus;
    lq.push_back(bus2);
    $display("q rd=%h s rd=%h lq=%h", q.rd(), s.rd(), lq[0].data);
    q.wr(8'h3c); $display("q wr=%h", bus.data);
    s.wr(8'h4d); $display("s wr=%h", bus.data);
    q.cl(); s.cl(); $display("call cnt=%0d", bus.cnt);
    void'(q.vq.pop_front());
    $display("q after pop=%h", q.vq[0].data);
    fork q.wt(); s.wt(); join_none
    #10 bus.flag = 1;
    #5 $finish;
  end
endmodule
"#,
        100,
    );
    check(
        &o,
        &[
            "q rd=000000a5 s rd=000000a5 lq=11",
            "q wr=3c",
            "s wr=4d",
            "call cnt=3",
            "q after pop=4d",
            "s wt t=10",
            "q wt t=10",
        ],
    );
}

#[test]
fn super_vif_null_tests_and_clocking_drives_through_chains() {
    let o = out(
        r#"
interface bus_if (input logic clk);
  logic [7:0] data;
  logic [7:0] q;
  clocking cb @(posedge clk); output q; input data; endclocking
endinterface
class base_d; virtual bus_if vif; endclass
class der_d extends base_d;
  function int rd_super(); return super.vif.data; endfunction
  task drv_cb(byte v); @(vif.cb); vif.cb.q <= v; endtask
endclass
class cfg_c; virtual bus_if vif; endclass
class env; cfg_c cfg; function new(); cfg = new(); endfunction
  task drv_chain(byte v); @(cfg.vif.cb); cfg.vif.cb.q <= v; endtask
  function bit isnull(); return cfg.vif == null; endfunction
endclass
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if bus(clk);
  der_d d; env e;
  initial begin
    bus.data = 8'h5a; bus.q = 0;
    d = new(); e = new();
    $display("null_before=%0d", e.isnull());
    d.vif = bus; e.cfg.vif = bus;
    $display("null_after=%0d super=%h", e.isnull(), d.rd_super());
    d.drv_cb(8'h11);
    #1 $display("t=%0t q=%h", $time, bus.q);
    e.drv_chain(8'h22);
    #1 $display("t=%0t q=%h", $time, bus.q);
    #20 $finish;
  end
endmodule
"#,
        100,
    );
    check(
        &o,
        &[
            "null_before=1",
            "null_after=0 super=0000005a",
            "t=6 q=11",
            "t=16 q=22",
        ],
    );
}
