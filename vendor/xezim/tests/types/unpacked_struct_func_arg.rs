//! Passing an unpacked struct (or struct containing dynamic string / unpacked fields)
//! as a value argument to functions or class methods.
//!
//! §6.18 / §6.20: Unpacked structs split member-wise into formal ports.
//! Previously, passing an unpacked struct by value failed to bind the member ports,
//! causing functions/methods taking an unpacked struct (such as `uvm_element_container::add`)
//! to receive default/empty values for struct fields.

use xezim::simulate;

const SRC: &str = r#"
package pkg;
    typedef struct {
        int a;
        string b;
    } my_struct_t;

    class my_class;
        function int get_a(my_struct_t s);
            return s.a;
        endfunction

        function string get_b(my_struct_t s);
            return s.b;
        endfunction
    endclass

    function int func_get_a(my_struct_t s);
        return s.a;
    endfunction
endpackage

module top;
    import pkg::*;

    int res_func_a;
    int res_class_a;

    initial begin
        my_struct_t st;
        my_class c;
        st.a = 42;
        st.b = "hello";
        c = new();

        res_func_a = func_get_a(st);
        res_class_a = c.get_a(st);
    end
endmodule
"#;

fn get_sig(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

#[test]
fn test_unpacked_struct_func_and_method_arg() {
    let sim = simulate(SRC, 1000).expect("compilation & elaboration failed");
    let fa = get_sig(&sim, "res_func_a");
    let ca = get_sig(&sim, "res_class_a");

    assert_eq!(
        fa, 42,
        "Function call failed to bind unpacked struct argument"
    );
    assert_eq!(
        ca, 42,
        "Method call failed to bind unpacked struct argument"
    );
}
