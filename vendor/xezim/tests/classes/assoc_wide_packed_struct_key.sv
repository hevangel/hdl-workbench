`timescale 1ns/1ns
typedef struct packed {
    longint unsigned min;
    longint unsigned max;
    int unsigned stride;
} range_t;

class C;
    int data[range_t];
endclass

module top;
    C c;
    initial begin
        range_t r;
        c = new();
        r.min = 64'h1000;
        r.max = 64'h2000;
        r.stride = 4;
        c.data[r] = 42;
        $display("c.data.size = %0d", c.data.size());
        foreach (c.data[k]) begin
            $display("k.min=%0h k.max=%0h k.stride=%0d val=%0d", k.min, k.max, k.stride, c.data[k]);
            if (k.min == 64'h1000 && k.max == 64'h2000 && c.data[k] == 42)
                $display("TAG_PASS");
            else
                $display("TAG_FAIL");
        end
    end
endmodule
