// top: t23h
interface sif; logic [3:0] s = 4'h6; endinterface
module leaf;
  int lv = 1;
  event ev;
  wire [3:0] lw = 4'h3;
  int hits = 0;
  always @(ev) hits++;
  task automatic wait_ev(output int t); @(ev); t = $time; endtask
  function int get(); return lv; endfunction
  initial begin : nb
    int inner = 77;
  end
endmodule
module upl;
  // upward references by module type name and by instance name of an ancestor
  initial #5 $display("T|23.8a|%m by_type=%0d by_inst=%0d top=%0d", mtype.tv, inst_x.tv, t23h.topv);
  // unqualified function not declared here: resolved upward (23.9)
  initial #6 $display("T|23.9a|%m helper=%0d", helper(2));
endmodule
module mtype;
  upl ul();
  int tv = 42;
  function int helper(int x); return x * 1000 + tv; endfunction
  leaf l();
  sif si();
endmodule
module wrapper;
  mtype inst_x();
endmodule
module arrm(input int a);
  leaf sub();
endmodule
module \esc.mod ; int ev = 5; endmodule
module t23h;
  int topv = 9;
  wrapper w();
  arrm ar[1:0] (.a(3));
  for (genvar g = 0; g < 3; g++) begin : gg
    leaf gl();
  end
  \esc.mod \e.x ();
  wire [3:0] mirror = w.inst_x.l.lw;          // hierarchical in continuous assign
  int seen = 0;
  always @(w.inst_x.l.lv) seen++;             // hierarchical in sensitivity
  // a struct variable named like an instance: member select vs hierarchical
  typedef struct { int lv; } st_t;
  st_t w2;
  initial begin
    int t;
    w2.lv = 55;
    #1;
    $display("T|23.6a|%0d %0d %0d %0d", w.inst_x.l.lv, ar[1].sub.lv, gg[2].gl.lv, $root.t23h.gg[0].gl.lw);
    $display("T|23.6b|mirror=%h get=%0d", mirror, w.inst_x.l.get());
    w.inst_x.l.lv = 10; #1 $display("T|23.6c|seen=%0d", seen);
    fork w.inst_x.l.wait_ev(t); begin #1 -> w.inst_x.l.ev; end join
    #1 $display("T|23.6d|t=%0d hits=%0d", t, w.inst_x.l.hits);
    $display("T|23.6e|nb=%0d", w.inst_x.l.nb.inner);
    $display("T|23.6f|if=%h esc=%0d", w.inst_x.si.s, \e.x .ev);
    $display("T|23.7|w2=%0d", w2.lv);
    force w.inst_x.l.lw = 4'hc; #1 $display("T|23.6g|mirror=%h", mirror); release w.inst_x.l.lw; #1 $display("T|23.6h|mirror=%h", mirror);
    #10 $finish;
  end
endmodule
