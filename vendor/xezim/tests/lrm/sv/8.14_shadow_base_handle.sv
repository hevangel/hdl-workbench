// top: t8_14
class Base; int v = 1; endclass
class Derived extends Base; int v = 2; endclass
module t8_14;
  Base b; Derived d;
  initial begin d = new; b = d; $display("T|a|d.v=%0d b.v=%0d", d.v, b.v); end
endmodule
