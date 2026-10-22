//! §13.5.1 — an `output`/`inout`/`ref` formal whose ACTUAL is a class
//! PROPERTY (a fixed-size unpacked array or an associative array reached
//! through `this`, whether spelled bare `prop`, `this.prop`, or `c.prop`)
//! must copy back onto the property, not onto a stray bare-named signal.
//!
//! The PR #275 fix (#277) made class-method output-assoc copy-back work when
//! the actual was a CALLER-LOCAL variable; the still-open follow-up was the
//! shape where the actual is an implicit-`this` property (the maintainer's
//! `fill(prop)`). `bind_assoc_param`/`bind_array_arg` accepted only a bare
//! `Ident` and resolved it with `resolve_hier_name` — an implicit-`this`
//! member `A` resolves to the bare name `A`, whose storage is actually at
//! `<handle>#A`, and an explicit `c.A` / `this.A` member access was not an
//! `Ident` at all, so the binder dropped the writeback. A queue property
//! actual already worked (its binder goes through the member-aware
//! `queue_actual_storage`). Expected lines are the reference simulator's.

use xezim::simulate;

/// The maintainer's literal shape: `fill(prop)` with `prop` an implicit-`this`
/// property, for both an associative-array member and a fixed-size member.
#[test]
fn implicit_this_property_output_actual_writes_back() {
    let src = r#"
module top;
  class C;
    int A[string];
    int F[2];
    function void fill(output int p[string], output int q[2]);
      p["x"] = 42;
      q[0] = 42;
    endfunction
    function void go();
      fill(A, F);   // implicit this.A / this.F as output actuals
    endfunction
  endclass
  initial begin
    C c = new;
    c.go();
    $display("RES assoc=%0d fixed=%0d", c.A.num(), c.F[0]);
    if (c.A["x"] == 42 && c.F[0] == 42) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 10)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = ["RES assoc=1 fixed=42", "TAG_PASS"];
    assert_eq!(out, want, "{out:?}");
}

/// An explicit `c.member` actual (called from outside the owning class) must
/// write back too, for assoc and fixed-array properties, plus the `ref` and
/// `inout` directions.
#[test]
fn explicit_member_and_ref_inout_directions() {
    let src = r#"
module top;
  class C;
    int A[string];
    int F[2];
    function void fout(output int p[string], output int q[2]);
      p["x"] = 42;
      q[0] = 42;
    endfunction
    function void fref(ref int p[string], ref int q[2]);
      p["y"] = 43;
      q[1] = 43;
    endfunction
    function void finout(inout int p[string]);
      p["z"] = 44;
    endfunction
  endclass
  initial begin
    C c = new;
    bit ok = 1;
    c.fout(c.A, c.F);
    c.fref(c.A, c.F);
    c.finout(c.A);
    if (!(c.A["x"] == 42)) ok = 0;
    if (!(c.A["y"] == 43)) ok = 0;
    if (!(c.A["z"] == 44)) ok = 0;
    if (!(c.F[0] == 42)) ok = 0;
    if (!(c.F[1] == 43)) ok = 0;
    if (ok) $display("TAG_PASS all=%0d,%0d,%0d %0d,%0d",
                     c.A["x"], c.A["y"], c.A["z"], c.F[0], c.F[1]);
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 10)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert!(
        out.contains(&"TAG_PASS all=42,43,44 42,43".to_string()),
        "{out:?}"
    );
}

/// The #277 caller-local shape must keep working (regression guard), and the
/// implicit-`this` property shape now matches it.
#[test]
fn caller_local_assoc_still_works_alongside_member() {
    let src = r#"
module top;
  class C;
    int A[string];
    function void fill(output int p[string]);
      p["x"] = 42;
    endfunction
    function void go();
      fill(A);
    endfunction
  endclass
  initial begin
    C c = new;
    int L[string];
    c.fill(L);       // #277 caller-local shape
    c.go();          // Point-2 implicit-this member shape
    $display("LOCAL=%0d MEMBER=%0d", L["x"], c.A["x"]);
    if (L["x"] == 42 && c.A["x"] == 42) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 10)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = ["LOCAL=42 MEMBER=42", "TAG_PASS"];
    assert_eq!(out, want, "{out:?}");
}
