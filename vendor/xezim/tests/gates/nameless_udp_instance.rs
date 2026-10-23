//! §29.3: a UDP instance may omit its name (`p (q, d);`); a module instance
//! may not (§23.3.2). Both match the reference simulator.

use xezim::simulate;

#[test]
fn nameless_udp_instances_simulate() {
    let src = r#"
primitive p (q, d);
  output q; input d;
  table
    0 : 1;
    1 : 0;
  endtable
endprimitive
module top;
  reg a;
  wire q, r;
  p (q, a), (r, q);
  initial begin
    a = 0;
    #1 $display("P|%b %b", q, r);
    a = 1;
    #1 $display("P|%b %b", q, r);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with("P|"))
        .collect();
    assert_eq!(got, ["P|1 0", "P|0 1"]);
}

#[test]
fn nameless_module_instance_is_rejected() {
    let src = "module sub(input a, output b); assign b = a; endmodule\n\
               module top; wire x, y; sub (x, y); endmodule\n";
    let e = simulate(src, 10)
        .err()
        .expect("a nameless module instance must be rejected");
    assert!(e.contains("§23.3.2"), "{e}");
}
