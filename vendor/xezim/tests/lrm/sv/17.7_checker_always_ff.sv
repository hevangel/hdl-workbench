// top: c17_simple
checker req_ack(logic req, logic ack, event clk_e);
  int nreq = 0;
  a1: assert property (@clk_e req |-> ##[1:2] ack) else $display("T|17.2|req_ack F t=%0t", $time);
  always_ff @clk_e if (req) nreq <= nreq + 1;
  final $display("T|17.7|nreq = %0d", nreq);
endchecker
module c17_simple;
  bit clk; logic req, ack;
  always #5 clk = ~clk;
  initial begin
    req = 0; ack = 0;
    @(negedge clk) req = 1; @(negedge clk) req = 0; ack = 1; @(negedge clk) ack = 0;
    @(negedge clk) req = 1; @(negedge clk) req = 0; repeat (3) @(negedge clk);
    @(negedge clk) req = 1; @(negedge clk) req = 0; @(negedge clk); ack = 1; @(negedge clk) ack = 0;
    repeat (2) @(negedge clk); $finish;
  end
  req_ack ck1(req, ack, posedge clk);
endmodule
