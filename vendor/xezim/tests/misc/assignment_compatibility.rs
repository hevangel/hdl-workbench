//! Assignment compatibility (§6.19.3 enums, §7.6 unpacked arrays, §8.15 class
//! handles, §8.7 `new`; an input argument is an assignment too). Each rejected
//! case is also rejected by the reference simulator, and the legal module
//! compiles and prints the same line there.

use xezim::simulate;

fn rejected(src: &str) -> String {
    match simulate(src, 100) {
        Ok(_) => panic!("accepted an illegal design:\n{src}"),
        Err(e) => e,
    }
}

#[test]
fn enum_takes_only_its_own_type() {
    let e = rejected("module top; typedef enum {A, B} e_t; e_t e; initial e = 1; endmodule");
    assert!(e.contains("§6.19.3"), "{e}");
    // Arithmetic on an enum yields an int (`e += 1` is `e = e + 1`).
    rejected(
        "module top; typedef enum {A, B} e_t; e_t e; initial begin e = A; e += 1; end endmodule",
    );
    // Another enum type, an int argument, an int return value.
    rejected(
        "module top; typedef enum {A, B} e_t; typedef enum {C, D} f_t; e_t e; f_t f;\n\
         initial begin f = C; e = f; end endmodule",
    );
    rejected("module top; typedef enum {A, B} e_t; task t(e_t x); endtask initial t(0); endmodule");
    rejected(
        "module top; typedef enum {A, B} e_t; function e_t f(); return 1; endfunction\n\
         initial $display(f()); endmodule",
    );
    // Through a struct member, a class property and an array element.
    rejected(
        "module top; typedef enum {A, B} e_t; struct packed { e_t e; } s;\n\
         initial s.e = 1; endmodule",
    );
    rejected(
        "module top; typedef enum {A, B} e_t; class C; e_t e; endclass C c;\n\
         initial begin c = new; c.e = 1; end endmodule",
    );
    rejected("module top; typedef enum {A, B} e_t; e_t ea[2]; initial ea[0] = 1; endmodule");
    // A non-ANSI task port redeclared with the enum type.
    rejected(
        "typedef enum integer {A, B} T;\nmodule top; task t; input [31:0] x; T x; endtask\n\
         initial t(10); endmodule",
    );
}

#[test]
fn unpacked_arrays_need_same_shape_and_equivalent_elements() {
    let e = rejected("module top; logic [7:0] a [4]; bit [7:0] b [4]; initial a = b; endmodule");
    assert!(e.contains("not equivalent"), "{e}");
    rejected("module top; int a [4]; int b [3]; initial a = b; endmodule");
    rejected("module top; int a [4]; int b; initial a = b; endmodule");
    rejected("module top; wire [1:0] x [1:0][1:0]; reg [1:0] y [3:0]; assign x = y; endmodule");
    rejected(
        "module top; logic [31:0] d1 []; logic signed [31:0] d2 []; initial d1 = d2; endmodule",
    );
    rejected("module top; int q1 [$]; real q2 [$]; initial q1 = q2; endmodule");
}

/// §6.19.3/§7.6: an enum array takes no plain integral array (xezim has
/// rejected this since its first enum check, and ivtest expects it; the
/// reference simulator accepts it). The other direction is accepted by both.
#[test]
fn enum_arrays_take_no_integral_arrays() {
    rejected("module top; wire enum integer {A} x[1:0]; integer y[1:0]; assign x = y; endmodule");
    rejected(
        "module top; enum logic [31:0] {A} d1[]; logic [31:0] d2[]; initial d1 = d2; endmodule",
    );
    rejected(
        "module top; enum logic [31:0] {A} q1[$]; logic [31:0] q2[$]; initial q1 = q2; endmodule",
    );
    let src = "module top; wire integer x[1:0]; enum integer {A} y[1:0];\n\
               assign x = y;\n\
               initial begin y[0] = A; y[1] = A; #1 $display(\"I|%0d\", x[0]); end endmodule";
    let sim = simulate(src, 10).expect("an enum array into an integer array is legal");
    assert!(
        sim.output.iter().any(|o| o.message == "I|0"),
        "{:?}",
        sim.output
    );
}

#[test]
fn class_handles_take_only_derived_classes() {
    let e = rejected(
        "module top; class B; endclass class D extends B; endclass B b; D d;\n\
         initial begin b = new; d = b; end endmodule",
    );
    assert!(e.contains("§8.15"), "{e}");
    rejected(
        "module top; class B; endclass class C; endclass B b; C c;\n\
         initial begin c = new; b = c; end endmodule",
    );
    rejected(
        "module top; class B; endclass class C; endclass B b;\n\
         initial b = C::new; endmodule",
    );
    rejected("module top; int i; initial i = new; endmodule");
    rejected("module top; int i []; initial i = new; endmodule");
}

#[test]
fn legal_assignments_still_run() {
    let src = r#"
module top;
  typedef enum logic [1:0] {A, B, C} e_t;
  typedef e_t e2_t;
  typedef struct packed { e_t f; logic [3:0] g; } s_t;
  class Base; int v; endclass
  class Der extends Base; e_t e; endclass
  function automatic e_t pick(e_t x, int n = 1);
    return n > 0 ? x : A;
  endfunction
  function automatic int sum(int a, int b = 2, int c = 3);
    return a + b + c;
  endfunction
  e_t e, e1;
  e2_t e2;
  s_t s;
  int i;
  logic [7:0] m1 [4];
  reg   [7:0] m2 [4];
  logic [7:0] d1 [];
  logic [7:0] q1 [$];
  Base b;
  Der dd;
  initial begin
    e = B;
    e1 = e;
    e2 = e;
    e = e_t'(2);
    e = (i > 0) ? A : C;
    e = pick(C);
    e = pick(.x(B), .n(0));
    s.f = C;
    i = e + 1;
    i = e;
    m2 = '{1, 2, 3, 4};
    m1 = m2;
    d1 = new[4];
    d1 = m1;
    q1 = d1;
    q1 = {q1, 8'd5};
    dd = new;
    dd.e = B;
    b = dd;
    b = null;
    if (!$cast(dd, b)) $display("cast failed");
    b = Der::new;
    i = sum(1) + sum(1, 1) + sum(.a(1), .c(1));
    $display("L|%0d %0d %0d %0d %0d %0d %0d", e, e1, e2, s.f, i, m1[3], q1.size());
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("legal design must run");
    assert!(
        sim.output.iter().any(|o| o.message == "L|0 1 1 2 15 4 5"),
        "{:?}",
        sim.output
    );
}
