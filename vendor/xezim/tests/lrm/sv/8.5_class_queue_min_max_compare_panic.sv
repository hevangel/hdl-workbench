// top: t7_12
class Arr; int a[$]; endclass
module t7_12;
  Arr ar;
  initial begin
    ar = new; ar.a = '{3, 1, 2};
    $display("T|b|cmp=%0d", ar.a.min() == ar.a.max());
  end
endmodule
