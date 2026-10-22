// §6.8 / §8.9 / §8.23: aliases do not create another class declaration.
package init_count_pkg;
  int calls[7];
  function int mark(int slot); calls[slot]++; return calls[slot]; endfunction
  function string label_text(); void'(mark(3)); return "ready"; endfunction
endpackage
class registered_marker;
  function new(); void'(init_count_pkg::mark(4)); endfunction
endclass
class collision_helper;
  static int token = init_count_pkg::mark(6);
endclass
module single_unit(output int value, second_value);
  class one_helper;
    static int token = init_count_pkg::mark(0);
  endclass
  initial begin
    one_helper a = new, b = new;
    value=a.token; second_value=b.token;
  end
endmodule
module pair_unit(output int value);
  class pair_helper;
    static int token = init_count_pkg::mark(1);
  endclass
  initial begin pair_helper h=new; value=h.token; end
endmodule
module two_class_unit(output int sum);
  class left_helper; static int token=init_count_pkg::mark(2); endclass
  class right_helper; static int token=init_count_pkg::mark(2); endclass
  initial begin
    left_helper a=new; right_helper b=new;
    sum=a.token+b.token;
  end
endmodule
module string_unit(output string value);
  class string_helper; static string token=init_count_pkg::label_text(); endclass
  initial begin string_helper h=new; value=h.token; end
endmodule
module constructor_unit(output bit live);
  class constructor_helper; static registered_marker token=new; endclass
  initial begin constructor_helper h=new; live=(h.token!=null); end
endmodule
module parameter_unit(output int explicit_value, default_value);
  class param_helper #(int N=2);
    static int token=init_count_pkg::mark(5)+N;
  endclass
  initial begin
    param_helper#(7) a=new; param_helper b=new;
    explicit_value=a.token; default_value=b.token;
  end
endmodule
module collision_unit(output int value);
  class collision_helper; static int token=init_count_pkg::mark(6); endclass
  initial begin collision_helper h=new; value=h.token; end
endmodule
module top;
  int one, second, pair1, pair2, sum, explicit_value, default_value, collision;
  string label; bit live; int errors;
  single_unit u0(one,second);
  pair_unit u1(pair1),u2(pair2);
  two_class_unit u3(sum);
  string_unit u4(label);
  constructor_unit u5(live);
  parameter_unit u6(explicit_value,default_value);
  collision_unit u7(collision);
  task automatic ck(string name, bit ok);
    if(ok) $display("PASS %s",name);
    else begin errors++; $display("FAIL %s",name); end
  endtask
  initial begin
    collision_helper global_object=new;
    #1;
    ck("one class and two handles",init_count_pkg::calls[0]==1 && one==1 && second==1);
    ck("independent module classes",init_count_pkg::calls[1]==2 && pair1+pair2==3);
    ck("two class declarations",init_count_pkg::calls[2]==2 && sum==3);
    ck("string function initializer",init_count_pkg::calls[3]==1 && label=="ready");
    ck("constructor initializer",init_count_pkg::calls[4]==1 && live);
    ck("default and explicit specialization",init_count_pkg::calls[5]==2 && explicit_value+default_value==12);
    ck("global name collision",init_count_pkg::calls[6]==2 && collision>0 && global_object.token>0);
    if(errors==0) $display("ALL PASS"); else $display("FAILED %0d",errors);
    $finish;
  end
endmodule
