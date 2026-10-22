//! §7.2: a struct member reference names a member of the struct; §6.20: a
//! parameter, localparam or enum member cannot be assigned. Each rejected
//! case is also rejected by the reference simulator, and the legal module
//! prints the same line there.

use xezim::simulate;

#[test]
fn unknown_struct_members_are_rejected() {
    for src in [
        "module test; struct packed {logic a;} s; assign s.a = 1; assign s.c = 1; endmodule",
        "module test; struct packed {struct packed {logic a;} t;} u;\n\
         assign u.t.a = 1; assign u.t.c = 1; endmodule",
        "module test; struct {int a; int b;} s; int y; initial y = s.z; endmodule",
    ] {
        let e = simulate(src, 10)
            .err()
            .expect("an unknown member was accepted");
        assert!(e.contains("§7.2"), "{e}");
    }
}

#[test]
fn constants_are_not_assignment_targets() {
    for src in [
        "module test #(parameter id_8 = 32'd99) (); assign id_8 = 1; endmodule",
        "module test; localparam L = 3; initial L = 4; endmodule",
        "module test; typedef enum {A, B} e_t; initial A = B; endmodule",
    ] {
        let e = simulate(src, 10)
            .err()
            .expect("an assignment to a constant was accepted");
        assert!(e.contains("§6.20"), "{e}");
    }
}

#[test]
fn struct_members_and_parameters_in_legal_uses() {
    let src = r#"
module test;
  typedef struct packed { logic [3:0] a; logic [3:0] b; } p_t;
  struct { int x; p_t p; } s;
  localparam p_t K = 8'h5A;
  initial begin
    s.x = 3; s.p.a = K.b; s.p.b = K.a;
    $display("S|%0d %h %h", s.x, s.p.a, s.p.b);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal design must run");
    assert!(
        sim.output.iter().any(|o| o.message == "S|3 a 5"),
        "{:?}",
        sim.output
    );
}
