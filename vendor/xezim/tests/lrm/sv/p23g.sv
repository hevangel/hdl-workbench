// top: t23g
// 23.10.1 defparam forms and precedence
module leaf #(parameter P = 1, parameter Q = P + 1);
  initial #1 $display("T|%m|P=%0d Q=%0d", P, Q);
endmodule
module mid #(parameter MP = 0);
  leaf l();
  leaf l2();
  defparam l2.P = 40;         // defparam in a submodule
  if (MP > 0) begin : g
    leaf gl();
  end
endmodule
module t23g;
  mid m1();
  mid #(.MP(1)) m2();
  defparam m1.l.P = 7;              // relative hierarchical defparam
  defparam t23g.m2.l.P = 8;         // absolute path
  defparam m2.g.gl.P = 9, m2.g.gl.Q = 90;  // into generate scope, list
  leaf #(.P(3)) prec();
  defparam prec.P = 33;             // defparam wins over instance override
  defparam m1.l2.P = 41;            // higher-level defparam wins over the one in mid
  leaf dep();
  defparam dep.P = 100;             // Q recomputed from overridden P
endmodule
