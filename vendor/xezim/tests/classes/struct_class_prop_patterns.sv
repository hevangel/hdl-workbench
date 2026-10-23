// Assignment patterns written to unpacked-struct class properties.
module pc;
  typedef struct { int x; int y; } pl_t;
  typedef struct { int z; pl_t in; } nest_t;
  class C;
    pl_t a;
    nest_t n;
    pl_t arr[2];
    function new();
      a = '{x: 1, y: 2};
      n = '{z: 3, in: '{x: 4, y: 5}};
      arr[1] = '{6, 7};
    endfunction
    function void reset();
      a = '{default: 9};
      n = '{default: 8};
    endfunction
  endclass
  C c;
  initial begin
    c = new;
    $display("T|ctor a=%0d,%0d n=%0d,%0d,%0d arr1=%0d,%0d", c.a.x, c.a.y, c.n.z, c.n.in.x, c.n.in.y,
             c.arr[1].x, c.arr[1].y);
    c.a = '{y: 11, x: 10};
    c.n = '{1, '{2, 3}};
    $display("T|ext a=%0d,%0d n=%0d,%0d,%0d", c.a.x, c.a.y, c.n.z, c.n.in.x, c.n.in.y);
    c.reset();
    $display("T|dflt a=%0d,%0d n=%0d,%0d,%0d", c.a.x, c.a.y, c.n.z, c.n.in.x, c.n.in.y);
    $display("T|p %p", c.n);
    $finish;
  end
endmodule
