// top: rpd
module rpd;
  int a4[2:0];
  int a5[0:2];
  initial begin
    a4[2] = 2; a4[1] = 1; a4[0] = 0;
    a5[0] = 0; a5[1] = 1; a5[2] = 2;
    $display("T|r1|%p %p", a4, a5);
  end
endmodule
