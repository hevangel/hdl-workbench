module top;
  class shared_base;
    static logic [8:1] offset_bits=0;
    static logic [0:7] ascending_bits=0;
    static bit [7:0] known_bits=0;
    static logic [1:0][3:0] lanes=0;
    static logic [7:0] shadow=8'ha5;
    static logic [7:0] fixed_values[2];
    static byte unsigned queued_values[$];
    static logic [7:0] dynamic_values[];
    static int keyed_values[int];
    function void set_queue();
      queued_values.push_back(8'h12); queued_values[0]=8'h34;
    endfunction
    function int local_shadow();
      byte unsigned shadow[$]; shadow.push_back(8'h11);
      shadow[0]=8'h22; return shadow[0];
    endfunction
    function void set_offset(); offset_bits[4:1]=4'hb; endfunction
    function void set_ascending(); ascending_bits[1 +: 3]=3'b101; endfunction
    function void set_known(); known_bits[3:0]=4'bx1z0; endfunction
    function void set_lane(); lanes[1][2:1]=2'b11; endfunction
  endclass
  class inherited extends shared_base; endclass
  int errors;
  task automatic ck(string label,logic [31:0] got,logic [31:0] expected);
    if(got===expected) $display("PASS %s",label);
    else begin errors++; $display("FAIL %s got=%h expected=%h",label,got,expected); end
  endtask
  initial begin
    shared_base a,b;
    inherited d;
    a=new(); b=new(); d=new();
    a.set_offset(); ck("nonzero range",shared_base::offset_bits,8'h0b);
    ck("shared through second object",b.offset_bits,8'h0b);
    a.set_ascending(); ck("ascending indexed",shared_base::ascending_bits,8'h50);
    shared_base::offset_bits[8 -: 2]=2'b11;
    ck("scoped indexed",shared_base::offset_bits,8'hcb);
    d.offset_bits[2]=1'b0;
    ck("inherited instance select",shared_base::offset_bits,8'hc9);
    a.set_known(); ck("two-state selected store",shared_base::known_bits,8'h04);
    a.set_lane(); ck("nested static dimensions",shared_base::lanes,8'h60);
    ck("local queue shadows static",a.local_shadow(),8'h22);
    ck("shadowed static unchanged",shared_base::shadow,8'ha5);
    ck("static fixed default",shared_base::fixed_values[0],8'hxx);
    a.fixed_values[1]=8'hab;
    ck("static fixed shared",b.fixed_values[1],8'hab);
    a.set_queue();
    ck("static queue element",shared_base::queued_values[0],8'h34);
    shared_base::dynamic_values=new[2]; a.dynamic_values[1]=8'h56;
    ck("static dynamic element",b.dynamic_values[1],8'h56);
    a.keyed_values[3]=19;
    ck("static associative element",shared_base::keyed_values[3],19);
    if(errors==0) $display("ALL PASS");
    $finish;
  end
endmodule
