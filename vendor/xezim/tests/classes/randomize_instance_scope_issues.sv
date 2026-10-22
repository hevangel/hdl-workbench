// Issues #303, #304 and #305: std::randomize() with an inline constraint
// block inside a module instance.
module fixed_kid;
  int period;
  bit [15:0] delays [16];
  initial begin
    period = 1000;
    void'(std::randomize(delays) with {
      foreach (delays[i]) {
        delays[i] >= 0.9 * period;
        delays[i] <= 1.1 * period;
      }
    });
    if (delays[0] >= 900 && delays[0] <= 1100 && delays[15] >= 900 && delays[15] <= 1100)
      $display("T|303 PASS");
    else
      $display("T|303 FAIL first=%0d", delays[0]);
  end
endmodule

module presized_kid;
  int freq_a, freq_b, period;
  bit [15:0] delays [];
  initial begin
    int ok;
    freq_a = 1000;
    freq_b = freq_a;
    period = 1000000 / freq_b;
    delays = new[16];
    ok = std::randomize(delays) with {
      delays.size() == 16;
      foreach (delays[i]) {
        delays[i] >= 0.9 * period;
        delays[i] <= 1.1 * period;
      }
    };
    if (ok && delays.size() == 16 && delays[0] >= 900 && delays[0] <= 1100)
      $display("T|304 PASS size=%0d", delays.size());
    else
      $display("T|304 FAIL ok=%0d size=%0d first=%0d", ok, delays.size(), delays[0]);
  end
endmodule

module leak_kid;
  int period;
  bit [15:0] delays [];
  initial begin
    period = 1000;
    void'(std::randomize(delays) with {
      delays.size() == 16;
      foreach (delays[i]) {
        delays[i] >= 0.9 * period;
        delays[i] <= 1.1 * period;
      }
    });
    $display("T|305 child size=%0d inrange=%0d", delays.size(),
             delays.size() > 0 && delays[0] >= 900 && delays[0] <= 1100);
  end
endmodule

module top;
  bit [15:0] delays [];
  fixed_kid u_fixed();
  presized_kid u_presized();
  leak_kid u_leak();
  initial begin
    #1;
    $display("T|305 top size=%0d", delays.size());
  end
endmodule
