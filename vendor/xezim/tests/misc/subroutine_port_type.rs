//! §13.3: a subroutine port's type name must be declared. The reference
//! simulator rejects the undeclared type and runs the legal module with the
//! same output.

use xezim::simulate;

#[test]
fn undeclared_port_type() {
    let src = r#"
module m;
  task t1;
    input make_me_crash i;
    begin
    end
  endtask
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn declared_port_types() {
    let src = r#"
package p;
  typedef logic [3:0] nib_t;
endpackage
module m;
  import p::*;
  typedef logic [7:0] byte_t;
  task t1;
    input byte_t i;
    input nib_t n;
    $display("T|%0d %0d", i, n);
  endtask
  task t2(input integer i);
    $display("T|%0d", i);
  endtask
  initial begin
    t1(8'd200, 4'd9);
    t2(-3);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("declared port types are legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("T|"))
        .collect();
    assert_eq!(lines, ["200 9", "-3"], "{:?}", sim.output);
}
