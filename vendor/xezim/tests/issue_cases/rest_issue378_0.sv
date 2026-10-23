// A local declared with a class type parameter (T x;) has the type T is
// specialized to (IEEE 1800-2017 8.25). Each method is called 2000 times, so
// the last 1000 calls run after xezim compiles it (default tier: 1000 calls).
class C #(type T = int);
  function logic neg(logic [7:0] v);
    T x;
    x = v;
    return x < 0;
  endfunction
  function longint ones();
    T x;
    x = '1;
    return x;
  endfunction
endclass
module top;
  initial begin
    C #(byte) cb = new;
    C #(logic [63:0]) cw = new;
    int bad_b = 0, bad_w = 0, first_b = -1, first_w = -1;
    for (int k = 0; k < 2000; k++) begin
      if (cb.neg(8'hC8) !== 1'b1) begin bad_b++; if (first_b < 0) first_b = k; end
      if (cw.ones() != -1) begin bad_w++; if (first_w < 0) first_w = k; end
    end
    if (bad_b == 0 && bad_w == 0) $display("PASS");
    else $display("FAIL wrong results: T=byte neg(8'hC8) %0d (from call %0d, last %b), T=logic [63:0] ones() %0d (from call %0d, last %h)",
                  bad_b, first_b, cb.neg(8'hC8), bad_w, first_w, cw.ones());
  end
endmodule
