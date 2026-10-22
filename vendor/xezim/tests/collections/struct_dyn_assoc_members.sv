// Associative members: methods through a variable and through an AA element.
module p6;
  typedef struct { int x; int m[string]; } as_t;
  as_t a1;
  as_t e[int];
  string k;
  int n;
  initial begin
    a1.m["k"] = 9; a1.m["j"] = 8; a1.m["z"] = 7;
    void'(a1.m.first(k)); $display("T|first %s", k);
    void'(a1.m.next(k)); $display("T|next %s", k);
    void'(a1.m.last(k)); $display("T|last %s", k);
    a1.m.delete("j");
    $display("T|del n=%0d ex=%0d", a1.m.num(), a1.m.exists("j"));
    e[3].m["p"] = 1; e[3].m["q"] = 2;
    e[5].x = 4;
    $display("T|e n=%0d m=%0d q=%0d ex=%0d", e.num(), e[3].m.num(), e[3].m["q"], e[3].m.exists("p"));
    n = 0; foreach (e[3].m[s]) n += e[3].m[s];
    $display("T|e sum=%0d", n);
    e[3].m = '{"r": 5};
    $display("T|e asg n=%0d r=%0d", e[3].m.num(), e[3].m["r"]);
    e[7] = '{x: 1, m: '{"a": 10, "b": 20}};
    $display("T|e pat n=%0d e7=%0d b=%0d", e.num(), e[7].m.num(), e[7].m["b"]);
    a1 = e[7];
    $display("T|copy n=%0d a=%0d", a1.m.num(), a1.m["a"]);
    $display("T|p %p", e[7]);
    $finish;
  end
endmodule
