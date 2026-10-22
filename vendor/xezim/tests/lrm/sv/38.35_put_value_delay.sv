// top: t38   (VPI side: 38.35_put_value_delay.c, gcc -shared -fPIC; xezim --vpi-lib)
module t38;
  int a = 1, b = 2;
  initial begin
    #1 $put_delayed;
    #0 $display("T|a|t=%0t a=%0d b=%0d (expect old values)", $time, a, b);
    #5 $display("T|b|t=%0t a=%0d b=%0d (expect 99 55)", $time, a, b);
  end
endmodule
