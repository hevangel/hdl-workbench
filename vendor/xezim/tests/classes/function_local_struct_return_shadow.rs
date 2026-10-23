//! §13.4.1: `return r;` of a function-LOCAL unpacked struct came back x when
//! an `initial` block elsewhere declares a struct variable of the same name.
//!
//! The local's members live in the call frame (`r.level`, `r.name`), but a
//! bare `r` read as a value consults the module's variables first — and the
//! `initial`-block `r` answered with its own (never assembled) container.
//! A returned struct local is now assembled from the frame's leaves. Every
//! expectation below was cross-checked against the reference simulator.

use xezim::simulate;

#[test]
fn returned_local_struct_ignores_same_named_module_variable() {
    let sim = simulate(
        r#"
typedef struct { int level; string name; } row_t;
class C;
  row_t da[];
  function row_t mk(int l, string n);
    row_t r;
    r.level = l; r.name = n;
    return r;
  endfunction
  function void run();
    row_t r2;
    da = new[2];
    da[1] = mk(3, "three");
    $display("D1 %0d '%s'", da[1].level, da[1].name);
    r2 = mk(4, "four");
    $display("R2 %0d '%s'", r2.level, r2.name);
  endfunction
endclass
module top;
  initial begin
    C c = new;
    row_t r;
    c.run();
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for w in ["D1 3 'three'", "R2 4 'four'"] {
        assert!(o.iter().any(|l| l == w), "expected {w:?} in:\n{o:#?}");
    }
}
