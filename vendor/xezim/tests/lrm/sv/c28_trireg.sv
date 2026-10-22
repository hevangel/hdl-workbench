// top: c28_trireg
`timescale 1ns/1ns
module c28_trireg;
  logic d, en;
  trireg tr1; trireg (small) tr2; trireg (large) #(0, 0, 50) tr3;
  bufif1 b1 (tr1, d, en); bufif1 b2 (tr2, d, en); bufif1 b3 (tr3, d, en);
  wire [3:0] bus; logic [3:0] dv; buf ga[3:0] (bus, dv);   // array of instances
  initial begin
    d = 1; en = 1; #5 $display("T|28.13|driven tr1=%v tr2=%v tr3=%v", tr1, tr2, tr3);
    en = 0; #5 $display("T|28.13|stored tr1=%v tr2=%v tr3=%v", tr1, tr2, tr3);
    #60 $display("T|28.13|after decay tr3=%v t=%0t", tr3, $time);
    dv = 4'b1010; #1 $display("T|28.3.6|array bus=%b", bus);
  end
endmodule
