//! §28.3–§28.9: gate and switch primitives take a fixed number of terminals
//! — an n-input or n-output gate two or more, a tri-state buffer three, a
//! `tran` two. Each rejected case is also rejected by the reference
//! simulator, and the legal module prints the same line there.

use xezim::simulate;

#[test]
fn wrong_terminal_counts_are_rejected() {
    for src in [
        "module top; wire iscl; buf (strong0, highz1) #1 sclbuf0(iscl); endmodule",
        "module top; wire iscl; not (iscl); endmodule",
        "module top; wire o; and (o); endmodule",
        "module top; wire o, a; bufif1 (o, a); endmodule",
        "module top; wire a, b, c; tran (a, b, c); endmodule",
    ] {
        let e = simulate(src, 10)
            .err()
            .expect("a gate with a wrong terminal count was accepted");
        assert!(e.contains("terminal"), "{e}");
    }
}

#[test]
fn legal_terminal_counts() {
    let src = r#"
module top;
  reg a, c;
  wire o1, o2, o3, o4, p;
  and (o1, a);
  buf (o2, o3, a);
  bufif1 (o4, a, c);
  pullup (p);
  initial begin a = 1; c = 1; #1 $display("T|%b %b %b %b %b", o1, o2, o3, o4, p); end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal gates must run");
    assert!(
        sim.output.iter().any(|o| o.message == "T|1 1 1 1 1"),
        "{:?}",
        sim.output
    );
}
