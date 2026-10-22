// top: c19_more
class Tr;
  bit [2:0] k; bit [1:0] m;
  covergroup cg;
    option.per_instance = 1;
    ck : coverpoint k;
    cm : coverpoint m { bins m0 = {0}; bins m1 = {1}; }
    kxm : cross ck, cm;
  endgroup
  function new(); cg = new; endfunction
endclass
module c19_more;
  Tr t1, t2; int hits; real c;
  covergroup cg_g with function sample(int x);
    option.goal = 50;
    cp : coverpoint x { bins b[4] = {[0:7]}; }
  endgroup
  covergroup cg_sv (ref int r) ;
    type_option.weight = 2;
    cp : coverpoint r { bins z = {0}; bins nz = {[1:$]}; }
  endgroup
  cg_g g1, g2; int rv; cg_sv s1;
  initial begin
    t1 = new; t2 = new;
    for (int i = 0; i < 8; i++) begin t1.k = i; t1.m = i % 2; t1.cg.sample(); end
    t2.k = 1; t2.m = 0; t2.cg.sample();
    $display("T|19.4.1|class cg t1 inst=%0.2f t2 inst=%0.2f type=%0.2f", t1.cg.get_inst_coverage(), t2.cg.get_inst_coverage(), t1.cg.get_coverage());
    $display("T|19.8|ck t1=%0.2f kxm t1=%0.2f", t1.cg.ck.get_inst_coverage(), t1.cg.kxm.get_inst_coverage());
    g1 = new; g2 = new; g1.sample(0); g1.sample(3); g2.sample(7);
    $display("T|19.8|type cov over insts=%0.2f g1=%0.2f g2=%0.2f", cg_g::get_coverage(), g1.get_inst_coverage(), g2.get_inst_coverage());
    begin int cov_n, tot_n; c = g1.get_coverage(cov_n, tot_n); $display("T|19.8|get_coverage(ref) c=%0.2f %0d/%0d", c, cov_n, tot_n); end
    s1 = new(rv); rv = 0; s1.sample(); rv = 5; s1.sample(); $display("T|19.3|ref arg cov=%0.2f", s1.get_coverage());
    g1.set_inst_name("myinst"); $display("T|19.8|set_inst_name ok");
  end
endmodule
