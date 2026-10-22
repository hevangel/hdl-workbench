// top: c10
`timescale 1ns/1ns
module c10;
  logic [3:0] a = 0, b = 0;
  // 10.3.3 continuous assignment delays
  wire [3:0] wd;
  assign #3 wd = a;
  wire [3:0] #2 wn;
  assign wn = a;
  // inertial: pulse shorter than delay is filtered
  logic p = 0;
  wire pd;
  assign #4 pd = p;
  // rise/fall/turnoff delays
  logic r = 0, ren = 1;
  wire rf;
  assign #(2, 5) rf = r;
  wire tz;
  bufif1 #(1, 2, 3) g1(tz, r, ren);
  // 10.3.4 strengths
  wire sw;
  assign (strong1, weak0) sw = 1'b0;
  assign (weak1, strong0) sw = 1'b1;
  wire sw2;
  assign (pull1, pull0) sw2 = 1'b1;
  assign (weak1, weak0) sw2 = 1'b0;
  wire sw3;
  assign (highz1, strong0) sw3 = 1'b1;
  // 10.6 assign/deassign, 10.6.2 force/release
  logic [3:0] pv;
  wire [3:0] fnet;
  assign fnet = a;
  logic [7:0] fv;
  wire [7:0] fwb;
  assign fwb = {a, b};
  // 10.4 procedural
  logic [7:0] m1, m2;
  int arr[4];
  // 10.10 unpacked array concat
  int ua[];
  int fx[3];
  initial begin
    a = 5; #1 $display("T|10.3.3a|wd=%h wn=%h", wd, wn);
    #2 $display("T|10.3.3b|wd=%h wn=%h", wd, wn);
    #1 $display("T|10.3.3c|wd=%h", wd);
    p = 1; #2 p = 0; #5 $display("T|10.3.3d|pd=%b (pulse filtered)", pd);
    r = 1; #3 $display("T|10.3.3e|rf=%b", rf);  // rise 2
    r = 0; #3 $display("T|10.3.3f|rf=%b", rf);  // fall 5: still 1
    #3 $display("T|10.3.3g|rf=%b", rf);
    ren = 0; #2 $display("T|10.3.3h|tz=%b", tz); #2 $display("T|10.3.3i|tz=%b", tz);
    $display("T|10.3.4a|sw=%b %v", sw, sw);
    $display("T|10.3.4b|sw2=%b %v sw3=%b %v", sw2, sw2, sw3, sw3);
    // 10.4.1 blocking, 10.4.2 NBA swap
    m1 = 1; m2 = 2;
    m1 <= m2; m2 <= m1; #0 $display("T|10.4.2a|%0d %0d", m1, m2); #1 $display("T|10.4.2b|%0d %0d", m1, m2);
    // NBA to array element with index evaluated at schedule time
    begin int idx = 0; arr = '{0,0,0,0}; arr[idx] <= 7; idx = 2; arr[idx] <= 9; #1 $display("T|10.4.2c|%p", arr); end
    // 10.6.1 assign/deassign
    pv = 1; assign pv = 4'hc; #1 pv = 3; #1 $display("T|10.6.1a|pv=%h", pv);
    deassign pv; #1 $display("T|10.6.1b|pv=%h", pv); pv = 2; #1 $display("T|10.6.1c|pv=%h", pv);
    // 10.6.2 force/release on variable and net
    fv = 8'h11; force fv = 8'hff; fv = 8'h22; #1 $display("T|10.6.2a|fv=%h", fv);
    release fv; #1 $display("T|10.6.2b|fv=%h", fv);   // variable keeps forced value until next assign
    force fnet = 4'h9; #1 $display("T|10.6.2c|fnet=%h", fnet);
    a = 2; #1 $display("T|10.6.2d|fnet=%h", fnet);
    release fnet; #1 $display("T|10.6.2e|fnet=%h", fnet);   // net returns to driver
    // force on bit-select of net and variable
    force fwb[0] = 1'b1; force fwb[7:6] = 2'b00; a = 4'hf; b = 4'h0; #1 $display("T|10.6.2f|fwb=%b", fwb);
    release fwb[0]; #1 $display("T|10.6.2g|fwb=%b", fwb);
    release fwb[7:6]; #1 $display("T|10.6.2h|fwb=%b", fwb);
    // force with expression tracks RHS
    force fv = {4'h0, a}; a = 4'h3; #1 $display("T|10.6.2l|fv=%h", fv); a = 4'h4; #1 $display("T|10.6.2m|fv=%h", fv); release fv;
    // 10.9 assignment patterns
    begin int pp[4]; pp = '{default: 5}; $display("T|10.9.1a|%p", pp);
      pp = '{1: 10, 3: 30, default: 0}; $display("T|10.9.1b|%p", pp);
      pp = '{4{8}}; $display("T|10.9.1c|%p", pp);
      pp = '{0:1, 1:2, 2:3, 3:4}; $display("T|10.9.1d|%p", pp);
      begin logic [7:0] pk; pk = {1'b1, 1'b0, 6'h3f}; $display("T|10.9.1e|%b", pk); end
      begin logic [3:0] pk2 [2]; pk2 = '{default: '1}; $display("T|10.9.1f|%p", pk2); end
      begin typedef struct {int x; int y[2];} s_t; s_t s; s = '{x: 1, y: '{2, 3}}; $display("T|10.9.2|%p", s);
        s = '{default: 4}; $display("T|10.9.2b|%p", s); end
      begin int m2d[2][2] = '{default: 7}; $display("T|10.9.1g|%p", m2d); end
    end
    // 10.10 unpacked array concatenation
    fx = '{1,2,3};
    ua = {fx, 4, 5}; $display("T|10.10a|%p", ua);
    ua = {ua, ua}; $display("T|10.10b|%p", ua.size());
    begin int q1[$] = {1}; q1 = {q1, q1, 2}; $display("T|10.10c|%p", q1); end
    begin int z[$]; z = {}; $display("T|10.10d|%0d", z.size()); end
    // 10.11 net aliasing (alias)
    // 10.8 assignment-like contexts: truncation and extension
    begin logic [3:0] t4; logic [7:0] t8; logic signed [3:0] s4 = -2;
      t8 = s4; $display("T|10.8a|%b", t8);
      t4 = 8'hab; $display("T|10.8b|%h", t4);
      t8 = 4'sb1000; $display("T|10.8c|%b", t8);
    end
    // compound assignment ops
    begin int c = 10; c += 5; c -= 1; c *= 2; c /= 4; c %= 4; c <<= 3; c >>= 1; c |= 1; c &= 7; c ^= 2; $display("T|10.4.cmp|%0d", c);
      c = -16; c >>>= 2; $display("T|10.4.cmp2|%0d", c);
      c = 5; c <<<= 1; $display("T|10.4.cmp3|%0d", c); end
  end
  // 10.11 alias
  wire [3:0] al1, al2;
  alias al1 = al2;
  assign al2 = 4'h6;
  initial #1 $display("T|10.11|al1=%h", al1);
endmodule
