module top;
  logic clk = 0, a = 0, b = 0, c = 0;
  int n_impl = 0, n_nest = 0, n_if = 0;
  always #5 clk = ~clk;  // rising edges at 5, 15, 25, 35, 45
  // §20.12: from here on, a pass action runs only on a nonvacuous success.
  initial $assertvacuousoff;
  a_impl: assert property (@(posedge clk) a |-> b)
    begin n_impl++; $display("%0t a_impl pass action", $time); end
  a_nest: assert property (@(posedge clk) c |-> (a |-> b))
    begin n_nest++; $display("%0t a_nest pass action", $time); end
  a_if: assert property (@(posedge clk) if (c) (a |-> b))
    begin n_if++; $display("%0t a_if pass action", $time); end
  initial begin
    @(negedge clk) c = 1;                          // edge 15: c=1, a=0: all vacuous
    @(negedge clk) c = 0;                          // edge 25: all vacuous
    @(negedge clk) begin a = 1; b = 1; c = 1; end  // edge 35: all nonvacuous
    @(negedge clk) begin a = 0; b = 0; c = 0; end  // edge 45: all vacuous
    @(negedge clk);
    if (n_impl == 1 && n_nest == 1 && n_if == 1) $display("PASS");
    else $display("FAIL: pass actions a_impl=%0d a_nest=%0d a_if=%0d, want 1 each",
                  n_impl, n_nest, n_if);
    $finish;
  end
endmodule
