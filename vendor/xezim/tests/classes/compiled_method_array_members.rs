//! class-perf task (a): FIXED-ARRAY member element read/write admission in
//! compiled class-method bodies.
//!
//! A bare (or `this.`-rooted / dotted) element access on a fixed-size
//! instance array member (`hist[i]`, `o.hist[k]`, `this.hist[j]`) inside a
//! method body lowers to `LoadCollElem` / `StoreCollElem`. The exec arms
//! resolve the store through `instance_assoc_member` — whose
//! `build_class_coll_index` already lists `array_properties` — to the SAME
//! `<handle>#hist` store the AST interpreter's `expr_assoc_name` uses, and
//! read/write `signals["<handle>#hist[i]"]` byte-identically. Before this
//! change these methods forced a whole-method `method_class_shadow` fallback
//! to the AST interpreter (measured 83% of a customer run's wall time).
//!
//! Admission is specifically to the ELEMENT set (`coll_elem_members`), NOT
//! the builtin-call set (`coll_members`): a fixed array has no legal
//! value-arg builtin methods (`hist.size()`, `push_back`, ...), so those
//! names must not join `CallCollMethod` admission.
//!
//! Fixed-array side effects (width fit, out-of-range read => 0, per-instance
//! isolation) are the interpreter's existing behavior; the compiled path
//! reproduces them through the same runtime funnel. Expectations are
//! reference-simulator-validated (the plain `module top` shape runs
//! byte-for-byte identically under the reference simulator, gate OFF, gate
//! ON, and compiled). The gate is forced ON so a plain `cargo test`
//! exercises the compiled path.
use xezim::simulate;

fn gate_on() -> bool {
    super::compiled_method_test_env::eager()
}

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

#[test]
fn fixed_array_member_element_ops_compile() {
    if !gate_on() {
        return;
    }
    let src = r#"
class Hist;
  int h[4];          // fixed-size instance array member
  int neg[3];        // [0:2] ascending bounds
  function automatic void reset();
    int i;
    for (i = 0; i < 4; i = i + 1) h[i] = 0;
  endfunction
  function automatic void bump(int i);
    if (i >= 0 && i < 4) h[i] = h[i] + 1;   // bare read+write, formal index
  endfunction
  function automatic int total();
    return h[0] + h[3] + h[2];              // bare reads, constant indices
  endfunction
  function automatic int sel(int i);
    return this.h[i];                       // explicit-this read
  endfunction
endclass

module top;
  integer i, oob, r1, r3, tot;
  initial begin
    Hist a, b;
    a = new(); b = new();
    a.reset();
    for (i = 0; i < 7; i = i + 1) a.bump(i % 5); // bumps h[1] an extra time
    b.reset(); b.bump(2); b.bump(2);
    tot = a.total();
    r3 = a.sel(3);
    oob = 0; if (a.h[9] == 0) oob = 1;      // out-of-range read => 0
    r1 = a.h[1];
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    // a.h after bumps: i in 0..6 -> i%5: 0,1,2,3,0,1,2, guarded by `i<4` so
    // the i%5==4 bump is discarded. So h[0]=2 (i=0,5), h[1]=2 (i=1,6),
    // h[2]=1 (i=2), h[3]=1 (i=3).
    // total = h[0]+h[3]+h[2] = 2+1+1 = 4.
    assert_eq!(u(&sim, "tot"), 4, "bare fixed-array reads (total)");
    assert_eq!(u(&sim, "r3"), 1, "explicit-this fixed-array read");
    assert_eq!(u(&sim, "r1"), 2, "bare fixed-array element");
    assert_eq!(u(&sim, "oob"), 1, "out-of-range fixed-array read is 0");
}

#[test]
fn fixed_array_member_two_instances_stay_isolated() {
    if !gate_on() {
        return;
    }
    let src = r#"
class Counter;
  int c[2];
  function automatic void inc(int i);
    c[i] = c[i] + 1;
  endfunction
  function automatic int get(int i);
    return c[i];
  endfunction
endclass
module top;
  integer i, c0a, c0b, c1a, tot;
  initial begin
    Counter x, y;
    x = new(); y = new();
    for (i = 0; i < 5; i = i + 1) x.inc(0);
    y.inc(1);
    c0a = x.get(0);
    c0b = y.get(0);
    c1a = y.get(1);
    tot = c0a * 100 + c0b * 10 + c1a; // 500 + 0 + 1 = 501
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    // x.c[0] = 5, y.c[0] = 0, y.c[1] = 1.
    assert_eq!(
        u(&sim, "tot"),
        501,
        "per-instance fixed-array storage isolation"
    );
}

#[test]
fn fixed_array_write_then_ast_and_compiled_agree() {
    if !gate_on() {
        return;
    }
    let src = r#"
class Buf;
  int d[3];
  function automatic void store(int i, int v);
    d[i] = v;                 // compiled write
  endfunction
  function automatic int load(int i);
    return d[i];               // compiled read
  endfunction
endclass
module top;
  integer i, v0, v1, v2, sum;
  initial begin
    Buf b1, b2;
    b1 = new(); b2 = new();
    b1.store(0, 10); b1.store(1, 20); b1.store(2, 30);
    b2.store(1, 7);
    v0 = b1.load(0); v1 = b1.load(1); v2 = b1.load(2);
    sum = v0 + v1 + v2 + b2.load(1); // 10+20+30+7 = 67
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    assert_eq!(
        u(&sim, "sum"),
        67,
        "compiled fixed-array store/load round trip"
    );
}
