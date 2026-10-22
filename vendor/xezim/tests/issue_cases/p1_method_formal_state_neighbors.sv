module top;
  typedef byte alias_byte;
  logic [7:0] unknown8=8'bxz110011;
  logic [31:0] unknown32={8{4'bxz01}};
  logic unknown1=1'bx;
  class FormalStates;
    function int bytes(input byte b);
      byte c=b;
      return b==8'h33 && c==8'h33;
    endfunction
    function int integers(input int n);
      int c=n;
      return n==32'h11111111 && c==32'h11111111;
    endfunction
    function int bits(input bit b);
      bit c=b;
      return b==0 && c==0;
    endfunction
    function int vectors(input bit [7:0] b);
      bit [7:0] c=b;
      return b==8'h33 && c==8'h33;
    endfunction
    function int aliases(input alias_byte b);
      byte c=b;
      return b==8'h33 && c==8'h33;
    endfunction
    function int direct_byte(input logic [7:0] b);
      return bytes(b);
    endfunction
    function logic [7:0] four_state(input logic [7:0] b);
      logic [7:0] c=b;
      return c;
    endfunction
  endclass
  initial begin
    FormalStates f;
    int b,n,t,v,a,d;
    logic [7:0] control;
    f=new;
    for(int i=0;i<1200;i++) begin
      b=f.bytes(unknown8); n=f.integers(unknown32); t=f.bits(unknown1);
      v=f.vectors(unknown8); a=f.aliases(unknown8); control=f.four_state(unknown8);
      d=f.direct_byte(unknown8);
    end
    $display("T|byte-state %0d",b);
    $display("T|int-state %0d",n);
    $display("T|bit-state %0d",t);
    $display("T|vector-state %0d",v);
    $display("T|alias-state %0d",a);
    $display("T|four-state %b",control);
    $display("T|direct-byte-state %0d",d);
    $finish;
  end
endmodule
