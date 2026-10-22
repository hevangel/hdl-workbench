// §27.4 / §27.6: scopes number all constructs, with independent nested ordinals.
module scope_unit(output logic ok);
 if(1)begin:named_first end
 for(genvar k=0;k<2;k++)begin
  for(genvar j=0;j<2;j++)begin logic [3:0] datum=k*2+j;end
 end
 for(genvar k=0;k<2;k++)begin logic [3:0] datum=k+8;end
 assign ok=genblk2[0].genblk1[0].datum==0 && genblk2[1].genblk1[1].datum==3 && genblk3[0].datum==8 && genblk3[1].datum==9;
endmodule
module top;
 logic clk=0,a,b; int hits=0,fails=0;
 scope_unit u(a),v(b);
 always #1 clk=~clk;
 for(genvar i=-1;i<2;i++)begin:g
  ap:assert property(@(posedge clk) i>=-1 && i<2)hits+=i+2;else fails+=i+2;
 end
 initial begin #6;if(a && b && hits==18 && fails==0)$display("PASS generate neighbors");else $display("FAIL generate neighbors a=%b b=%b hits=%0d fails=%0d",a,b,hits,fails);$finish;end
endmodule
