// top: c09
module c09;
  logic clk = 0;
  logic [3:0] a = 0, b = 0, y_comb, y_star, y_latch, y_ff;
  logic en = 0;
  int cnt_comb = 0;
  // 9.2.2.2 always_comb runs at time 0 and on changes incl. in called function
  function automatic logic [3:0] addb(input logic [3:0] x); return x + b; endfunction
  always_comb begin y_comb = addb(a); cnt_comb++; end
  // 9.2.2.2.1 @* does not include function-internal reads
  always @* y_star = addb(a);
  always_latch if (en) y_latch = a;
  always_ff @(posedge clk) y_ff <= a;
  initial forever #5 clk = ~clk;
  // 9.3 fork/join
  task automatic tdelay(int d, string nm, ref string log[$]); #d log.push_back(nm); endtask
  string lg[$];
  // 9.4.2 event control with iff
  logic [3:0] ev_cnt = 0;
  always @(posedge clk iff en) ev_cnt++;
  // edge on multi-bit: lsb only for posedge
  logic [3:0] mb = 0; int mb_pos = 0;
  always @(posedge mb) mb_pos++;
  // 9.4.2 named events and sequence
  event ev;
  // 9.4.5 intra-assignment
  logic [3:0] ia, ib;
  int ord[$];
  initial begin
    #1;
    $display("T|9.2.2.2a|y_comb=%0d y_star=%0d cnt=%0d", y_comb, y_star, cnt_comb);
    b = 3; #1;
    $display("T|9.2.2.2b|y_comb=%0d y_star=%0d", y_comb, y_star);   // @* misses b
    a = 1; #1;
    $display("T|9.2.2.2c|y_comb=%0d y_star=%0d", y_comb, y_star);
    en = 1; a = 7; #1 $display("T|9.2.2.3|latch=%0d", y_latch);
    en = 0; a = 8; #1 $display("T|9.2.2.3b|latch=%0d", y_latch);
    @(posedge clk); #1 $display("T|9.2.2.4|ff=%0d", y_ff);
    // fork variants
    lg = {};
    fork
      tdelay(3, "A", lg);
      tdelay(1, "B", lg);
    join
    $display("T|9.3.1a|%p t=%0t", lg, $time);
    lg = {};
    fork
      tdelay(3, "C", lg);
      tdelay(1, "D", lg);
    join_any
    lg.push_back("after_any");
    $display("T|9.3.1b|%p", lg);
    wait fork;
    $display("T|9.6.1|%p", lg);
    lg = {};
    fork
      tdelay(2, "E", lg);
    join_none
    lg.push_back("after_none");
    #0 $display("T|9.3.1c|%p", lg);
    #3 $display("T|9.3.1d|%p", lg);
    // join_none with loop var capture (automatic)
    lg = {};
    for (int i = 0; i < 3; i++) begin
      automatic int k = i;
      fork
        begin #1 lg.push_back($sformatf("k%0d", k)); end
      join_none
    end
    wait fork;
    lg.sort();
    $display("T|9.3.2|%p", lg);
    // disable fork
    lg = {};
    fork
      tdelay(5, "F", lg);
      tdelay(1, "G", lg);
    join_any
    disable fork;
    #10 $display("T|9.6.3|%p", lg);
    // disable named block
    begin : blk
      lg = {};
      fork
        begin #1 lg.push_back("H"); disable blk; end
        begin #3 lg.push_back("I"); end
      join
      lg.push_back("not_here");
    end
    #5 $display("T|9.6.2|%p", lg);
    // disable task
    // 9.4.1 delay control
    begin time t0; t0 = $time; #(2.4) ; $display("T|9.4.1a|%0t", $time - t0); end
    begin time t0; t0 = $time; #(1'bx) ; $display("T|9.4.1b|%0t", $time - t0); end
    // 9.4.2 iff
    en = 1; repeat (2) @(posedge clk); en = 0; repeat (2) @(posedge clk);
    $display("T|9.4.2a|ev_cnt=%0d", ev_cnt);
    // posedge of vector: lsb
    mb = 4'b0010; #1 mb = 4'b0011; #1 mb = 4'b0010; #1 mb = 4'b0x00; #1 mb = 4'b0001; #1;
    $display("T|9.4.2b|mb_pos=%0d", mb_pos);
    // 9.4.2.1 event OR, comma
    fork
      begin @(ev or posedge en) ord.push_back(1); end
      begin @(ev, b) ord.push_back(2); end
      #1 -> ev;
    join
    $display("T|9.4.2c|%p", ord);
    // 9.4.2.4 edge event: 'edge'
    begin int ecount = 0; fork
        repeat (2) begin @(edge clk) ecount++; end
      join
      $display("T|9.4.2d|edge ok %0d", ecount); end
    // 9.4.3 level-sensitive wait
    begin bit flag = 0; fork
        begin wait (flag) $display("T|9.4.3|woke t=%0t", $time % 100); end
        begin #7 flag = 1; end
      join end
    // 9.4.5 intra-assignment delays
    ia = 1; ib = #2 ia + 1; $display("T|9.4.5a|ib=%0d", ib);
    ia = 3; ib <= #2 ia; ia = 5; #1 $display("T|9.4.5b|ib=%0d", ib); #2 $display("T|9.4.5c|ib=%0d", ib);
    ib = @(posedge clk) ia; $display("T|9.4.5d|ib=%0d", ib);
    ib <= repeat (2) @(posedge clk) 4'd9; #0 $display("T|9.4.5e|ib=%0d", ib);
    repeat (3) @(posedge clk); $display("T|9.4.5f|ib=%0d", ib);
    // 9.7 process class
    begin process p1, p2; int st;
      fork
        begin p1 = process::self(); #20; end
        begin p2 = process::self(); #2; end
      join_none
      #1;
      $display("T|9.7a|%s %s", p1.status().name(), p2.status().name());
      p1.suspend(); #0 $display("T|9.7b|%s", p1.status().name());
      p1.resume(); #2;
      $display("T|9.7c|%s", p2.status().name());
      p1.kill(); #0 $display("T|9.7d|%s", p1.status().name());
      begin process me; me = process::self(); $display("T|9.7e|%s", me.status().name()); end
      fork begin p1 = process::self(); #5; end join_none
      #1 p1.await(); $display("T|9.7f|await %s", p1.status().name());
    end
    $finish;
  end
  // 9.2.3 final
  final $display("T|9.2.3|final");
endmodule
