// §7.2.1, §7.4, §8.25: a field of an element of a class array of packed
// structs is a slice of the element, for writes as for reads, also when
// the element type is a class type parameter.
module class_struct_array_field_writes;
  typedef struct packed { logic v; logic [127:0] data; logic [15:0] mask; logic last; } beat_t;
  typedef struct packed { bit [3:0] a; bit [3:0] b; } sm_t;
  class C #(type T = beat_t);
    T w[]; beat_t u[]; sm_t s[]; sm_t f[2]; sm_t one;
    function void fill();
      w = new[3]; u = new[3]; s = new[2];
      w[1].last = 1; w[1].mask = 16'h1234; u[1].last = 1; u[1].mask = 16'h1234;
      s[1].b = 5; f[1].b = 6; f[0] = 8'h12; one.b = 3;
      $display("T|in w %0d %h u %0d %h s %h f %h %h one %h", w[1].last, w[1].mask, u[1].last, u[1].mask, s[1], f[1], f[0], one);
    endfunction
  endclass
  initial begin
    C x = new;
    x.fill();
    x.u[2].mask = 16'hbeef; x.s[0].a = 3; x.f[0].b = 7; x.w[2].v = 1;
    $display("T|out w %0d %h %0d u %0d %h %h s %h f %h", x.w[1].last, x.w[1].mask, x.w[2].v, x.u[1].last, x.u[1].mask, x.u[2].mask, x.s[0], x.f[0]);
    $display("T|bits %0d %0d", $bits(x.w[0]), $bits(x.w[0].mask));
  end
endmodule
