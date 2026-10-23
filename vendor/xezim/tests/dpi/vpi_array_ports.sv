// Instances with unpacked-array input ports connected to whole arrays, for
// tests/dpi/vpi_array_ports.c (see tests/strings/vpi_array_ports.rs). Every
// instance copies the elements of its port to its outputs on each clock edge;
// the port arrays of `top` are driven by nothing but the VPI library.
module leaf #(parameter int W = 8, parameter int N = 2) (
  input  logic         clk,
  input  logic [W-1:0] a [0:N-1],
  output logic [W-1:0] q [0:N-1]);
  always_ff @(posedge clk) for (int i = 0; i < N; i++) q[i] <= a[i];
endmodule

module mid (input logic clk, input logic [7:0] a [0:1], output logic [7:0] q [0:1]);
  leaf #(8, 2) l (.clk(clk), .a(a), .q(q));
endmodule

module outer (input logic clk, input logic [7:0] a [0:1], output logic [7:0] q [0:1]);
  mid m (.clk(clk), .a(a), .q(q));
endmodule

module grid (input logic clk, input logic [7:0] a [0:1][0:2],
             output logic [7:0] q [0:1][0:2]);
  always_ff @(posedge clk)
    for (int i = 0; i < 2; i++) for (int j = 0; j < 3; j++) q[i][j] <= a[i][j];
endmodule

module down (input logic clk, input logic [7:0] a [3:0], output logic [7:0] q [3:0]);
  always_ff @(posedge clk) for (int i = 0; i < 4; i++) q[i] <= a[i];
endmodule

module peek #(parameter int W = 8) (input logic clk, input logic [W-1:0] a [0:7],
                                    output logic [W-1:0] q3, output logic [W-1:0] q5);
  always_ff @(posedge clk) begin q3 <= a[3]; q5 <= a[5]; end
endmodule

module top (
  input logic [7:0]  a8 [0:1],
  input logic [7:0]  c8 [0:1],
  input logic [99:0] w  [0:1],
  input logic        s1 [0:3],
  input logic [7:0]  r8 [0:1],
  input logic [7:0]  d8 [0:1],
  input logic [7:0]  g8 [0:1][0:2],
  input logic [7:0]  n8 [3:0]);
  logic clk = 0;
  always #5 clk = ~clk;
  // Variables (packed-arena memories when arrays this size have no element
  // names) rather than nets.
  logic [7:0] m8 [0:7];
  logic [63:0] m64 [0:7];
  logic m1 [0:7];
  logic [7:0] qa [0:1], qc [0:1], qr [0:1], qd [0:1], qg [0:1][0:2], qn [3:0];
  logic [99:0] qw [0:1];
  logic [7:0] q83, q85;
  logic [63:0] q643, q645;
  logic qs [0:3], q13, q15;
  leaf #(8, 2)   u1   (.clk(clk), .a(a8),  .q(qa));
  outer          o    (.clk(clk), .a(c8),  .q(qc));
  leaf #(100, 2) uw   (.clk(clk), .a(w),   .q(qw));
  leaf #(1, 4)   us   (.clk(clk), .a(s1),  .q(qs));
  leaf #(8, 2)   ur   (.clk(clk), .a(r8),  .q(qr));
  leaf #(8, 2)   ud   (.clk(clk), .a(d8),  .q(qd));
  grid           ug   (.clk(clk), .a(g8),  .q(qg));
  down           un   (.clk(clk), .a(n8),  .q(qn));
  peek #(8)      um8  (.clk(clk), .a(m8),  .q3(q83),  .q5(q85));
  peek #(64)     um64 (.clk(clk), .a(m64), .q3(q643), .q5(q645));
  peek #(1)      um1  (.clk(clk), .a(m1),  .q3(q13),  .q5(q15));
  initial begin
    #7;
    $display("T|one level|%0d %0d", qa[0], qa[1]);
    $display("T|three levels|%0d %0d", qc[0], qc[1]);
    $display("T|wide|%h %h", qw[0], qw[1]);
    $display("T|one bit|%b%b%b%b", qs[0], qs[1], qs[2], qs[3]);
    $display("T|parent write|%0d %0d", qr[0], qr[1]);
    $display("T|whole array|%0d %0d", qd[0], qd[1]);
    $display("T|two dims|%0d %0d", qg[0][1], qg[1][2]);
    $display("T|descending|%0d %0d", qn[3], qn[0]);
    $display("T|variable 8|%0d %0d", q83, q85);
    $display("T|variable 64|%h %h", q643, q645);
    $display("T|variable 1|%b%b", q13, q15);
    $display("T|names|%0d %0d %0d %0d %0d", a8[0], o.a[1], o.m.a[1], c8[0], ur.a[1]);
    #7; $finish;
  end
endmodule
