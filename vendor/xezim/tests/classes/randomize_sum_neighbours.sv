module q295;
  bit [15:0] u16 [];
  byte sb [];
  shortint ss [];
  bit [3:0] n4 [];
  int r, ok;
  function automatic longint rawsum16(); longint t = 0; foreach (u16[i]) t += u16[i]; return t; endfunction
  initial begin
    // sum with size tied to a scalar
    begin
      int n;
      r = std::randomize(n, u16) with { n inside {[2:6]}; u16.size() == n; u16.sum() == 100; };
      $display("T|a r=%0d ok=%0d", r, r && u16.size() == n && u16.sum() == 100);
    end
    r = std::randomize(u16) with { u16.size() inside {[3:5]}; u16.sum() with (int'(item)) == 70000; };
    $display("T|b r=%0d ok=%0d", r, r && (u16.sum() with (int'(item))) == 70000);
    r = std::randomize(u16) with { u16.size() == 3; u16.sum() with (int'(item)) == 300000; };
    $display("T|c r=%0d", r);
    r = std::randomize(u16) with { u16.size() == 3; foreach (u16[i]) u16[i] > 30000; u16.sum() == 5; };
    $display("T|d r=%0d ok=%0d", r, r && u16.sum() == 5 && u16[0] > 30000 && u16[1] > 30000 && u16[2] > 30000);
    r = std::randomize(sb) with { sb.size() == 4; sb.sum() == -100; };
    $display("T|e r=%0d ok=%0d", r, r && sb.sum() == -100);
    r = std::randomize(sb) with { sb.size() == 4; sb.sum() < -120; };
    $display("T|f r=%0d ok=%0d", r, r && sb.sum() < -120);
    r = std::randomize(ss) with { ss.size() == 3; ss.sum() with (int'(item)) == -70000; };
    $display("T|g r=%0d ok=%0d", r, r && (ss.sum() with (int'(item))) == -70000);
    r = std::randomize(n4) with { n4.size() == 3; n4.product() == 6; };
    $display("T|h r=%0d ok=%0d", r, r && n4.product() == 6);
    r = std::randomize(n4) with { n4.size() == 3; n4.xor() == 4'hA; };
    $display("T|i r=%0d ok=%0d", r, r && n4.xor() == 4'hA);
    r = std::randomize(n4) with { n4.size() == 3; n4.and() == 4'h1; n4.or() == 4'h3; };
    $display("T|j r=%0d ok=%0d", r, r && n4.and() == 1 && n4.or() == 3);
    r = std::randomize(n4) with { n4.size() == 4; n4.sum() with (item > 7) == 1; };
    $display("T|k r=%0d ok=%0d", r, r && (n4.sum() with (item > 7)) == 1);
    r = std::randomize(n4) with { n4.size() == 4; n4.sum() with (int'(item > 7)) == 3; };
    $display("T|l r=%0d ok=%0d", r, r && (n4.sum() with (int'(item > 7))) == 3);
    r = std::randomize(u16) with { u16.size() == 4; u16.sum() >= 10; u16.sum() <= 20; };
    $display("T|m r=%0d ok=%0d", r, r && u16.sum() >= 10 && u16.sum() <= 20);
    r = std::randomize(u16) with { u16.size() == 2; u16.sum() inside {[1:2], 65535}; };
    $display("T|n r=%0d ok=%0d", r, r && (u16.sum() inside {[1:2], 65535}));
    r = std::randomize(sb) with { sb.size() == 3; sb.sum() with (item * 2) == 6; };
    $display("T|o r=%0d ok=%0d", r, r && (sb.sum() with (item * 2)) == 6);
    r = std::randomize(u16) with { u16.size() == 4; u16.sum() == 3; u16.sum() != 3; };
    $display("T|p r=%0d", r);
  end
endmodule
