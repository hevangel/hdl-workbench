// Indexed / part-select stores INTO an element of an unpacked-array class
// property whose element is a multi-dim packed array (IEEE 1800-2017
// §7.4.1/§7.4.2): `arr[1][2]` writes the 8-bit element [23:16] of the
// 72-bit `arr[1]`, and `arr[1][8:0]` writes elements 8..0 (72 bits).
// An x/z or out-of-range select writes nothing (§11.5.1).
class c9;
  bit [8:0][7:0] arr[3];
  function void write1(int i, int k, bit v); arr[i][k] = v; endfunction
  function void write2(int i, int w); arr[i][8:0] = w; endfunction
  function void write2b(int i, int w); arr[i][4:3] = w; endfunction
  function void write3(int i, int b, int w); arr[i][b +: 2] = w; endfunction
  function void write4(int i, int b, int w); arr[i][b -: 3] = w; endfunction
  function void twrite(int i, int k, bit v); this.arr[i][k] = v; endfunction
  function logic [71:0] rd(int i); return arr[i]; endfunction
endclass

class hot;
  bit [8:0][7:0] arr[2];
  function void wb(int i, int k, bit v); arr[i][k] = v; endfunction
  function void wr(int i, int w); arr[i][8:0] = w; endfunction
  function void twb(int i, int k, bit v); this.arr[i][k] = v; endfunction
  function logic [71:0] rd(int i); return arr[i]; endfunction
endclass

module top;
  c9 d;
  hot h;
  logic [7:0] kx;
  initial begin
    d = new();
    d.arr[0] = 72'h03020100AA;
    d.arr[1] = 72'h0102030405;
    d.arr[2] = 72'h1122334455;
    // element store through a bare member in a method
    d.write1(1, 2, 1'b1);
    $display("T| w1  %h", d.rd(1));
    d.write1(1, 8, 1'b1);
    $display("T| w1b %h", d.rd(1));
    // whole inner-dimension range, sub-range, +: and -:
    d.write2(1, 9'h1FF);
    $display("T| w2  %h", d.rd(1));
    d.write2b(1, 2'h2);
    $display("T| w2b %h", d.rd(1));
    d.write3(1, 3, 16'hBCDE);
    $display("T| w3  %h", d.rd(1));
    d.write4(1, 7, 24'hC0FFEE);
    $display("T| w4  %h", d.rd(1));
    // `this.`-qualified store
    d.twrite(2, 5, 1'b1);
    $display("T| tw  %h", d.rd(2));
    // x/z index: the select writes nothing (§11.5.1)
    kx = 8'hx;
    d.arr[0][kx] = 1'b1;
    $display("T| xz  %h", d.rd(0));
    // out-of-range element index: writes nothing
    d.write1(0, 9, 1'b1);
    $display("T| oob %h", d.rd(0));
    // whole-element store from module scope still whole
    d.arr[0][2] = 8'h7E;
    $display("T| elem %h", d.rd(0));
    // hot loop: the same stores after the methods tier to compiled form
    h = new();
    h.arr[0] = 72'h0102030405;
    h.arr[1] = 72'hA1A2A3A4A5;
    for (int n = 0; n < 1500; n++) begin
      h.wb(0, 2, 1'b1); h.wb(0, 4, 1'b0);
      h.twb(0, 5, 1'b0);
      h.wr(1, 9'h0AA);
      h.wr(1, 9'h1F0);
    end
    $display("T| hot %h %h", h.rd(0), h.rd(1));
  end
endmodule
