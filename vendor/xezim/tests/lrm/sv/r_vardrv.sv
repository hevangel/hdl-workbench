// top: r_vardrv
// illegal: a variable driven by an output port and also procedurally
module vo(output logic [3:0] v); assign v = 4'h5; endmodule
module r_vardrv;
  logic [3:0] pv;
  vo u(.v(pv));
  initial begin #1 pv = 4'h9; #1 $display("T|vd|pv=%h", pv); end
endmodule
