module chk (input logic [7:0] arr [0:1], input logic [7:0] brr [1:2]);
  initial #1 $display("arr0=%0d arr1=%0d brr1=%0d brr2=%0d", arr[0], arr[1], brr[1], brr[2]);
endmodule
module top;
  logic [7:0] a [0:3] = '{10, 11, 12, 13};
  logic [7:0] b [0:1] = '{20, 21};
  chk u (.arr(a[1:2]), .brr(b));
endmodule
