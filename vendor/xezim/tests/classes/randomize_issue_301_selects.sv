// #301 neighbours: selects of multi-dimensional packed rand members in
// constraints (part-selects of either dimension, indexed part-selects, a
// rand index, elements of an unpacked array of packed arrays, 3-D loops,
// $countones of an element), and a class the joint solver does not take
// (a rand real member), whose loop over a state vector the trials solve.
module issue_301_selects;
  class c1; rand bit [9:1][8:1] W; constraint c { W[5][8:5] == 4'ha; W[7:6] == 16'h1234; W[2][3 +: 2] == 2'b10; W[9][8 -: 3] == 3'b101; }
    function bit ok(); return W[5][8:5] == 4'ha && W[7:6] == 16'h1234 && W[2][3 +: 2] == 2'b10 && W[9][8 -: 3] == 3'b101; endfunction endclass
  class c2; rand bit [31:0][47:0] W; rand bit [4:0] k; constraint c { W[k] == 48'h5a5a; k > 3; W[0][47:40] == W[31][7:0]; }
    function bit ok(); return W[k] == 48'h5a5a && k > 3 && W[0][47:40] == W[31][7:0]; endfunction endclass
  class c3; rand bit [8:0][7:0] arr[3]; constraint c { foreach (arr[i]) { arr[i][2] == i; arr[i][8] != arr[i][0]; } }
    function bit ok(); foreach (arr[i]) if (arr[i][2] != i || arr[i][8] == arr[i][0]) return 0; return 1; endfunction endclass
  class c4; rand bit [3:0][7:0] W; constraint c { W[3:2] == W[1:0]; W[0] inside {[1:5]}; W[1] dist {7 := 1, 9 := 1}; }
    function bit ok(); return W[3:2] == W[1:0] && W[0] inside {[1:5]} && W[1] inside {7, 9}; endfunction endclass
  class c5; rand bit [8:0][7:0] W, V; constraint c { W < V; W[8] == V[8]; V[0] == 0; }
    function bit ok(); return W < V && W[8] == V[8] && V[0] == 0; endfunction endclass
  class c6; rand bit [2:0][4:0][7:0] W; constraint c { foreach (W[i, j]) W[i][j] == i * 5 + j; }
    function bit ok(); foreach (W[i, j]) if (W[i][j] != i * 5 + j) return 0; return 1; endfunction endclass
  class c7; rand bit [2:0][4:0][7:0] W; constraint c { foreach (W[i]) W[i][4:3] == i; }
    function bit ok(); foreach (W[i]) if (W[i][4:3] != i) return 0; return 1; endfunction endclass
  class c8; rand bit [15:0][7:0] W; constraint c { W[15:8] == ~W[7:0]; $countones(W[3]) == 2; }
    function bit ok(); return W[15:8] == ~W[7:0] && $countones(W[3]) == 2; endfunction endclass
  class b2; rand bit [8:0][7:0] W; endclass
  class b1; rand bit [71:0] W; endclass
  class b64; rand bit [7:0][7:0] W; endclass
  class t2; rand b2 a, b; rand real f; bit [0:0] sel; constraint c { foreach (sel[i]) { b.W == a.W; } } function new(); a = new(); b = new(); endfunction
    function bit ok(); return a.W == b.W; endfunction endclass
  class t1; rand b1 a, b; rand real f; bit [0:0] sel; constraint c { foreach (sel[i]) { b.W == a.W; } } function new(); a = new(); b = new(); endfunction
    function bit ok(); return a.W == b.W; endfunction endclass
  class t64; rand b64 a, b; rand real f; bit [0:0] sel; constraint c { foreach (sel[i]) { b.W == a.W; } } function new(); a = new(); b = new(); endfunction
    function bit ok(); return a.W == b.W; endfunction endclass
  initial begin
    c1 a = new(); c2 b = new(); c3 c = new(); c4 d = new(); c5 e = new(); c6 f = new(); c7 g = new(); c8 h = new();
    t2 x2 = new(); t1 x1 = new(); t64 x64 = new();
    int r;
    repeat (2) begin
      r = a.randomize(); $display("T|c1 r=%0d ok=%0d", r, a.ok());
      r = b.randomize(); $display("T|c2 r=%0d ok=%0d", r, b.ok());
      r = c.randomize(); $display("T|c3 r=%0d ok=%0d", r, c.ok());
      r = d.randomize(); $display("T|c4 r=%0d ok=%0d", r, d.ok());
      r = e.randomize(); $display("T|c5 r=%0d ok=%0d", r, e.ok());
      r = f.randomize(); $display("T|c6 r=%0d ok=%0d", r, f.ok());
      r = g.randomize(); $display("T|c7 r=%0d ok=%0d", r, g.ok());
      r = h.randomize(); $display("T|c8 r=%0d ok=%0d", r, h.ok());
      r = x2.randomize(); $display("T|t2 r=%0d ok=%0d", r, x2.ok());
      r = x1.randomize(); $display("T|t1 r=%0d ok=%0d", r, x1.ok());
      r = x64.randomize(); $display("T|t64 r=%0d ok=%0d", r, x64.ok());
    end
  end
endmodule
