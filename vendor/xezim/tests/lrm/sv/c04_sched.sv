// top: c04
module c04;
  reg a, b, c;
  int d;
  reg [3:0] q;
  wire w;
  assign w = a;
  // 4.4.2.2 active vs inactive (#0) vs NBA
  initial begin
    a = 0; d = 0;
    a <= 1;
    #0 $display("T|4.4.2.3|after#0 a=%b", a);  // inactive before NBA: a still 0
    #0 $display("T|4.4.2.3b|after2#0 a=%b", a);
  end
  // $strobe in postponed sees final values
  initial begin
    #5 b = 0; b <= 1; $strobe("T|4.4.2.9|strobe b=%b", b); $display("T|4.4.2.9b|display b=%b", b);
    b = 0; // later blocking in same step still overridden by NBA
  end
  // NBA ordering same process (4.6/10.4.2): last NBA wins
  initial begin
    #10 c = 0; c <= 0; c <= 1; #1 $display("T|4.6|c=%b", c);
  end
  // monitor: fires once per timestep in postponed
  initial begin
    #20 q = 0;
    $monitor("T|4.4.2.9m|t=%0t q=%0d", $time, q);
    #1 q = 1; q = 2; q = 3;
    #1 q <= 4; q <= 5;
    #1 $monitoroff; q = 6;
    #1 $monitoron;
    #1 q = 7;
    #1 $monitoroff;
  end
  // Determinism 4.6: statements in a begin-end execute in order
  int seq[$];
  initial begin
    #30 seq.push_back(1); seq.push_back(2); #0 seq.push_back(3);
    $display("T|4.6s|%p", seq);
  end
  // continuous assign updated before #0 continuation? (both active; order nondeterministic) -- check after #1
  initial begin #40 a = 1; #1 $display("T|4.5|w=%b", w); a = 0; #0 #0 $display("T|4.5b|w=%b", w); end
  // NBA to same var from two always at same time; later events
  reg r1;
  initial begin #50 r1 <= 0; r1 <= #1 1; #2 $display("T|4.6d|r1=%b", r1); end
  // event from NBA region triggers active process re-eval
  reg clk = 0; reg [1:0] s1, s2;
  always @(posedge clk) s1 <= s1 + 1;
  always @(posedge clk) s2 <= s1;   // classic race-free shift
  initial begin s1 = 0; s2 = 0; #60 clk = 1; #1 clk = 0; #1 clk = 1; #1 $display("T|4.9nba|s1=%0d s2=%0d", s1, s2); end
  // $display at time 0 vs strobe ordering at the end
  final $display("T|9.2.3|final t=%0t", $time);
endmodule
