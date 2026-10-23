// top: c19_bins2
module c19_bins2;
  bit [3:0] v; bit [1:0] w; int i;
  bit [3:0] s [] = '{0,1,2,3,3,3,7,8,9,1,2,12,14,0,5};
  covergroup cg with function sample(bit [3:0] x, bit [1:0] y);
    option.per_instance = 1;
    cpw : coverpoint x { bins wc[] = {4'b00??}; wildcard bins ww = {4'b11?0}; }
    cpwith : coverpoint x { bins ev[] = {[0:15]} with (item % 4 == 0); }
    cpseq : coverpoint x { bins s1 = (0 => 1 => 2); bins s2 = (3 [*3]); bins s3 = (3 [*2:3] => 7); bins s4 = (1, 2 => [12:14]); bins s5[] = (8, 9 => 1, 2); bins dflt = default sequence; }
    cpfix : coverpoint x { bins f[3] = {[0:9]}; }    // uneven split
    cpig : coverpoint x { ignore_bins i = {[8:15]}; }   // auto bins minus ignored
    cpy : coverpoint y;
    xx : cross cpfix, cpy { bins sel = binsof(cpfix.f) intersect {[0:2]}; ignore_bins ig = binsof(cpy) intersect {3}; illegal_bins il = binsof(cpfix) intersect {9} && binsof(cpy) intersect {2}; }
    xw : cross cpig, cpy { bins wb = xw with (cpig < 2); }
  endgroup
  cg g = new;
  initial begin
    foreach (s[k]) g.sample(s[k], k % 4);
    $display("T|19.5.1|wildcard %0.2f", g.cpw.get_inst_coverage());
    $display("T|19.5.1|with %0.2f", g.cpwith.get_inst_coverage());
    $display("T|19.5.2|transitions %0.2f", g.cpseq.get_inst_coverage());
    $display("T|19.5|fixed split %0.2f", g.cpfix.get_inst_coverage());
    $display("T|19.5.5|ignore auto %0.2f", g.cpig.get_inst_coverage());
    $display("T|19.6|cross sel %0.2f", g.xx.get_inst_coverage());
    $display("T|19.6.1|cross with %0.2f", g.xw.get_inst_coverage());
    $display("T|19.5|total %0.2f", g.get_inst_coverage());
  end
endmodule
