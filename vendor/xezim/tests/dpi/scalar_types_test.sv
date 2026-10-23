module tb;
  import "DPI-C" function bit st_not(input bit x);
  import "DPI-C" function bit st_pass(input bit x);
  import "DPI-C" function bit st_and(input bit a, input bit b);
  int f = 0;
  `define CK(e,m) if(!(e)) begin f++; $display("FAIL %s", m); end
  initial begin
    `CK(st_not(1'b0)===1'b1, "not0")
    `CK(st_not(1'b1)===1'b0, "not1")
    `CK(st_pass(1'b1)===1'b1, "pass1")
    `CK(st_and(1'b1,1'b1)===1'b1, "and11")
    `CK(st_and(1'b1,1'b0)===1'b0, "and10")
    if(f==0) $display("TEST_PASS"); else $display("TEST_FAIL count=%0d", f);
    $finish;
  end
endmodule
