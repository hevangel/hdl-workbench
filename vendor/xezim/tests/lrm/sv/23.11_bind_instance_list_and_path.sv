// top: r_bind
module chk #(parameter P = 1) (input [7:0] a);
  initial #1 $display("T|bind|%m P=%0d a=%h", P, a);
endmodule
module tgt #(parameter W = 8, parameter V = 0);
  logic [W-1:0] sig = V;
endmodule
module mid; tgt #(.V(8'h44)) deep(); endmodule
module r_bind;
  tgt #(.V(8'h11)) u1();
  tgt #(.V(8'h22)) u2();
  tgt #(.V(8'h33)) u3();
  mid m();
  bind tgt: u1, u3 chk c_list(.a(sig));       // only u1 and u3
  bind u2 chk #(.P(W)) c_path(.a(sig));      // instance path, parameter of the target
  bind m.deep chk #(7) c_deep(.a(sig));      // deeper instance path
endmodule
