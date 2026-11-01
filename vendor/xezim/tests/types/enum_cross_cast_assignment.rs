//! §6.19.3: a value of enum type A assigned to a variable of enum type B is
//! illegal unless cast to B — a cast to a *different* enum A does not make
//! the assignment legal. The reference elaborator rejects
//! `f_t fv; fv = e_t'(1);`. xezim used
//! to lose the cast target's enum type and silently accepted it; the cast
//! target must type the assignment check.
use xezim::simulate;

#[test]
fn cross_enum_cast_to_other_enum_is_rejected() {
    // `e_t'(1)` has enum type e_t, but the target is f_t — a different enum.
    // Only `f_t'(1)` (cast to the target) is legal.
    const BAD: &str = r#"
module top;
  typedef enum logic [1:0] {A=0, B=1} e_t;
  typedef enum logic [1:0] {X=0, Y=1} f_t;
  f_t fv;
  initial begin
    fv = e_t'(1);   // cast to e_t, assigned to f_t: must be rejected
    $display("SHOULD NOT REACH");
  end
endmodule
"#;
    match simulate(BAD, 1) {
        Ok(_) => panic!("assigning enum e_t value to f_t variable must fail"),
        Err(e) => {
            assert!(
                e.contains("an enum takes only its own members"),
                "unexpected error text: {}",
                e
            );
        }
    }
}

#[test]
fn cast_to_target_enum_is_accepted() {
    // Casting to the *target* enum type (f_t) is an explicit cast and legal.
    const GOOD: &str = r#"
module top;
  typedef enum logic [1:0] {A=0, B=1} e_t;
  typedef enum logic [1:0] {X=0, Y=1} f_t;
  f_t fv;
  initial begin
    fv = f_t'(1);
    if (fv == f_t'(1)) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let sim = simulate(GOOD, 1).expect("cast to target enum type must compile");
    let out = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(out.contains("TAG_PASS"), "got:\n{}", out);
}
