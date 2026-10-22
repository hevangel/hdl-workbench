// program_localparam.sv
//
// A user-defined function call inside a constant expression is not evaluated
// in a program-block localparam when the function is declared at
// compilation-unit scope. The same call is evaluated correctly when the
// function is declared in the program itself or in a package.

package fib_pkg;
    function automatic int unsigned pkg_fib(int unsigned n);
        if (n < 2) return n;
        return pkg_fib(n - 1) + pkg_fib(n - 2);
    endfunction
endpackage

// compilation-unit scope
function automatic int unsigned unit_fib(int unsigned n);
    if (n < 2) return n;
    return unit_fib(n - 1) + unit_fib(n - 2);
endfunction

program top;

    import fib_pkg::*;

    function automatic int unsigned prog_fib(int unsigned n);
        if (n < 2) return n;
        return prog_fib(n - 1) + prog_fib(n - 2);
    endfunction

    localparam int unsigned V_PROG = prog_fib(15);     // expected 610
    localparam int unsigned V_PKG  = pkg_fib(15);      // expected 610
    localparam int unsigned V_UNIT = unit_fib(15);     // expected 610

    initial begin
        $display("VALUE_PROG=%0d", V_PROG);
        $display("VALUE_PKG=%0d", V_PKG);
        $display("VALUE_UNIT=%0d", V_UNIT);
        $finish;
    end

endprogram
