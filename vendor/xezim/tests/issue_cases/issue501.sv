module top;
  logic signed [7:0] a = -10, b = 3;   // a is 8'hf6: 246 as unsigned
  logic [7:0] u = 100;
  logic [15:0] u16 = 1000;
  logic z = 0;
  int errors = 0;
  wire [7:0] c_div = z ? '0 : (a / b);   // compiled path
  task automatic ck(string label, logic [15:0] got, logic [15:0] want);
    if (got !== want) begin errors++; $display("FAIL %s: got %h, want %h", label, got, want); end
    else $display("PASS %s", label);
  endtask
  initial begin
    #1;
    ck("continuous z ? '0 : a / b", c_div, 8'd82);
    begin logic [7:0] v;  v = z ? '0 : (a / b);   ck("z ? '0 : a / b", v, 8'd82); end
    begin logic [7:0] v;  v = z ? 8'd0 : (a / b); ck("z ? 8'd0 : a / b", v, 8'd82); end
    begin logic [7:0] v;  v = z ? '0 : (a % b);   ck("z ? '0 : a % b", v, 8'd0); end
    begin logic [15:0] v; v = z ? '0 : (a + b);   ck("z ? '0 : a + b", v, 16'h00f9); end
    begin logic v;        v = (a / b) > u;        ck("(a / b) > u", v, 1'b0); end
    begin logic [7:0] v;  v = u + a / b;          ck("u + a / b", v, 8'hb6); end
    begin logic [15:0] v; v = u16 + (a + b);      ck("u16 + (a + b)", v, 16'd1249); end
    begin logic [7:0] v;  v = z ? 8'sd0 : (a / b); ck("z ? 8'sd0 : a / b (signed)", v, 8'hfd); end
    begin logic [15:0] v; v = {u, a / b};         ck("{u, a / b} (self-determined)", v, 16'h64fd); end
    if (errors == 0) $display("ALL PASS"); else $display("FAILED %0d", errors);
  end
endmodule
