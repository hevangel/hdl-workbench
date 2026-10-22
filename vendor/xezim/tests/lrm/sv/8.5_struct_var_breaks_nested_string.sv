// top: t8_5c   (any unpacked-struct variable anywhere in the design - even in another module - breaks reads of g.h.<member>: strings become spaces, ints x. Packed struct or struct inside a class: fine)
typedef struct { int k; int h; } rec_t;
class Item; string tag = "abc"; int n = 7; function string show(); return "show"; endfunction endclass
class Hold; Item h; function new(); h = new; endfunction function Item get(); return h; endfunction endclass
module t8_5c;
  Hold g; Item i; rec_t r;   // removing `rec_t r;` makes the strings correct
  string s;
  initial begin
    g = new; i = new;
    $display("T|a|[%s] [%s] [%s] [%s] [%s]", g.get().show(), g.h.tag, g.h.show(), i.tag, i.show());
    s = g.h.tag; $display("T|b|assigned [%s] len=%0d eq=%0d n=%0d", s, s.len(), g.h.tag == "abc", g.h.n);
  end
endmodule
