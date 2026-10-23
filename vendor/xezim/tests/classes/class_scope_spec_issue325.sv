// A bare reference to a static function yields the wrong value when the
// enclosing class parameter is bound to a parameterized class.
class holder #(type T = int, string Tname = "<unknown>");
    static function string type_name();
        return Tname;
    endfunction
endclass

class plain_holder;
    static function string type_name();
        return "plain";
    endfunction
endclass

// the failing shape: bare reference, parameter bound to a parameterized class
class via_param_class #(type Tregistry = int);
    virtual function string get_type_name();
        return Tregistry::type_name;
    endfunction
endclass

// control 1: same binding, parenthesised call
class via_param_class_paren #(type Tregistry = int);
    virtual function string get_type_name();
        return Tregistry::type_name();
    endfunction
endclass

// control 2: bare reference, but the parameter is bound to a plain class
class via_plain_class #(type Tregistry = int);
    virtual function string get_type_name();
        return Tregistry::type_name;
    endfunction
endclass

module top;
    int failures = 0;

    task check(string what, string got, string want);
        if (got == want) begin
            $display("PASS %s: '%s'", what, got);
        end else begin
            $display("FAIL %s: got '%s', want '%s'", what, got, want);
            failures++;
        end
    endtask

    initial begin
        via_param_class       #(holder #(int, "base_test")) a;
        via_param_class_paren #(holder #(int, "base_test")) b;
        via_plain_class       #(plain_holder)               c;

        a = new();
        b = new();
        c = new();

        // the failing combination
        check("bare, param bound to parameterized class", a.get_type_name(), "base_test");

        // controls: each factor alone is fine
        check("parenthesised, same binding",              b.get_type_name(), "base_test");
        check("bare, param bound to plain class",         c.get_type_name(), "plain");

        $display("");
        if (failures == 0) $display("ALL CHECKS PASSED");
        else $display("%0d CHECK(S) FAILED", failures);
        $finish;
    end
endmodule
