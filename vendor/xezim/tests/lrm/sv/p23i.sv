// top: t23i
int uvar = 1;                         // $unit item
typedef logic [5:0] u6_t;
function automatic int ufn(int x); return x + 500; endfunction
extern module em #(parameter W = 4) (input [W-1:0] a, output [W-1:0] y);
module em #(parameter W = 4) (input [W-1:0] a, output [W-1:0] y);
  assign y = ~a;
endmodule
module chk #(parameter P = 1) (input [7:0] a, input clk);
  int cnt = 0;
  always @(posedge clk) cnt <= cnt + P;
  initial #1 $display("T|23.11a|%m P=%0d a=%h", P, a);
endmodule
interface bif(input logic [7:0] sig); logic [7:0] copy; assign copy = sig; endinterface
module tgt #(parameter W = 8, parameter V = 0) (input clk);
  logic [W-1:0] sig = V;
  int uvar = 2;                        // shadows $unit::uvar
  initial #2 $display("T|26.unit|%m %0d %0d %0d", uvar, $unit::uvar, ufn(1));
endmodule
module holder;
  // bind statement inside a module
  bind tgt: t23i.u2 chk #(.P(5)) c_in(.a(sig), .clk(clk));
endmodule
module t23i;
  logic clk = 0;
  always #5 clk = ~clk;
  tgt #(.V(8'h11)) u1(clk);
  tgt #(.V(8'h22)) u2(clk);
  tgt #(.W(4), .V(4'h3)) u3(clk);
  holder h();
  // bind to every instance of the module type
  bind tgt chk c_all(.a(sig), .clk(clk));
  // bind to an instance list of a module type
  bind tgt: u1, u3 chk #(3) c_list(.*, .a(8'(sig)));
  // bind to an instance path
  bind u2 chk #(.P(W)) c_path(.a(sig), .clk(clk));
  // bind an interface
  bind tgt bif b_if(.sig(sig));
  wire [5:0] eo;
  em #(6) e(.a(6'h0f), .y(eo));
  u6_t uv = 6'h3f;
  initial begin
    #31;
    $display("T|23.11b|%0d %0d %0d", u1.c_all.cnt, u2.c_in.cnt, u3.c_list.cnt);
    $display("T|23.11c|%h %h", u1.b_if.copy, u3.b_if.copy);
    $display("T|23.5|eo=%h", eo);
    $display("T|26.unit2|%0d %0d %0d", uvar, $bits(uv), $unit::ufn(2));
    $finish;
  end
endmodule
