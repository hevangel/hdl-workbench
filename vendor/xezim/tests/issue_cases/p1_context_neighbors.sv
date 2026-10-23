module top;
  bit clk;
  event e;
  int errors, a, b, c, d, h;
  task stat_delay; #1; endtask
  task automatic auto_delay; #1; endtask
  task stat_wait; @e; endtask
  class D; task wait_delay; #1; endtask endclass
  always @(posedge clk) begin int n; n++; a = n; end
  always @(posedge clk) begin logic [7:0] n; if(n===8'bx) n=0; n++; b=n; end
  always @(posedge clk) begin static int n = 10; n++; c=n; end
  always @(posedge clk) begin automatic int n = 0; n++; d=n; end
  always @(posedge clk) begin : named
    int n; n++; h=n;
  end
  initial stat_wait;
  initial begin D obj; obj = new; obj.wait_delay; end
  initial begin #1 ->e; end
  initial begin
    stat_delay; auto_delay;
    repeat(3) begin #1 clk=1; #1 clk=0; end
    if(a!=3 || b!=3 || c!=13 || d!=1 || h!=3) errors++;
    $display("T|counts=%0d %0d %0d %0d %0d errors=%0d",a,b,c,d,h,errors);
    $finish;
  end
endmodule
