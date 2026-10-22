// top: c16_imm
module c16_imm;
  int a, b, pass_n, fail_n; logic x;
  always_comb begin
    a0: assert #0 (a == b) else $display("T|16.4|deferred fail a=%0d b=%0d t=%0t", a, b, $time);
  end
  always_comb begin
    a1: assert final (a >= b) $display("T|16.4|final pass a=%0d t=%0t", a, $time); else $display("T|16.4|final fail a=%0d b=%0d", a, b);
  end
  initial begin
    x = 1'bx;
    s1: assert (1) $display("T|16.3|pass action"); else $display("T|16.3|BAD");
    s2: assert (0) $display("T|16.3|BAD"); else $display("T|16.3|fail action %m");
    s3: assert (x) else $display("T|16.3|x fails");
    s4: assert (1'bz) else $display("T|16.3|z fails");
    as1: assume (1 == 1) else $display("T|16.3|BAD");
    cv: cover (1) $display("T|16.3|cover pass");
    cv2: cover (0) $display("T|16.3|cover BAD");
    // glitch: deferred should report only final value in time step
    a = 1; b = 0; #0 b = 1; #0;
    #1 a = 5; b = 5; a = 6; b = 6;
    #1 a = 2; b = 3; #0 a = 3;
    #1;
    $display("T|16.3|default severity follows");
    sd: assert (0);
    #1 a = 9; b = 9;
    begin
      $assertoff(0, c16_imm.s5);
      s5: assert (0) else $display("T|20.12|BAD s5 not off");
      $asserton(0, c16_imm.s5);
      s5b: assert (0) else $display("T|20.12|s5b fires");
    end
    #1 $finish;
  end
  // failure action severities
  initial begin
    #10;
    f1: assert (0) else $info("info sev");
    f2: assert (0) else $warning("warn sev %0d", 1);
    f3: assert (0) else $error("err sev");
  end
endmodule
