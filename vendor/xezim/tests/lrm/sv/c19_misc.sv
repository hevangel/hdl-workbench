// top: c19_misc
module c19_misc;
  bit clk, en; bit [2:0] v;
  covergroup cge @(posedge clk iff en); coverpoint v; endgroup
  covergroup cgm; type_option.merge_instances = 1; cp : coverpoint v { bins b[] = {[0:3]}; } endgroup
  cge g = new; cgm m1 = new, m2 = new;
  always #5 clk = ~clk;
  initial begin
    for (int i = 0; i < 8; i++) begin @(negedge clk) v = i; en = i < 4; end
    @(negedge clk);
    $display("T|19.3|event iff cov=%0.2f", g.get_coverage());
    v = 0; m1.sample(); v = 1; m2.sample();
    $display("T|19.7.1|merge_instances type=%0.2f i1=%0.2f", cgm::get_coverage(), m1.get_inst_coverage());
    $display("T|19.9|$get_coverage=%0.2f", $get_coverage());
    $finish;
  end
endmodule
