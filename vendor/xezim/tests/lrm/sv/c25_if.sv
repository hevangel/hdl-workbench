// top: c25
interface bus_if #(parameter W = 8) (input logic clk);
  logic [W-1:0] data;
  logic valid;
  logic ready;
  int cnt = 0;
  modport mst (output data, valid, input ready, import send);
  modport slv (input data, valid, output ready, import recv);
  modport mon (input data, valid, ready);
  // 25.7 tasks in interfaces
  task send(input logic [W-1:0] d);
    data = d; valid = 1;
    @(posedge clk);
    valid = 0;
    cnt++;
  endtask
  task recv(output logic [W-1:0] d);
    @(posedge clk iff valid);
    d = data;
  endtask
  function int width(); return W; endfunction
  // 25.5.4 modport expressions
  modport exp_mp (input .lo(data[3:0]));
  // clocking in interface
  clocking cb @(posedge clk); input data; endclocking
  modport cbm (clocking cb);
endinterface

interface simple_if; logic [3:0] a; logic [3:0] b; endinterface

module producer(bus_if.mst m);
  initial begin
    #1 m.send(8'h5a);
    m.send(8'h6b);
  end
endmodule
module consumer(bus_if.slv s);
  logic [7:0] got;
  initial begin
    repeat (2) begin s.recv(got); $display("T|25.7a|recv %h t=%0t", got, $time); end
  end
endmodule
module expmod(bus_if.exp_mp e);
  initial #30 $display("T|25.5.4|lo=%h", e.lo);
endmodule
// generic interface port
module generic(interface g);
  initial #30 $display("T|25.3.3|generic data=%h", g.data);
endmodule
// interface port with default modport-less
module plain(simple_if sif);
  assign sif.b = sif.a + 1;
endmodule
class drv;
  virtual bus_if #(8) vif;
  virtual bus_if.mon vmon;
  function new(virtual bus_if #(8) v); vif = v; vmon = v; endfunction
  task run(); @(posedge vif.clk); $display("T|25.9a|vif data=%h valid=%b mon=%h", vif.data, vif.valid, vmon.data); endtask
endclass
module c25;
  logic clk = 0;
  always #5 clk = ~clk;
  bus_if #(8) b(clk);
  bus_if #(.W(16)) b16(clk);
  producer p(b.mst);
  consumer c(b.slv);
  expmod e(b);
  generic g(b16);
  simple_if si();
  plain pl(si);
  // array of interfaces
  simple_if sarr[2] ();
  virtual simple_if vsa[2];
  initial begin
    drv d = new(b);
    si.a = 4'h3;
    b16.data = 16'hbeef;
    #2 d.run();
    #25;
    $display("T|25.7b|cnt=%0d width=%0d w16=%0d", b.cnt, b.width(), b16.width());
    $display("T|25.3|si.b=%h", si.b);
    $display("T|25.10|%0d", $bits(b16.data));
    // 25.9 virtual interface assignment and compare
    begin virtual bus_if #(8) v1, v2; v1 = b; v2 = v1; $display("T|25.9b|%0d %0d", v1 == v2, v1 != null);
      v1.data = 8'h77; #0 $display("T|25.9c|%h", b.data);
      v1 = null; $display("T|25.9d|%0d", v1 == null); end
    vsa[0] = sarr[0]; vsa[1] = sarr[1];
    vsa[1].a = 4'h9; #0 $display("T|25.9e|%h", sarr[1].a);
    // clocking via vif
    begin virtual bus_if #(8) vc = b; @(vc.cb); $display("T|25.9f|cb data=%h", vc.cb.data); end
    $finish;
  end
endmodule
