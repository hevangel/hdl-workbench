//! §13.4: the names in a function's return range and in subroutine port
//! ranges are resolved in the enclosing scope and must be declared there.
//! The reference simulator rejects the same modules and runs the legal one
//! with the same output.

use xezim::simulate;

#[test]
fn undeclared_range_names() {
    for sub in [
        "function [w-1:0] copy;\n    input [w-1:0] z;\n    copy = z;\n  endfunction",
        "function [7:0] copy;\n    input [w-1:0] z;\n    copy = z;\n  endfunction",
        "task t;\n    input [N:0] z;\n    $display(\"%0d\", z);\n  endtask",
    ] {
        let src = format!("module test;\n  {sub}\nendmodule\n");
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn declared_range_names() {
    let src = r#"
module test;
  parameter W = 4;
  localparam N = 2;
  function [W-1:0] copy;
    input [W-1:0] z;
    copy = z;
  endfunction
  task t;
    input [N:0] z;
    $display("S|%0d", z);
  endtask
  initial begin
    $display("S|%0d", copy(4'd9));
    t(3'd5);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("declared range names are legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("S|"))
        .collect();
    assert_eq!(lines, ["9", "5"], "{:?}", sim.output);
}
