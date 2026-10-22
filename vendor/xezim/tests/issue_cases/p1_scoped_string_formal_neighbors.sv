package text_scopes;
  typedef string base_t;
  typedef base_t chain_t;
endpackage
package bits_scopes;
  typedef logic [31:0] base_t;
endpackage
module top;
  typedef int base_t;
  typedef int chain_t;
  string text;
  logic [31:0] packed_value;
  task automatic text_change(inout text_scopes::chain_t s);
    s[0]="S"; s={s,"!"};
  endtask
  task automatic bits_change(inout bits_scopes::base_t s);
    s=32'h12345678; s[0]=1'b1;
  endtask
  initial begin
    text="abc"; text_change(text); bits_change(packed_value);
    $display("T|scoped-chain %s",text);
    $display("T|scoped-packed %h",packed_value);
    $finish;
  end
endmodule
