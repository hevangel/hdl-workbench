// top: rvc
interface bif(input logic clk);
  logic [7:0] data;
  clocking cb @(posedge clk); input data; endclocking
endinterface
module rvc;
  logic clk = 0;
  always #5 clk = ~clk;
  bif b(clk);
  virtual bif vc;
  initial begin
    vc = b;
    b.data = 8'h77;
    @(vc.cb);
    $display("T|r1|t=%0t vc.cb.data=%h b.cb.data=%h", $time, vc.cb.data, b.cb.data);
    b.data = 8'h12;
    @(b.cb);
    $display("T|r2|t=%0t vc.cb.data=%h b.cb.data=%h", $time, vc.cb.data, b.cb.data);
    $finish;
  end
endmodule
