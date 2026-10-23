//! IEEE 1800-2023 §13.5.2: an `output` formal whose actual is a `[2][2]`
//! class property (`fm(M)`) or a fixed-array property reached through a
//! handle chain (`ff2(o.i.F)`) writes back, also after earlier method calls
//! whose array formal had the same name. A class method left its array
//! formals registered after the call, so a later same-named formal of
//! another shape was read through the stale shape and the copy-back was
//! lost. Expected lines are the reference simulator's output.

use xezim::simulate;

#[test]
fn multidim_and_chained_property_actuals_write_back() {
    let out: Vec<String> = simulate(
        r#"
class In; int F[2]; endclass
class Out; In i; function new(); i = new; endfunction endclass
class C;
  int F[3];
  int M[2][2];
  function void ff(output int p[3]); p[0] = 1; p[1] = 2; p[2] = 3; endfunction
  function void fm(output int p[2][2]); p[1][0] = 7; endfunction
  function void rf(ref int p[3]); p[2] = 33; endfunction
  function void run();
    ff(F);
    $display("T| d1 F %0d %0d %0d", F[0], F[1], F[2]);
    rf(F);
    $display("T| d1 rF %0d", F[2]);
    fm(M);
    $display("T| d1 M %0d", M[1][0]);
  endfunction
endclass
module top;
  function automatic void ff2(output int p[2]); p[0] = 4; p[1] = 5; endfunction
  initial begin
    C c = new; Out o = new;
    c.run();
    $display("T| d1 ext F %0d %0d %0d", c.F[0], c.F[1], c.F[2]);
    ff2(o.i.F);
    $display("T| d1 o.i.F %0d %0d", o.i.F[0], o.i.F[1]);
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed")
    .output
    .iter()
    .map(|o| o.message.clone())
    .filter(|m| m.starts_with("T|"))
    .collect();
    let want = [
        "T| d1 F 1 2 3",
        "T| d1 rF 33",
        "T| d1 M 7",
        "T| d1 ext F 1 2 33",
        "T| d1 o.i.F 4 5",
    ];
    assert_eq!(out, want, "{out:?}");
}
