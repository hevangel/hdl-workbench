// #291 as reported: exported functions with a chandle formal, a string
// result and a 128-bit vector are called from C. Each was a stub returning 0
// that never ran the function. The 128-bit vector travels through an output
// (a wide result is not legal DPI, see export_wide_result_test.sv).
// Expected values from the reference simulator.
module top;
    import "DPI-C" context function void c_main();
    int hits = 0;
    function void put(string s); $display("T|%s", s); endfunction
    export "DPI-C" function put;
    function int ok_add(input int a, input int b);
        hits++;
        return a + b;
    endfunction
    export "DPI-C" function ok_add;
    function int add2_with_h(input int a, input int b, input chandle h);
        hits++;
        return a + b + (h == null ? 0 : 100);
    endfunction
    export "DPI-C" function add2_with_h;
    function string str_ret();
        hits++;
        return "hello";
    endfunction
    export "DPI-C" function str_ret;
    function void wide(input logic [127:0] v, output logic [127:0] o);
        hits++;
        o = ~v;
    endfunction
    export "DPI-C" function wide;
    initial begin
        c_main();
        $display("T|HITS=%0d of 5", hits);
        $finish;
    end
endmodule
