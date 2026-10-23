// top: c36
module leaf (input int i, output logic [3:0] w); assign w = i[3:0]; endmodule
module c36;
  parameter P = 3;
  int a = 1; int c; int cnt; bit clk; wire [3:0] wv;
  leaf u (.i(a), .w(wv));
  always #5 clk = ~clk;
  always @(posedge clk) cnt <= cnt + 1;
  ap: assert property (@(posedge clk) cnt != 2) else $display("T|36|ap fail action t=%0t", $time);
  initial begin
    #1 $probe_vpi;
    #0 $display("T|36|after put c=%0d a=%0d t=%0t", c, a, $time);
    #4 $display("T|36|a before delay %0d t=%0t", a, $time);
    #1 $display("T|36|a after delay %0d t=%0t", a, $time);
    #20 $probe_vpi;
    $finish;
  end
endmodule
