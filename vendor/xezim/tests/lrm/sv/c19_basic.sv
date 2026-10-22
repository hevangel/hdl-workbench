// top: c19_basic
module c19_basic;
  bit clk; bit [3:0] a; bit [1:0] b; bit en; int i;
  covergroup cg_auto @(posedge clk); coverpoint b; endgroup
  covergroup cg_bins @(posedge clk);
    option.per_instance = 1;
    cp_a : coverpoint a iff (en) {
      bins lo = {[0:3]};
      bins mid[] = {[4:7]};
      bins hi[2] = {[8:15]};
      bins odd = {1, 3, 5};
      ignore_bins ign = {12};
      illegal_bins ill = {15};
    }
    cp_b : coverpoint b { bins z = {0}; bins other = default; }
    cp_w : coverpoint a { wildcard bins w1 = {4'b1??1}; }
    cp_t : coverpoint a { bins t1 = (1 => 2); bins t2 = (2 => 3 => 4); bins t3 = (5 [*2]); bins t4 = (1, 2 => 6, 7); bins t5 = (8 [-> 2] => 9); bins t6 = (10 [= 2]); }
    x_ab : cross cp_a, cp_b;
  endgroup
  covergroup cg_x;
    cpa : coverpoint a { bins l = {[0:7]}; bins h = {[8:15]}; }
    cpb : coverpoint b;
    xc : cross cpa, cpb { ignore_bins ig = binsof(cpb) intersect {3}; bins hb = binsof(cpa.h) && binsof(cpb) intersect {[0:1]}; }
  endgroup
  covergroup cg_args (int lo, int hi) with function sample(bit [3:0] v);
    option.at_least = 2;
    cp : coverpoint v { bins r = {[lo:hi]}; bins rest = {[hi+1:15]}; }
  endgroup
  covergroup cg_opt;
    option.auto_bin_max = 4;
    cp : coverpoint a;
    cp2 : coverpoint b { option.weight = 0; }
  endgroup
  cg_auto ga = new; cg_bins gb = new; cg_bins gb2 = new; cg_x gx = new; cg_args gg = new(2, 5); cg_opt go = new;
  bit [3:0] seq [] = '{1,2,3,4,5,5,1,6,8,0,8,9,10,0,10,12,15,7,13,4};
  always #5 clk = ~clk;
  initial begin
    gb2.stop();
    for (i = 0; i < seq.size(); i++) begin
      @(negedge clk) a = seq[i]; b = i % 4; en = (i != 3);
      gx.sample(); go.sample(); gg.sample(a);
    end
    @(negedge clk);
    $display("T|19.4|auto b cov=%0.2f", ga.get_coverage());
    $display("T|19.5|bins cp_a=%0.2f cp_b=%0.2f cp_w=%0.2f cp_t=%0.2f x=%0.2f total=%0.2f inst=%0.2f", gb.cp_a.get_coverage(), gb.cp_b.get_coverage(), gb.cp_w.get_coverage(), gb.cp_t.get_coverage(), gb.x_ab.get_coverage(), gb.get_coverage(), gb.get_inst_coverage());
    $display("T|19.5|stopped inst=%0.2f", gb2.get_inst_coverage());
    $display("T|19.6|cross cpa=%0.2f cpb=%0.2f xc=%0.2f tot=%0.2f", gx.cpa.get_coverage(), gx.cpb.get_coverage(), gx.xc.get_coverage(), gx.get_coverage());
    $display("T|19.7|args at_least cp=%0.2f", gg.get_coverage());
    $display("T|19.7|auto_bin_max cp=%0.2f cp2=%0.2f tot=%0.2f", go.cp.get_coverage(), go.cp2.get_coverage(), go.get_coverage());
    $finish;
  end
endmodule
