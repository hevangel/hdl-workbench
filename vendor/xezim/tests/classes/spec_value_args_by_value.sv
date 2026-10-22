class P #(int N = 4); static int c; endclass
class Q #(N = 4); static int c; endclass
class R #(int A = 1, int B = 2); static int c; endclass
interface vi_if #(int W = 4); logic [W-1:0] d; endinterface
class D #(type T = int); static int c; endclass
module tb;
  localparam int LP = 4;
  initial begin
    P#(4)::c++;
    $display("T|mul|%0d", P#(2*2)::c);
    $display("T|shift|%0d", P#(1<<2)::c);
    $display("T|paren|%0d", P#((8)/2)::c);
    $display("T|dec|%0d", P#('d4)::c);
    $display("T|sized|%0d", P#(4'd4)::c);
    $display("T|named|%0d", P#(.N(3+1))::c);
    $display("T|lp|%0d", P#(LP)::c);
    $display("T|lpx|%0d", P#(LP*1)::c);
    $display("T|dflt|%0d", P#()::c);
    Q#(4)::c++;
    $display("T|q_sized|%0d", Q#(4'd4)::c);
    $display("T|q_expr|%0d", Q#(2+2)::c);
    R#(1,2)::c++;
    $display("T|r_expr|%0d", R#(2-1, 1+1)::c);
    $display("T|r_part|%0d", R#(3-2)::c);
    D#(virtual vi_if#(4))::c++;
    $display("T|vif_mul|%0d", D#(virtual vi_if#(2*2))::c);
    $display("T|vif_dflt|%0d", D#(virtual vi_if)::c);
    $finish;
  end
endmodule
