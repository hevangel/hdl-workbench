// top: t17
class Leaf; string tag = "s"; int k = 4; endclass
class Hold; Leaf h; function new(); h = new; endfunction endclass
module t17;
  Hold n; string x;
  initial begin
    n = new;
    $display("T|a|two-level [%s] k=%0d", n.h.tag, n.h.k);
    x = {n.h.tag, "!"}; $display("T|b|concat [%s] eq=%0d", x, n.h.tag == "s");
  end
endmodule
