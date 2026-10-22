// top: c2427
// 26 packages
package pa;
  typedef enum {RED, GREEN} col_t;
  int cnt = 1;
  function int inc(); cnt++; return cnt; endfunction
  localparam int PW = 12;
endpackage
package pb;
  import pa::*;
  export pa::col_t;
  export pa::inc;
  int bval = 5;
endpackage
package pc;
  int cnt = 50;     // same name as pa::cnt
  int only_c = 7;
endpackage
package pexp;
  import pa::*;
  export *::*;
  int pe = 3;
endpackage
// $unit items
int unit_var = 99;
typedef logic [2:0] unit_t;
function automatic int unit_fn(int x); return x + 1000; endfunction
// 24 programs
program automatic prg(input logic clk, output logic [3:0] pout);
  int pv = 0;
  initial begin
    @(posedge clk);
    pout <= 4'h5;
    $display("T|24.3a|program runs in reactive t=%0t", $time);
    repeat (2) @(posedge clk);
    pv = 1;
  end
  final $display("T|24.7|program final");
endprogram
// 27 generate
module gen #(parameter N = 3, parameter MODE = 1);
  genvar gi;
  logic [N-1:0] bits;
  for (gi = 0; gi < N; gi++) begin : g_loop
    localparam int LP = gi * 10;
    logic l;
    assign l = (gi % 2);
    assign bits[gi] = l;
  end
  if (MODE == 1) begin : g_if
    int which = 1;
  end else if (MODE == 2) begin : g_if
    int which = 2;
  end else begin : g_if
    int which = 3;
  end
  case (N)
    1: begin : g_case int cv = 100; end
    3: begin : g_case int cv = 300; end
    default: begin : g_case int cv = -1; end
  endcase
  // nested loops
  for (genvar i = 0; i < 2; i++) begin : outer_l
    for (genvar j = 0; j < 2; j++) begin : inner_l
      localparam int IJ = i * 2 + j;
      int v = IJ;
    end
  end
  // unnamed generate block naming: genblk
  if (1) begin int anon = 5; end
  // conditional generate with no block name
  generate
    if (N > 2) begin : g_gen_kw
      wire big = 1'b1;
    end
  endgenerate
endmodule
module c2427;
  import pb::*;
  import pc::only_c;
  logic clk = 0;
  always #5 clk = ~clk;
  logic [3:0] po;
  prg p(clk, po);
  gen #(.N(3), .MODE(2)) g1();
  gen #(.N(1), .MODE(7)) g2();
  initial begin
    col_t c = pa::GREEN;
    #1;
    // 26.3 explicit and wildcard imports
    $display("T|26.3a|%0d %s", bval, c.name());
    $display("T|26.3b|%0d %0d", inc(), pa::cnt);
    $display("T|26.3c|%0d %0d", pc::cnt, only_c);
    $display("T|26.3d|%0d", pa::PW);
    $display("T|26.6|%0d", pexp::pe);
    // $unit
    $display("T|26.2|%0d %0d %0d", unit_var, $bits(unit_t), unit_fn(1));
    $display("T|26.2b|%0d", $unit::unit_var);
    // 27 generate hierarchical refs
    $display("T|27.4a|%b %0d", g1.bits, g1.g_loop[2].LP);
    $display("T|27.4b|%b", g1.g_loop[1].l);
    $display("T|27.5a|%0d %0d", g1.g_if.which, g2.g_if.which);
    $display("T|27.5b|%0d %0d", g1.g_case.cv, g2.g_case.cv);
    $display("T|27.4c|%0d %0d", g1.outer_l[1].inner_l[0].v, g1.outer_l[1].inner_l[1].IJ);
    $display("T|27.6|%0d", g1.genblk5.anon);
    $display("T|27.3|%b", g1.g_gen_kw.big);
    // 24 program timing
    #40 $display("T|24.3b|po=%h", po);
    $finish;
  end
endmodule
