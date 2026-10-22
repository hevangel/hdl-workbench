`timescale 1ns/1ns
typedef struct { string path; int offset; int size; } slice_t;
module top;
    slice_t a[], b[], g[];
    function void set_slices(slice_t t[]);
        g = t;
    endfunction
    initial begin
        int ok = 1;
        a = '{ '{"$root.dut.b1.r1", -1, -1}, '{"x.y", 4, 8} };
        b = a;
        if (b.size() != 2 || b[0].path != "$root.dut.b1.r1" || b[1].offset != 4 || b[1].size != 8) ok = 0;
        set_slices('{ '{"lit.path", 7, 9} });
        if (g.size() != 1 || g[0].path != "lit.path" || g[0].offset != 7) ok = 0;
        if (ok) $display("TAG_PASS"); else $display("TAG_FAIL");
    end
endmodule
