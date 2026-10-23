// §6.20.3 / §8.23: inlined module type parameters retain scoped receivers.
package receiver_pkg;
  class item #(string LABEL = "pkg");
    static function string label_text(); return LABEL; endfunction
  endclass
endpackage
class item #(string LABEL = "global");
  static function string label_text(); return LABEL; endfunction
endclass
module receiver_probe #(type T = item#("default"))
    (output string bare, paren, initial_value, task_value, output int shadows);
  function string get_bare(); return T::label_text; endfunction
  function string get_paren(); return T::label_text(); endfunction
  task automatic get_task(output string value); value = T::label_text; endtask
  function automatic int formal_shadow(int T); return T + 1; endfunction
  function automatic int local_shadow(); int T = 5; return T + 1; endfunction
  initial begin
    bare = get_bare(); paren = get_paren(); initial_value = T::label_text;
    get_task(task_value); shadows = formal_shadow(3) + local_shadow();
  end
endmodule
module top;
  typedef item#("alias") alias_one;
  typedef alias_one alias_two;
  string b[4], p[4], i[4], t[4]; int s[4]; int errors;
  receiver_probe u0(b[0],p[0],i[0],t[0],s[0]);
  receiver_probe #(item#("explicit")) u1(b[1],p[1],i[1],t[1],s[1]);
  receiver_probe #(alias_two) u2(b[2],p[2],i[2],t[2],s[2]);
  receiver_probe #(receiver_pkg::item#("qualified")) u3(b[3],p[3],i[3],t[3],s[3]);
  task automatic ck(string name, bit ok);
    if (ok) $display("PASS %s",name); else begin errors++; $display("FAIL %s",name); end
  endtask
  initial begin
    #1;
    ck("default binding", b[0]=="default" && p[0]=="default" && i[0]=="default" && t[0]=="default");
    ck("explicit binding", b[1]=="explicit" && p[1]=="explicit" && i[1]=="explicit" && t[1]=="explicit");
    ck("typedef binding", b[2]=="alias" && p[2]=="alias" && i[2]=="alias" && t[2]=="alias");
    ck("package binding", b[3]=="qualified" && p[3]=="qualified" && i[3]=="qualified" && t[3]=="qualified");
    ck("formal and local shadow", s[0]==10 && s[1]==10 && s[2]==10 && s[3]==10);
    if (errors==0) $display("ALL PASS"); else $display("FAILED %0d",errors);
    $finish;
  end
endmodule
