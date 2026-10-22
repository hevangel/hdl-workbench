// §8.3 / §23.6: module classes retain lexical and instance identity.
class holder;
  static function string label_text(); return "global"; endfunction
  function int id(); return 1; endfunction
endclass
class base_holder;
  function int inherited(); return 100; endfunction
endclass
module local_a(output int id, output int count, output string label);
  class holder;
    static int calls;
    static function string label_text(); return "local-a"; endfunction
    static function int next(); calls++; return calls; endfunction
    function int id(); return 11; endfunction
  endclass
  initial begin
    holder h = new;
    $display("T|type %m %s",$typename(h)); id = h.id(); count = holder::next(); label = holder::label_text();
  end
endmodule
module local_b(output int id, output string label);
  class holder;
    static function string label_text(); return "local-b"; endfunction
    function int id(); return 22; endfunction
  endclass
  holder h = new;
  initial begin $display("T|type %m %s",$typename(h)); id = h.id(); label = holder::label_text(); end
endmodule
module local_alias(output int id);
  class holder; function int id(); return 33; endfunction endclass
  typedef holder alias_t;
  typedef alias_t chained_t;
  initial begin chained_t h = new; $display("T|type %m %s",$typename(h)); id = h.id(); end
endmodule
module local_inheritance(output int id);
  class base_holder; function int inherited(); return 4; endfunction endclass
  class holder extends base_holder;
    function int id(); return inherited() + 40; endfunction
  endclass
  initial begin holder h = new; $display("T|type %m %s",$typename(h)); id = h.id(); end
endmodule
module local_factory(output int id);
  class holder;
    function int id(); return 55; endfunction
    static function holder make(); holder h = new; return h; endfunction
  endclass
  initial begin holder h; h = holder::make(); $display("T|type %m %s",$typename(h)); id = h.id(); end
endmodule
module local_type_parameter(output string label);
  class holder; endclass
  class wrapper #(type holder = int);
    holder value;
    function string type_name(); return $typename(holder); endfunction
  endclass
  initial begin wrapper #(byte) h = new; label = h.type_name(); end
endmodule
module top;
  int a, a2, b, aliased, inherited, factory, n1, n2, errors;
  string la, la2, lb, parameter_label;
  local_a u1(a, n1, la), u2(a2, n2, la2);
  local_b u3(b, lb);
  local_alias u4(aliased);
  local_inheritance u5(inherited);
  local_factory u6(factory);
  local_type_parameter u7(parameter_label);
  task automatic ck(string label, bit ok);
    if (ok) $display("PASS %s", label);
    else begin errors++; $display("FAIL %s", label); end
  endtask
  initial begin
    holder h = new;
    #1;
    ck("global static function", holder::label_text() == "global");
    ck("global object", h.id() == 1);
    ck("first local object", a == 11);
    ck("second instance object", a2 == 11);
    ck("sibling local object", b == 22);
    ck("first local static", la == "local-a");
    ck("second local static", la2 == "local-a");
    ck("sibling local static", lb == "local-b");
    ck("independent class statics", n1 == 1 && n2 == 1);
    ck("module-local typedef chain", aliased == 33);
    ck("local inheritance", inherited == 44);
    ck("local factory return", factory == 55);
    ck("class parameter shadows enclosing type", parameter_label == "byte");
    if (errors == 0) $display("ALL PASS"); else $display("FAILED %0d", errors);
    $finish;
  end
endmodule
