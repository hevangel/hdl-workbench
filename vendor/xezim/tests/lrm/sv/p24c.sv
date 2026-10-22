// top: t24c
// 24.7 program control: implicit $finish, $exit, threads of an ending program
program p1;
  initial begin
    fork
      forever #3 $display("T|24.7a|p1 child t=%0t", $time);
    join_none
    #10 $display("T|24.7b|p1 done t=%0t", $time);
  end
endprogram
program p2;
  initial begin
    #4 $display("T|24.7c|p2 exit t=%0t", $time);
    $exit;
    $display("T|24.7d|after exit");
  end
  initial #20 $display("T|24.7e|p2 second initial t=%0t", $time);
endprogram
module t24c;
  logic clk = 0;
  always #1 clk = ~clk;
  p1 a(); p2 b();
  final $display("T|24.7g|end t=%0t", $time);
endmodule
