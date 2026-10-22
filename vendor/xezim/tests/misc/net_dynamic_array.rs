//! §7.4: a net array has fixed-size dimensions only — a dynamic, queue or
//! associative net is rejected, as by the reference simulator.

use xezim::simulate;

#[test]
fn nets_cannot_be_dynamic_arrays() {
    for src in [
        "module top; wire x[]; endmodule",
        "module top; wire [3:0] x[$]; endmodule",
    ] {
        let e = simulate(src, 10)
            .err()
            .expect("a dynamic net array was accepted");
        assert!(e.contains("fixed-size dimensions"), "{e}");
    }
}
