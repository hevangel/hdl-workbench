// top: c28_mos
module c28_mos;
  logic d, g, ng; wire n1, p1, c1, rn1, rp1, rc1;
  wire tA, tB, t1A, t1B, t0A, t0B, rtA, rtB;
  logic dA; logic en;
  nmos m1 (n1, d, g); pmos m2 (p1, d, g); cmos m3 (c1, d, g, ng);
  rnmos m4 (rn1, d, g); rpmos m5 (rp1, d, g); rcmos m6 (rc1, d, g, ng);
  assign tA = dA; tran t1 (tA, tB);
  assign t1A = dA; tranif1 t2 (t1A, t1B, en);
  assign t0A = dA; tranif0 t3 (t0A, t0B, en);
  assign rtA = dA; rtran t4 (rtA, rtB);
  logic vals[4] = '{1'b0, 1'b1, 1'bx, 1'bz};
  initial begin
    foreach (vals[i]) foreach (vals[j]) begin
      d = vals[i]; g = vals[j]; ng = ~g; #1;
      $display("T|28.7|d=%b g=%b nmos=%b/%v pmos=%b/%v cmos=%b/%v rnmos=%v rpmos=%v rcmos=%v", d, g, n1, n1, p1, p1, c1, c1, rn1, rp1, rc1);
    end
    foreach (vals[i]) foreach (vals[j]) begin
      dA = vals[i]; en = vals[j]; #1;
      $display("T|28.8|d=%b en=%b tran=%b/%v tif1=%b/%v tif0=%b/%v rtran=%v", dA, en, tB, tB, t1B, t1B, t0B, t0B, rtB);
    end
  end
endmodule
