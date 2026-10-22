// top: c06ic
module ic_src(output logic [3:0] o); assign o = 4'ha; endmodule
module ic_dst(input logic [3:0] i); initial #1 $display("T|6.6.8|dst got %h", i); endmodule
module c06ic;
  interconnect ic;
  ic_src s(.o(ic));
  ic_dst d(.i(ic));
  // 6.9.2 vectored/scalared, 6.20.7 $ parameter
  wire vectored [7:0] vw = 8'h3c;
  wire scalared [7:0] sw = 8'hc3;
  localparam P = $;
  initial #2 $display("T|6.9.2|%h %h %0d", vw, sw, $isunbounded(P));
endmodule
