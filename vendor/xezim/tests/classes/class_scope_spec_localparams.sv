class holder #(type T = int, string Tname = "<unknown>", int P = 4);
endclass
class cp #(type R = int);
  localparam int W = R::P;
  static function int w(); return W; endfunction
  static function int w2(); return R::P; endfunction
  static function int w3(); int x = W; return x; endfunction
endclass
class vp #(int N = 3);
  localparam int W = N * 2;
  static function int w(); return W; endfunction
endclass
module top;
  initial begin
    $display("T|I01 %0d", cp #(holder #(int, "k", 9))::w());
    $display("T|I02 %0d", cp #(holder #(int, "k", 9))::w2());
    $display("T|I03 %0d", cp #(holder #(int, "k", 9))::w3());
    $display("T|I04 %0d", vp #(5)::w());
  end
endmodule
