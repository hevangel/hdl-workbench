//! IEEE 1800-2017 §11.5.1: the bounds of a constant part-select `[l:r]` are
//! constant expressions. A variable bound is an elaboration error (every
//! engine used to read it at run time from the variable's current value);
//! parameter-derived bounds and dynamic bit-selects stay legal.
use xezim::simulate;

fn errs(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => String::new(),
        Err(e) => e.to_string(),
    }
}

#[test]
fn variable_bound_in_constant_part_select_is_an_error() {
    for (tag, body) in [
        ("left", "r = d[k:0];"),
        ("right", "r = d[3:m];"),
        ("both", "r = d[k:m];"),
        ("comb", "always_comb r = d[k:0];"),
        ("assign", "assign w = d[k:0];"),
    ] {
        let (proc_body, item) = if body.starts_with("r = ") {
            (body, "")
        } else {
            ("", body)
        };
        let src = format!(
            "module top;\n  logic [7:0] d = 8'hB7; int k = 3, m = 1; logic [3:0] r; wire [3:0] w;\n  {}\n  initial begin {} end\nendmodule\n",
            item, proc_body
        );
        let e = errs(&src);
        assert!(
            e.contains("must be constant"),
            "{tag}: expected the §11.5.1 error, got: {e:?}"
        );
    }
}

#[test]
fn constant_bounds_and_dynamic_bit_selects_stay_legal() {
    let src = r#"
module top;
  logic [7:0] d = 8'hB7; int k = 3; logic [3:0] r4; logic w1; logic [1:0] r5;
  parameter int P = 2; localparam int Q = P + 1;
  initial begin r4 = d[Q:P]; w1 = d[k]; r5 = d[k +: 2]; end
endmodule
"#;
    let e = errs(src);
    assert!(e.is_empty(), "legal forms rejected: {e}");
}
