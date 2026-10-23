//! §6.19: an enum's base type is an integer atom type or an integer vector
//! type with at most one packed dimension (also through a typedef); §7.2.1:
//! every member of a packed struct or union is packed. Each rejected case is
//! also rejected by the reference simulator, and the legal module prints the
//! same line there.

use xezim::simulate;

fn rejected(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("accepted an illegal design:\n{src}"),
        Err(e) => e,
    }
}

#[test]
fn illegal_enum_base_types() {
    for body in [
        "typedef logic T[1:0]; enum T {A} e;",
        "typedef logic T[]; enum T {A} e;",
        "typedef enum {X} T; enum T {A} e;",
        "typedef real T; enum T {A} e;",
        "typedef string T; enum T {A} e;",
        "typedef struct packed { int x; } T; enum T {A} e;",
        "typedef int T; enum T [1:0] {A} e;",
        "enum logic [1:0][1:0] {A} e;",
        "enum real {A} e;",
    ] {
        let e = rejected(&format!("module top; {body} endmodule"));
        assert!(e.contains("§6.19"), "{body}: {e}");
    }
}

#[test]
fn packed_aggregates_have_packed_members() {
    for body in [
        "struct packed { real r; } s;",
        "struct packed { int x; shortint y[2]; } s;",
        "union packed { int x[$]; } s;",
        "typedef struct packed { real r; } t; t s;",
    ] {
        let e = rejected(&format!("module top; {body} endmodule"));
        assert!(e.contains("§7.2.1"), "{body}: {e}");
    }
}

#[test]
fn legal_enum_bases_and_packed_members() {
    let src = r#"
module top;
  typedef logic [1:0] L2;
  typedef bit [1:0][1:0] B22;
  typedef int I;
  enum int {A0, A1} a;
  enum byte {B0 = 3} b;
  enum bit [3:0] {C0 = 4'd9} c;
  enum logic signed [7:0] {D0 = -8'sd2} d;
  enum L2 {E0, E1, E2} e;
  enum B22 {F0 = 7} f;
  enum I {G0 = 11} g;
  typedef struct packed { logic [3:0] n; } inner_t;
  struct packed { inner_t i; enum logic [1:0] {P, Q} k; int z; } s;
  initial begin
    e = E2; s.i.n = 4'd6; s.k = Q; s.z = -1;
    $display("L6|%0d %0d %0d %0d %0d %0d %0d", a, b, c, d, e, s.i.n, s.k);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("legal types must run");
    assert!(
        sim.output.iter().any(|o| o.message == "L6|0 0 0 x 2 6 1"),
        "{:?}",
        sim.output
    );
}
