//! §13.5.2 `ref` formals ALIAS the actual — reference-validated (H3 audit).
//!
//! The legacy model was copy-in at call, copy-out at return. Three visible
//! divergences: (1) a parallel process observing the actual mid-call never
//! saw the callee's writes; (2) the callee never saw the observer's writes;
//! (3) the return copy-out CLOBBERED whatever the observer wrote during the
//! call. Aliasing is applied where the actual is a plain module-visible
//! variable; caller-frame locals and aggregate elements keep the legacy
//! copy path (still correct for them at return, though not mid-call).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Reference: at5=10 (write visible mid-call), mid=20 (observer's write
/// visible after resume), g=21 (no copy-out clobber of the observer's 20),
/// ali=5 / g=5 (two ref formals of the same variable truly alias).
#[test]
fn ref_writes_and_reads_alias_the_actual() {
    let src = r#"
module top;
  int g;
  int seen_at5 = -1, mid = -1, ali = -1;

  task automatic bump(ref int r);
    r = 10;
    #10;
    mid = r;
    r = r + 1;
  endtask

  task automatic alias2(ref int a, ref int b);
    a = 5;
    ali = b;
  endtask

  initial begin
    g = 1;
    fork
      bump(g);
      begin #5 seen_at5 = g; g = 20; end
    join
    alias2(g, g);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    assert_eq!(
        u(&sim, "seen_at5"),
        10,
        "callee write visible to a parallel observer mid-call"
    );
    assert_eq!(
        u(&sim, "mid"),
        20,
        "observer write visible to the callee after resume"
    );
    assert_eq!(u(&sim, "g"), 5, "no copy-out clobber; alias2 leaves g=5");
    assert_eq!(
        u(&sim, "ali"),
        5,
        "double-ref of one variable: b reads a's write"
    );
}

/// Reference: foo=8 (formal named like the actual still aliases), loc=8
/// (a caller-LOCAL actual keeps the legacy copy path and still updates),
/// g2=99 / chained=99 (a ref passed on as ref reaches the original storage).
#[test]
fn ref_alias_edge_shapes() {
    let src = r#"
module top;
  int foo;
  int g2;
  int chained_saw = -1;
  int loc_out = -1;

  task automatic t_same(ref int foo);
    foo = 7;
    #1 foo = foo + 1;
  endtask

  task automatic inner(ref int r);
    r = 99;
    #1 chained_saw = r;
  endtask
  task automatic outer(ref int r);
    inner(r);
  endtask

  task automatic t_local();
    int loc;
    loc = 3;
    t_same(loc);
    loc_out = loc;
  endtask

  initial begin
    foo = 1;
    t_same(foo);
    g2 = 0;
    fork
      outer(g2);
      begin #1 ; end
    join
    t_local();
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    assert_eq!(
        u(&sim, "foo"),
        8,
        "same-named formal/actual aliases the storage"
    );
    assert_eq!(
        u(&sim, "g2"),
        99,
        "chained ref writes the original variable"
    );
    assert_eq!(u(&sim, "chained_saw"), 99);
    assert_eq!(
        u(&sim, "loc_out"),
        8,
        "caller-local actual: legacy copy path still lands"
    );
}

/// §13.5.2: `ref arr[i]` freezes the ELEMENT identity at call time — a later
/// change to `i` must not retarget the alias. Reference: a1=43 a3=0 (the old
/// copy-out re-evaluated the index at return and wrote arr[3]).
#[test]
fn ref_element_identity_frozen_at_call() {
    let src = r#"
module top;
  int arr [0:3];
  int idx = 1;
  task automatic bump(ref int r);
    r = 42;
    #5;
    r = r + 1;
  endtask
  initial begin
    fork
      bump(arr[idx]);
      begin #2 idx = 3; end
    join
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    let a1 = sim
        .get_signal("arr[1]")
        .and_then(|v| v.to_u64())
        .unwrap_or(999);
    let a3 = sim
        .get_signal("arr[3]")
        .and_then(|v| v.to_u64())
        .unwrap_or(999);
    assert_eq!(
        (a1, a3),
        (43, 0),
        "writes stay on the element captured at call time"
    );
}

/// J2 remnant pin: a user function call inside a DIMENSION width is constant-
/// evaluated (reference: wa=8 wb=16 WA=8).
#[test]
fn function_call_in_dimension_width() {
    let src = r#"
module top;
  function automatic int width_of(input int sel);
    return sel == 0 ? 8 : 16;
  endfunction
  logic [width_of(0)-1:0] a;
  logic [width_of(1)-1:0] b;
  localparam int WA = $bits(a);
  int r = 0;
  initial r = $bits(a) * 1000000 + $bits(b) * 1000 + WA;
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let r = sim.get_signal("r").and_then(|v| v.to_u64()).unwrap_or(0);
    assert_eq!(r, 8016008, "wa=8 wb=16 WA=8");
}
