//! §13.5: an event is not a value, so it cannot be passed to a subroutine
//! input that is not an event. The reference simulator rejects that call and
//! runs the legal module with the same output.

use xezim::simulate;

#[test]
fn event_to_value_argument() {
    let src = r#"
module top;
  event evt;
  reg rval;
  function func;
    input arg;
    func = arg;
  endfunction
  initial rval = func(evt);
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn values_to_value_arguments() {
    let src = r#"
module top;
  event evt;
  reg rval;
  function func;
    input arg;
    func = ~arg;
  endfunction
  initial begin
    fork
      begin @(evt) $display("E|seen"); end
      begin #1 -> evt; end
    join
    rval = func(1'b0);
    $display("E|%b", rval);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("value arguments are legal");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("E|"))
        .collect();
    assert_eq!(lines, ["seen", "1"], "{:?}", sim.output);
}
