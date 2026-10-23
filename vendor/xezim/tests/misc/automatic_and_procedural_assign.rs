//! §6.21: an automatic variable is not written by a nonblocking assignment
//! and not used in a procedural continuous assignment (§10.6), whose `assign`
//! target is a whole variable (§10.6.1). Each rejected case is also rejected
//! by the reference simulator, and the legal module prints the same line
//! there.

use xezim::simulate;

fn rejected(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("accepted an illegal design:\n{src}"),
        Err(e) => e,
    }
}

#[test]
fn automatic_variables_and_procedural_assign_targets() {
    let e = rejected("module top; task automatic t; int l; l <= 1; endtask initial t; endmodule");
    assert!(e.contains("nonblocking"), "{e}");
    rejected(
        "module top; reg g; task automatic t; reg l; l = 1; assign g = l; endtask\n\
         initial t; endmodule",
    );
    let e = rejected(
        "module top; reg [3:0] v; integer i; initial begin i = 1; assign v[i] = 1'b1; end endmodule",
    );
    assert!(e.contains("§10.6.1"), "{e}");
}

#[test]
fn static_variables_and_force_selects_are_legal() {
    let src = r#"
module top;
  reg [3:0] v;
  reg g, h;
  int s;
  task t; int l; l <= 1; #1 s = l; endtask
  task automatic u(output int o); int m; m = 4; g <= 1; o = m; endtask
  initial begin
    int o;
    assign h = g;
    t;
    u(o);
    force v[0] = 1'b1;
    #2 $display("L5|%0d %0d %b %b %b", s, o, g, h, v[0]);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal design must run");
    assert!(
        sim.output.iter().any(|o| o.message == "L5|1 4 1 1 1"),
        "{:?}",
        sim.output
    );
}
