module n295;
  // §7.12.3 neighbours: reduction width/sign, with and without `with`
  byte sb [] = '{-1, -2, 100, 27};      // sum -1-2+100+27 = 124; 8-bit
  byte sb2 [] = '{100, 100};            // 200 -> 8-bit signed -56
  bit [7:0] ub [] = '{200, 100};        // 300 -> 44
  bit [15:0] u16 [] = '{65535, 2};      // 1
  int  si [] = '{-5, 3};
  shortint ss [] = '{3, 5, 7};          // product 105
  bit [3:0] n4 [] = '{4'hF, 4'h3};      // product 45 -> 13 (4-bit)
  int r;
  initial begin
    $display("T|a %0d %0d %0d", sb.sum(), sb2.sum(), sb2.sum() with (int'(item)));
    $display("T|b %0d %0d", ub.sum(), ub.sum() with (16'(item)));
    $display("T|c %0d %0d", u16.sum(), si.sum());
    $display("T|d %0d %0d %0d", sb2.sum() == -56, sb2.sum() < 0, (sb2.sum() with (item)) == -56);
    $display("T|e %0d %0d", ss.product(), n4.product());
    $display("T|f %0d %0d %0d", n4.and(), n4.or(), n4.xor());
    $display("T|g %0d %0d", $bits(sb.sum()), $bits(ub.sum() with (16'(item))));
    $display("T|h %0d %0d", ub.sum() + 0, ub.sum() + 16'd0);
    $display("T|i %0d", sb2.sum() with (item > 50));
    $display("T|j %0d %0d", n4.and() with (item | 4'h8), n4.xor() with (item + 1));
    $display("T|k %0d", sb2.product());
  end
endmodule
