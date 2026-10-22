// §11.6.1, §11.8.2: propagate expression type into context-determined operands.
module top;
  logic signed [7:0] s = 8'shfc;
  logic [15:0] acc = 0;
  logic signed [15:0] signed_acc = 0;
  logic [15:0] ca, cb, cc, cd;
  logic [79:0] wide_acc = 0;
  int errors;
  assign ca = acc + (s >>> 1);
  always_comb cb = acc + (s >>> 1);
  task automatic ck(string label, logic [79:0] got, logic [79:0] want);
    if (got !== want) begin errors++; $display("FAIL %s %h %h", label, got, want); end
    else $display("PASS %s", label);
  endtask
  initial begin
    #1;
    cc = acc + (s >>> 1); cd = acc + (s >> 1);
    ck("continuous", 16'(ca), 16'h007e);
    ck("always_comb", 16'(cb), 16'h007e);
    ck("procedural arithmetic", 16'(cc), 16'h007e);
    ck("procedural logical", 16'(cd), 16'h007e);
    ck("reversed operands", 16'((s >>> 1) + acc), 16'h007e);
    ck("nested arithmetic", 16'(acc + ((s + s) >>> 1)), 16'h00fc);
    ck("left shift", 16'(acc + (s << 1)), 16'h01f8);
    ck("bitwise context", 16'(acc | (s >>> 1)), 16'h007e);
    ck("unary complement", 16'(acc + ~(s >>> 1)), 16'hff81);
    ck("unary minus", 16'(acc + -(s >>> 1)), 16'hff82);
    ck("conditional context", 16'(1'b1 ? (s >>> 1) : acc), 16'h007e);
    ck("unknown conditional", 16'(1'bx ? (s >>> 1) : 16'h007e), 16'h007e);
    ck("comparison context", 16'((16'h007e == (s >>> 1))), 1);
    ck("signed arithmetic control", 16'(signed_acc + (s >>> 1)), 80'hffffffffffffffff_fffe);
    ck("signed logical control", 16'(signed_acc + (s >> 1)), 16'h7ffe);
    ck("sized cast boundary", 16'(acc + 8'(s >>> 1)), 16'h00fe);
    ck("concat boundary", 16'(acc + {s >>> 1}), 16'h00fe);
    ck("unsigned cast", 16'(acc + (unsigned'(s) >>> 1)), 16'h007e);
    ck("signed cast", 16'(acc + (signed'(s) >>> 1)), 16'h007e);
    ck("wide context", wide_acc + (s >>> 1), 80'h7e);
    if (errors == 0) $display("ALL PASS"); else $display("FAILED %0d", errors);
    $finish;
  end
endmodule
