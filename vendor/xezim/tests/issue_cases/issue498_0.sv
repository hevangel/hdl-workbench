// type_param_in_body.sv
//
// A `parameter type` declared in a body scope (module body, package, class
// body) is not bound to its declared type by xezim:
//
//   * $bits(T) evaluates to 0 instead of the width of the declared type
//   * a declaration of the form `T v;` falls back to `int` (32 bit)
//
// The same type parameter declared in a module parameter port list or in a
// class parameter list works.

package p;
    parameter type T = logic [3:0];
    T v;
endpackage

class c;
    parameter type CT = logic [5:0];
    CT cv;
    function new();
    endfunction
endclass

// control: type parameter in a module parameter port list
module mp #(parameter type PT = logic [3:0]) ();
    localparam int PBITS = $bits(PT);
    initial $display("VALUE_PORTLIST_TYPEBITS=%0d", PBITS);
endmodule

module m;
    parameter type MT = logic [7:0];
    MT mv;
    localparam int MBITS = $bits(MT);
    initial begin
        $display("VALUE_MOD_BITS=%0d", $bits(mv));
        $display("VALUE_MOD_TYPEBITS=%0d", MBITS);
    end
endmodule

module top;
    m u();
    c cu;

    // control: type parameter in a module parameter port list
    mp up();

    initial begin
        cu = new();
        $display("VALUE_PKG_BITS=%0d", $bits(p::v));
        $display("VALUE_CLS_BITS=%0d", $bits(cu.cv));
        #1 $finish;
    end
endmodule
