typedef struct { int x; } elem_t;
typedef elem_t aa_t [string];
module tb;
  elem_t d [string];
  aa_t t;
  int i [string];
  elem_t di [int];
  initial begin
    elem_t e; e.x = 1;
    d["a"] = e; d["b"] = e; t["a"] = e; t["b"] = e; i["a"] = 1; i["b"] = 2; di[1] = e; di[2] = e;
    d.delete("a"); t.delete("a"); i.delete("a"); di.delete(1);
    $display("T|01 d=%0d t=%0d i=%0d di=%0d", d.num(), t.num(), i.num(), di.num());
    $display("T|02 d=%0d t=%0d di=%0d", d.exists("a"), t.exists("a"), di.exists(1));
    $display("T|03 d=%p di=%p", d, di);
    d.delete(); di.delete();
    $display("T|04 d=%0d di=%0d", d.num(), di.num());
    $finish;
  end
endmodule
