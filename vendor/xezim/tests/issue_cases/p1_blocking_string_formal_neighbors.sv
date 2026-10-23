`timescale 1ns/1ns
package delayed_text_types;
  typedef string text_t;
endpackage
module top;
  typedef string text_t;
  class DelayedBox #(type T=string);
    task automatic change(inout T s, input bit text_mode);
      #1;
      if (text_mode) begin s[0]="E"; s={s,"!"}; end
      else s[0]=1'b1;
      #1;
    endtask
    static task automatic change_static(inout T s, input bit text_mode);
      #1;
      if (text_mode) begin s[0]="S"; s={s,"!"}; end
      else s[0]=1'b0;
      #1;
    endtask
  endclass
  typedef DelayedBox#(string) StaticText;
  typedef DelayedBox#(int) StaticBits;
  task automatic direct(inout string s);
    #1; s[0]="A"; s={s,"!"}; #1;
  endtask
  task automatic aliased(inout delayed_text_types::text_t s);
    #1; s[0]="B"; s={s,"!"}; #1;
  endtask
  string a,b,c,d,e;
  int packed_value,static_value;
  initial begin
    DelayedBox#(string) text_box;
    DelayedBox#(int) bits_box;
    text_box=new; bits_box=new;
    a="abc"; b="abc"; c="abc"; d="abc"; e="xyz";
    packed_value=32'h12345678; static_value=32'h87654321;
    direct(a);
    aliased(b);
    fork
      text_box.change(c,1);
      bits_box.change(packed_value,0);
    join
    StaticText::change_static(d,1);
    StaticBits::change_static(static_value,0);
    text_box.change(e,1);
    $display("T|direct %s",a);
    $display("T|alias %s",b);
    $display("T|parameter-string %s",c);
    $display("T|parameter-int %h",packed_value);
    $display("T|static-string %s",d);
    $display("T|static-int %h",static_value);
    $display("T|parameter-string-again %s",e);
    $finish;
  end
endmodule
