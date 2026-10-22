`timescale 1ns/1ns
typedef struct packed { int unsigned a; int unsigned b; bit c; } st_t;
class C;
    st_t arr[3];
    st_t s;
    function new();
        s = '{1, 2, 1'b1};
        arr[0] = '{10, 20, 1'b1};
        arr[1] = '{11, 21, 1'b0};
    endfunction
endclass
module top;
    C o;
    initial begin
        o = new();
        $display("s      a=%0d b=%0d", o.s.a, o.s.b);
        $display("arr0   a=%0d b=%0d", o.arr[0].a, o.arr[0].b);
        $display("arr1   a=%0d c=%0b", o.arr[1].a, o.arr[1].c);
        if (o.s.a==1 && o.arr[0].a==10 && o.arr[1].a==11 && o.arr[1].c==0) $display("TAG_PASS");
        else $display("TAG_FAIL");
    end
endmodule
