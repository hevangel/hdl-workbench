// top: c29_udp
primitive mux2 (out, s, a, b);
  output out; input s, a, b;
  table
    // s a b : out
       0 0 ? : 0;
       0 1 ? : 1;
       1 ? 0 : 0;
       1 ? 1 : 1;
       x 0 0 : 0;
       x 1 1 : 1;
  endtable
endprimitive
primitive latch (q, en, d);
  output q; reg q; input en, d;
  table
    1 0 : ? : 0;
    1 1 : ? : 1;
    0 ? : ? : -;
  endtable
endprimitive
primitive dff (q, clk, d);
  output q; reg q; input clk, d;
  initial q = 1'b1;
  table
    (01) 0 : ? : 0;
    (01) 1 : ? : 1;
    (0?) 1 : 1 : 1;
    (0?) 0 : 0 : 0;
    (?0) ? : ? : -;
    ?   (??) : ? : -;
  endtable
endprimitive
primitive dffr (q, clk, d, rst);
  output q; reg q; input clk, d, rst;
  initial q = 0;
  table
    ? ? 1 : ? : 0;
    r 0 0 : ? : 0;
    r 1 0 : ? : 1;
    f ? 0 : ? : -;
    ? * 0 : ? : -;
    ? ? f : ? : -;
  endtable
endprimitive
primitive nomatch (o, a); output o; input a; table 1 : 1; endtable endprimitive
module c29_udp;
  logic s, a, b, en, d, clk, rst; wire m, l, q, qr, nm;
  mux2 u1 (m, s, a, b); latch u2 (l, en, d); dff u3 (q, clk, d); dffr u4 (qr, clk, d, rst); nomatch u5 (nm, a);
  wire #3 md; mux2 #(2) u6 (md, s, a, b);
  logic vals[3] = '{1'b0, 1'b1, 1'bx};
  initial begin
    #0 $display("T|29.6|init q=%b qr=%b", q, qr);
    foreach (vals[i]) foreach (vals[j]) foreach (vals[k]) begin s = vals[i]; a = vals[j]; b = vals[k]; #1 $display("T|29.3|s=%b a=%b b=%b m=%b nm=%b", s, a, b, m, nm); end
    en = 1; d = 0; #1 $display("T|29.4|latch %b", l); en = 0; d = 1; #1 $display("T|29.4|latch hold %b", l); en = 1'bx; #1 $display("T|29.4|latch en=x %b", l);
    rst = 0; clk = 0; d = 0; #1 clk = 1; #1 $display("T|29.5|dff q=%b qr=%b", q, qr);
    d = 1; #1 $display("T|29.5|d change no clk q=%b qr=%b", q, qr);
    clk = 0; #1 clk = 1; #1 $display("T|29.5|posedge q=%b qr=%b", q, qr);
    clk = 1'bx; #1 $display("T|29.5|1->x q=%b qr=%b", q, qr);
    clk = 0; #1 d = 0; clk = 1'bx; #1 $display("T|29.5|0->x d=0 q=%b qr=%b", q, qr);
    rst = 1; #1 $display("T|29.5|rst qr=%b", qr);
    s = 0; a = 1; #1 a = 0; #5 $display("T|29.7|delayed md=%b", md);
  end
endmodule
