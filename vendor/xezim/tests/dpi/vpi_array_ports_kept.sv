// An unpacked-array port that keeps storage of its own, beside one that
// collapses onto its actual, for tests/dpi/vpi_array_ports_kept.c (see
// tests/strings/vpi_array_ports.rs).
module four (input logic clk, input logic [7:0] a [0:1], output logic [7:0] q);
  always_ff @(posedge clk) q <= a[0];
endmodule

module top (input logic [7:0] e [0:1], input logic [7:0] k [0:1]);
  logic clk = 0;
  always #5 clk = ~clk;
  logic [7:0] qe, qk;
  // A second continuous driver on an element of the port: the drivers stay
  // on the names they were elaborated for.
  four ue (.clk(clk), .a(e), .q(qe));
  assign ue.a[1] = 8'h55;
  // Collapses.
  four uk (.clk(clk), .a(k), .q(qk));
  initial begin
    #7;
    $display("T|kept|%0d %0d", qe, qk);
    $finish;
  end
endmodule
