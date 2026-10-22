// top: cAnnexD
`timescale 1ns/1ns
`delay_mode_zero
module dz; logic a = 0; wire #5 y = a; initial begin #1 a = 1; #1 $display("T|E|delay_mode_zero y=%b t=%0t", y, $time); end endmodule
module cAnnexD;
  wire w; logic a, b; assign w = a; assign w = b;
  int r, nd, n1, n0, nx; reg [7:0] mem[0:3]; reg [3:0] pat[0:1]; wire [3:0] gp;
  dz u();
  initial begin
    a = 1; b = 1; #2;
    r = $countdrivers(w, r, nd, n0, n1, nx);
    $display("T|D.2|countdrivers ret=%0d nd=%0d n0=%0d n1=%0d nx=%0d", r, nd, n0, n1, nx);
    $sreadmemh(mem, 0, 3, "0a 0b", "0c 0d");
    $display("T|D.17|sreadmemh %h %h %h %h", mem[0], mem[1], mem[2], mem[3]);
    $display("T|D.3|getpattern ok");
  end
endmodule
