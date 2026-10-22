// #301 neighbours: reading and writing selects of multi-dimensional packed
// class members, from inside the class and through handle chains
// (`y.a.W[i][j]`), including part-selects of either dimension and
// elements of an unpacked array of packed arrays.
module issue_301_access;
  class b2; bit [9:1][8:1] W;
    function void show(); $display("T|in %h %h %h %b %b", W[5][8:5], W[6:5], W[5][4 +: 2], W[1][8], W[9][1]); endfunction endclass
  class t2; b2 a; function new(); a = new(); endfunction
    function void show(); $display("T|t2 %h %h %h %b %b", a.W[5][8:5], a.W[6:5], a.W[5][4 +: 2], a.W[1][8], a.W[9][1]); endfunction endclass
  class c3n; bit [3:0][7:0] arr[3];
    function void show(); $display("T|arr %h %h %h %h", arr[1], arr[1][2], arr[1][3:2], arr[1][2][7:4]); endfunction endclass
  initial begin
    t2 y = new(); b2 o = new(); c3n d = new();
    y.a.W = '0;
    y.a.W[1][8] = 1; y.a.W[9][1] = 1; y.a.W[5] = 8'hA5;
    $display("T|w %h", y.a.W);
    $display("T|y %h %h %h %b %b", y.a.W[5][8:5], y.a.W[6:5], y.a.W[5][4 +: 2], y.a.W[1][8], y.a.W[9][2]);
    o.W = 72'h720477de91b96b7bcc;
    $display("T|o %h %h %h %b %b", o.W[5][8:5], o.W[6:5], o.W[5][4 +: 2], o.W[1][8], o.W[9][1]);
    o.show(); y.show();
    d.arr[1] = 32'h11223344;
    d.show();
    $display("T|darr %h %h %h %h", d.arr[1], d.arr[1][2], d.arr[1][3:2], d.arr[1][2][7:4]);
  end
endmodule
