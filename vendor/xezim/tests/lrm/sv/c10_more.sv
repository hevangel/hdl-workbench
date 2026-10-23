// top: c10x
module c10x;
  // 10.3.2 continuous assignment to variable
  logic [3:0] cv;
  logic [3:0] src = 4'h3;
  assign cv = src + 1;
  // 10.3 net with drive strengths and wired logic resolution
  wire w1;
  assign (strong1, strong0) w1 = 1'b1;
  assign (pull1, pull0) w1 = 1'b0;      // strong wins -> 1
  wire w2;
  assign (weak1, weak0) w2 = 1'b1;
  assign (weak1, weak0) w2 = 1'b0;      // equal strength conflict -> x
  wire w3;
  assign w3 = 1'bz;
  assign (pull1, pull0) w3 = 1'b0;      // z vs pull0 -> 0
  // tri-state bus with multiple drivers
  logic en1 = 0, en2 = 0;
  wire [7:0] bus;
  assign bus = en1 ? 8'h11 : 8'hzz;
  assign bus = en2 ? 8'h22 : 8'hzz;
  // 10.3.1 net declaration assignment with delay
  logic dsrc = 0;
  wire #5 wdl = dsrc;
  // continuous assignment of struct and unpacked array to net/var
  typedef struct packed {logic [3:0] a; logic [3:0] b;} ps_t;
  ps_t pss;
  wire ps_t psn;
  assign psn = '{a: 4'h1, b: src};
  assign pss = psn;
  // assign to part of vector from multiple continuous assigns
  wire [7:0] split;
  assign split[3:0] = 4'ha;
  assign split[7:4] = src;
  // 10.6.2 force on whole vector net then release while driver changes
  wire [3:0] fw;
  assign fw = src;
  initial begin
    #1;
    $display("T|10.3.2|cv=%h", cv);
    $display("T|10.3.4c|w1=%b %v w2=%b %v w3=%b %v", w1, w1, w2, w2, w3, w3);
    $display("T|10.3.4d|bus=%h", bus);
    en1 = 1; #1 $display("T|10.3.4e|bus=%h", bus);
    en2 = 1; #1 $display("T|10.3.4f|bus=%b", bus);
    en1 = 0; #1 $display("T|10.3.4g|bus=%h", bus);
    dsrc = 1; #4 $display("T|10.3.1a|wdl=%b", wdl); #1 $display("T|10.3.1b|wdl=%b", wdl);
    src = 4'h7; #1 $display("T|10.3.2b|cv=%h psn=%h pss=%h split=%h", cv, psn, pss, split);
    force fw = 4'hf; src = 4'h1; #1 $display("T|10.6.2n|fw=%h", fw);
    release fw; #0 $display("T|10.6.2o|fw=%h", fw);
    // force of a variable driven by continuous assign
    force cv = 4'h0; #1 $display("T|10.6.2p|cv=%h", cv);
    release cv; #1 $display("T|10.6.2q|cv=%h", cv);    // re-evaluated by continuous assign
    // force of variable, then procedural assign while forced, then release
    // force hierarchical
    force sub.sv = 8'h55; #1 $display("T|10.6.2r|%h", sub.sv);
    release sub.sv; #1 $display("T|10.6.2s|%h", sub.sv);
  end
  c10sub sub();
endmodule
module c10sub; logic [7:0] sv = 8'h11; endmodule
