//! §9.2.1: the untimed-`always` rejection must see every form of timing
//! control. An intra-assignment EVENT control (`always v = @(e) x;`, ivtest
//! `always3.1.1I/J`) and a task enable written without parentheses
//! (`always my_task;`, ivtest `pr1862744a`, where the task holds the delay)
//! were both reported as "no timing control", failing legal designs.

use xezim::simulate;

#[test]
fn intra_assignment_event_control_counts_as_timing() {
    let src = "module main; reg [3:0] v; reg ev = 0;\n\
               initial begin #2 ev = 1; #1 $display(\"T|%h\", v); $finish; end\n\
               always v = @ (ev) 4'h5;\n\
               endmodule\n";
    let sim = simulate(src, 100).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "T|5"));
}

#[test]
fn bare_task_enable_counts_as_timing() {
    let src = "module main; reg v = 0;\n\
               task t; #1 v = 1; endtask\n\
               always t;\n\
               initial begin #3 $display(\"T|%b\", v); $finish; end\n\
               endmodule\n";
    let sim = simulate(src, 100).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "T|1"));
}

#[test]
fn untimed_always_is_still_rejected() {
    assert!(simulate("module main; reg v; always v = 1'b1; endmodule", 10).is_err());
}
