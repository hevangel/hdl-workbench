//! IEEE 1800 §13.4.3 / §6.19: constant functions work in each declaration scope.
use xezim::simulate;
#[test]
fn class_and_enum_constant_functions() {
    let result = simulate(include_str!("../issue_cases/issue488_0.sv"), 100).unwrap();
    assert!(
        result
            .output
            .iter()
            .any(|o| o.message == "=== 8 check(s), 0 failure(s) ==="),
        "{:?}",
        result.output
    );
}
#[test]
fn program_compilation_unit_constant_function() {
    let result = simulate(include_str!("../issue_cases/issue488_1.sv"), 100).unwrap();
    for line in ["VALUE_PROG=610", "VALUE_PKG=610", "VALUE_UNIT=610"] {
        assert!(
            result.output.iter().any(|o| o.message == line),
            "missing {line}: {:?}",
            result.output
        );
    }
}

#[test]
fn compilation_unit_functions_are_visible_in_interfaces_and_program_headers() {
    let result = simulate(
        r#"
 function automatic int add3(input int n);return n+3;endfunction
 interface bus_if;localparam int N=add3(7);endinterface
 program app #(parameter int N=add3(8))();initial $display("T|program %0d",N);endprogram
 module top;bus_if b();app a();initial $display("T|interface %0d",b.N);endmodule
 "#,
        100,
    )
    .unwrap();
    for line in ["T|program 11", "T|interface 10"] {
        assert!(
            result.output.iter().any(|o| o.message == line),
            "{:?}",
            result.output
        );
    }
}
