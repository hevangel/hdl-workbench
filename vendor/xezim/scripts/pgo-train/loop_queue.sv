
module tb;
  logic clk = 0; always #5 clk = ~clk;
  typedef struct packed { logic [7:0] a; } e_t;
  e_t [3:0][7:0] bqueue;
  logic [2:0] wptr [0:3];
  always @(posedge clk)
    for (int i = 0; i < 4; i++) begin
      bqueue[i][wptr[i]] <= e_t'(8'(i * 8'h11 + 1));
      wptr[i] <= wptr[i] + 1;
    end
  initial begin
    bqueue = '0; foreach (wptr[i]) wptr[i] = 3'(i);
    repeat (20000) @(posedge clk);
    #1 $display("QUEUE %h", bqueue);
    $finish;
  end
endmodule
