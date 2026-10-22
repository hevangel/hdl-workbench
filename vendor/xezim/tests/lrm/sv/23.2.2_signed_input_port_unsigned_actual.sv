// top: r_sx
module sx(input signed [7:0] si, output [15:0] ext, output lt0, output signed [15:0] e2);
  assign ext = si;
  assign lt0 = si < 0;
  assign e2 = si >>> 1;
endmodule
module sxn(si, ext);
  input signed [7:0] si;
  output [15:0] ext;
  assign ext = si;
endmodule
module r_sx;
  wire [15:0] e1, e2, e3, e4, f1, f2; wire l1, l2, l3;
  logic [7:0] uv = 8'h80;
  logic signed [7:0] sv = -8'sd128;
  sx a(.si(8'h80), .ext(e1), .lt0(l1), .e2(f1));
  sx b(.si(uv), .ext(e2), .lt0(l2), .e2(f2));
  sx c(.si(sv), .ext(e3), .lt0(l3), .e2());
  sxn d(uv, e4);
  initial begin
    #1 $display("T|sx1|%h %b %h", e1, l1, f1);
    $display("T|sx2|%h %b %h", e2, l2, f2);
    $display("T|sx3|%h %b", e3, l3);
    $display("T|sx4|%h", e4);
    uv = 8'h81; #1 $display("T|sx5|%h %b %h %h", e2, l2, f2, e4);
  end
endmodule
