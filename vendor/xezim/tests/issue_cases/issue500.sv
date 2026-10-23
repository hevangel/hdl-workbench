// §11.8.1: a bitwise operator's result is signed when both operands are signed.
// §10.7 / §10.8: assigning to a class property converts the value to the
// property's declared type, so a signed `int` property reads back signed.
class holder;
  int p;
  function void set(int v);
    p = v & 32'shffff_ffff;       // signed & signed, stored into an int property
  endfunction
endclass
module top;
  int j = 7, errors = 0;
  logic signed [15:0] s = -50;
  holder h;
  wire signed [31:0] c_and = (j & 255) + (s + 1);
  task automatic ck(string label, longint got, longint want);
    if (got != want) begin errors++; $display("FAIL %s: got %0d, want %0d", label, got, want); end
    else $display("PASS %s", label);
  endtask
  initial begin
    ck("(j & 255) + (s + 1)", (j & 255) + (s + 1), -42);
    ck("(j | 0) + (s + 1)", (j | 0) + (s + 1), -42);
    ck("(j ^ 0) + (s + 1)", (j ^ 0) + (s + 1), -42);
    ck("((j - 8) & 255) - 300 < 0", ((j - 8) & 255) - 300 < 0, 1);
    #1 ck("continuous (j & 255) + (s + 1)", c_and, -42);
    h = new;
    h.set(-10);
    ck("property p", h.p, -10);
    ck("property p < 0", h.p < 0, 1);
    if (errors == 0) $display("ALL PASS"); else $display("FAILED %0d", errors);
  end
endmodule
