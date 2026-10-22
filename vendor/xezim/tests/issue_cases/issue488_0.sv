// class_const_fn_not_evaluated.sv
//
// A user-defined function call inside a constant expression is not evaluated
// in two contexts:
//   * a class-body localparam of a non-parameterized class
//   * the value of an enumeration item
//
// The same call is evaluated correctly at module scope, in a class static
// property, and in the localparam of a parameterized class.
//
// The program-block case is a separate file (program_localparam.sv), because
// a program declaration inside a module is not supported by every tool.

package chk_pkg;
    int checked  = 0;
    int failures = 0;

    function automatic void report(string what, bit ok);
        checked++;
        if (ok) $display("PASS: %s", what);
        else begin
            failures++;
            $display("FAIL: %s", what);
        end
    endfunction
endpackage

module top;

    import chk_pkg::*;

    // ---- the functions used in the constant expressions ----
    function automatic int unsigned fib(int unsigned n);
        if (n < 2) return n;
        return fib(n - 1) + fib(n - 2);
    endfunction

    function automatic int unsigned add3(int unsigned n);
        return n + 3;
    endfunction

    // ---- affected: class-body localparam, non-parameterized class ----
    class plain_cls;
        localparam int unsigned V_REC     = fib(15);      // expected 610
        localparam int unsigned V_NONREC = add3(607);     // expected 610
        localparam int unsigned V_CONST  = 610;           // expected 610
    endclass

    // ---- affected: enumeration item value ----
    typedef enum int { E_A = 5, E_B = E_A + fib(3) } e_t;  // expected 7

    // ---- controls ----
    // module scope
    localparam int unsigned M_REC     = fib(15);           // expected 610
    localparam int unsigned M_NONREC = add3(607);          // expected 610

    // class static property
    class static_cls;
        static int unsigned V = fib(15);                   // expected 610
    endclass

    // parameterized class: the localparam is evaluated with the
    // specialization bound
    class param_cls #(parameter int unsigned W = 8);
        localparam int unsigned V = fib(15);                // expected 610
    endclass

    initial begin
        // --- affected ---
        report("class localparam, recursive call",   plain_cls::V_REC     == 610);
        report("class localparam, plain call",       plain_cls::V_NONREC == 610);
        report("enum item value, recursive call",    e_t'(E_B)           == 7);

        // --- controls ---
        report("control: class localparam, constant",  plain_cls::V_CONST  == 610);
        report("control: module localparam, recursive", M_REC              == 610);
        report("control: module localparam, plain",     M_NONREC          == 610);
        report("control: class static property",        static_cls::V      == 610);
        report("control: parameterized class localparam",
               param_cls#(.W(8))::V == 610);

        $display("=== %0d check(s), %0d failure(s) ===", chk_pkg::checked,
                 chk_pkg::failures);
    end

endmodule
