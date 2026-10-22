package text_kinds;
  typedef string text_t;
endpackage
module top;
  typedef string text_t;
  typedef text_t chained_text_t;
  string a,b,c,d,e;
  logic [31:0] packed_value;
  task automatic direct(inout string s); s[0]="A"; s={s,"!"}; endtask
  task automatic aliased(inout text_t s); #1; s[0]="B"; s={s,"!"}; endtask
  task automatic chained(inout chained_text_t s); s[0]="C"; s={s,"!"}; endtask
  task automatic scoped(inout text_kinds::text_t s); s[0]="D"; s={s,"!"}; endtask
  task automatic packed_control(output logic [31:0] s); #1; s=32'h12345678; s[0]=1'b1; endtask
  class TextBox #(type T=string);
    task automatic change(inout T s); s[0]="E"; s={s,"!"}; endtask
  endclass
  initial begin
    TextBox #(string) box;
    a="abc"; b="abc"; c="abc"; d="abc"; e="abc";
    box=new;
    direct(a);
    fork aliased(b); packed_control(packed_value); join
    chained(c); scoped(d); box.change(e);
    $display("T|direct %s",a);
    $display("T|alias %s",b);
    $display("T|chain %s",c);
    $display("T|scoped %s",d);
    $display("T|parameter %s",e);
    $display("T|packed %h",packed_value);
    $finish;
  end
endmodule
