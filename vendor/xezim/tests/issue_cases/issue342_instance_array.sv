module leaf(input logic [3:0] d, output logic [3:0] q);
  assign q = ~d;
endmodule
module wrap(input logic [7:0] d, output logic [7:0] q);
  leaf ua[1:0] (.d(d), .q(q));                 // the enclosing module's ANSI ports
endmodule
module top;
  logic [7:0] d, e, q1, q2, q3, q4, q5;
  logic [3:0] up [0:1];
  wrap w (.d(d), .q(q1));
  leaf b[0:1] (.d(d), .q(q2));                 // ascending range
  leaf c[1:0] (.d(d), .q({q3[7:4], q3[3:0]})); // output concatenation
  leaf x[1:0] (.d(d ^ e), .q(q4));             // expression
  leaf u[1:0] (.d(up), .q(q5));                // unpacked array, other direction
  task automatic chk(string what, bit ok, logic [7:0] q, logic [3:0] hi, logic [3:0] lo);
    $display("%s %s: q=%h [1].d=%h [0].d=%h", ok ? "PASS" : "FAIL", what, q, hi, lo);
  endtask
  initial begin
    d = 8'h3C; e = 8'h66; up[0] = 4'h5; up[1] = 4'h6;
    #1;
    chk("ansi port  ", q1 === 8'hC3 && w.ua[1].d === 4'h3 && w.ua[0].d === 4'hC, q1, w.ua[1].d, w.ua[0].d);
    chk("ascending  ", q2 === 8'hC3 && b[1].d === 4'hC && b[0].d === 4'h3, q2, b[1].d, b[0].d);
    chk("concat out ", q3 === 8'hC3 && c[1].d === 4'h3 && c[0].d === 4'hC, q3, c[1].d, c[0].d);
    chk("expression ", q4 === 8'hA5 && x[1].d === 4'h5 && x[0].d === 4'hA, q4, x[1].d, x[0].d);
    chk("unpacked   ", q5 === 8'hA9 && u[1].d === 4'h5 && u[0].d === 4'h6, q5, u[1].d, u[0].d);
  end
endmodule
