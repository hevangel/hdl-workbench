// top: c34_prot
module c34_prot;
  initial $display("T|34|before");
`pragma protect begin
  initial $display("T|34|inside protect region");
`pragma protect end
  initial #1 $display("T|34|after");
endmodule
