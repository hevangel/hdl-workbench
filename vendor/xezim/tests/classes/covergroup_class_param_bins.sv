module top;
  parameter int M = 3;
  class group_n #(int N = 4);
    covergroup cg with function sample(int i);
      option.per_instance = 1;
      cp: coverpoint i { bins b[] = {[0:N-1]}; }
    endgroup
    function new(); cg = new(); endfunction
    function void sample(int i); cg.sample(i); endfunction
    function real coverage(); return cg.get_inst_coverage(); endfunction
  endclass
  class shapes #(int N = 2, int K = 1);
    covergroup cg with function sample(int i);
      option.per_instance = 1;
      one: coverpoint i { bins b = {N}; bins z = {0}; }
      rng: coverpoint i { bins r = {[N:N*2]}; bins lo = {[0:N-1]}; }
      ign: coverpoint i { bins b[] = {[0:N+1]}; ignore_bins ig = {N}; }
      ill: coverpoint i { bins b[] = {[0:3]}; illegal_bins il = {N*10}; }
      mix: coverpoint i { bins b[] = {[K:N+M]}; }
    endgroup
    function new(); cg = new(); endfunction
    function void sample(int i); cg.sample(i); endfunction
    function void report(string tag);
      $display("T|%s one=%0.2f rng=%0.2f ign=%0.2f ill=%0.2f mix=%0.2f inst=%0.2f", tag,
        cg.one.get_inst_coverage(), cg.rng.get_inst_coverage(), cg.ign.get_inst_coverage(),
        cg.ill.get_inst_coverage(), cg.mix.get_inst_coverage(), cg.get_inst_coverage());
    endfunction
  endclass
  group_n #(2) g;
  group_n g4;
  group_n #(5) g5;
  shapes s_def;
  shapes #(3) s3;
  shapes #(4, 2) s42;
  initial begin
    g = new(); g.sample(0);
    $display("T|COVERAGE=%0.2f", g.coverage());
    g4 = new(); g4.sample(0); g4.sample(3);
    $display("T|g4=%0.2f", g4.coverage());
    g5 = new(); g5.sample(4); g5.sample(5);
    $display("T|g5=%0.2f", g5.coverage());
    g.sample(1);
    $display("T|g2_full=%0.2f", g.coverage());
    s_def = new(); s3 = new(); s42 = new();
    s_def.sample(2); s_def.sample(0); s_def.sample(3);
    s_def.report("def");
    s3.sample(3); s3.sample(6); s3.sample(1);
    s3.report("s3");
    s42.sample(4); s42.sample(8); s42.sample(2); s42.sample(5);
    s42.report("s42");
    $display("T|done");
  end
endmodule
