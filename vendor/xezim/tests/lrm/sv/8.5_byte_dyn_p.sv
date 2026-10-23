// top: t8_5
class P; byte dyn[]; byte fa[2]; endclass
module t8_5;
  P p; byte md[]; 
  initial begin p = new; p.dyn = new[2]; p.dyn[1] = 8'hff; p.fa[1] = -2; md = new[1]; md[0] = -1;
    $display("T|a|cls dyn=%p fa=%p mod=%p el=%0d", p.dyn, p.fa, md, p.dyn[1]); end
endmodule
