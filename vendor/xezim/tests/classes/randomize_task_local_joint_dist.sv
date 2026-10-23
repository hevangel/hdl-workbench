module p297;
  typedef logic [31:0] u32_t;
  typedef logic [3:0] u4_t;
  u32_t nowr [$];
  u32_t fly [$];
  int ones, total, nsum, fails;
  task automatic t4();
    bit bc [];
    u32_t addr [];
    u4_t host [];
    u4_t n;
    if (std::randomize(n, bc, addr, host) with {
          n >= 4; addr.size() == n; bc.size() == n; host.size() == n;
          foreach (host[i]) { host[i] < 2; }
          foreach (bc[i]) { bc[i] dist { 1 := 2, 0 := 1 }; !(addr[i] inside { nowr }); addr[i] inside { fly }; }
        }) begin
      nsum += n;
      foreach (bc[i]) begin ones += bc[i]; total++;
        if (!(addr[i] inside {fly}) || (addr[i] inside {nowr}) || host[i] >= 2) fails++; end
      if (addr.size() != n || bc.size() != n || n < 4) fails++;
    end else fails++;
  endtask
  initial begin
    nowr = '{32'h180, 32'h5004};
    fly = '{32'h5000, 32'h5004, 32'h5008};
    repeat (300) t4();
    $display("T|fails=%0d ones_pct=%0d avg_n_x10=%0d", fails, 100 * ones / total, nsum * 10 / 300);
  end
endmodule
