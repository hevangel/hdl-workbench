//! class-perf Step 9b: builtin-collection calls on class member collections
//! lower to `CallCollMethod` inside compiled class-method bodies. The exec
//! arm re-enters the interpreter's own `eval_builtin_method` — on the
//! interpreter-resolved store (`<h>#member` scoped via
//! `handle_collection_name`, or the bare name whose §8.10 rewrite
//! (`instance_assoc_member`) resolves it with the live `this`) — so results
//! are byte-identical to the AST path by delegation.
//!
//! Covers the admission matrix: bare member queue (size/insert/delete/
//! push_back/pop_back), `this.`-rooted dotted, dotted through an
//! intermediate handle, member assoc (exists/num/delete), string keys, and
//! the static member queue (bare receiver only). Excluded shapes keep the
//! AST path: a user method named like a builtin wins (pre-existing funnel
//! precedence — the CallCollMethod runtime has the same `exec_method_call`
//! net), a ref-key method (`first`) is not on the admitted list, and a
//! dotted static stays AST (broken zero path in `exec_method_call`).
//!
//! Expectations are reference-simulator-validated (the method_vm29 /
//! method_vm30 shapes and their reference-simulator runs print identical
//! values). The gate is forced ON so a plain `cargo test` exercises the
//! compiled path.
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

/// The method_vm29 shape: every receiver shape the compiler admits, one
/// value per family (queue ops, this.-rooted ops, assoc ops, dotted-through-
/// intermediate ops). Reference simulator: TAG r1=6 r2=43 r3=103 r4=4,
/// qsz=3 q0=99 q1=20, aax=0 aay=2, iqsz=2 iak=42, TAG_PASS.
#[test]
fn collection_calls_all_receiver_shapes() {
    if !gate_on() {
        return;
    }
    let src = r#"
class inner_t;
  int iq[$];
  int ia[string];
  int q2[$];
endclass

class outer_t;
  int q[$];
  int aa[string];
  int ai[int];
  inner_t in;

  function void seed();
    q.push_back(10);
    q.push_back(20);
    q.push_back(30);
    aa["x"] = 1;
    aa["y"] = 2;
    ai[7] = 77;
  endfunction

  function int bare_ops();
    int s;
    s = q.size();
    q.insert(1, 99);
    q.delete(0);
    return s + q.size();
  endfunction

  function int this_ops();
    int s;
    s = this.q.size();
    this.q.push_back(40);
    return s + this.q.pop_back();
  endfunction

  function int assoc_ops();
    int e;
    if (aa.exists("x")) e += 100;
    e += aa.num();
    aa.delete("x");
    if (aa.exists("x")) e += 1000;
    e += ai.exists(7);
    return e;
  endfunction

  function int inner_ops();
    int s;
    if (in == null) return -1;
    in.iq.push_back(5);
    in.iq.push_back(6);
    s = in.iq.size();
    in.ia["k"] = 42;
    s += in.ia.exists("k");
    s += in.ia.num();
    return s;
  endfunction
endclass

module top;
  outer_t o;
  int r1, r2, r3, r4;
  int qsz, q0, q1, aax, aay, iqsz, iak;
  initial begin
    o = new();
    o.in = new();
    o.seed();
    r1 = o.bare_ops();
    r2 = o.this_ops();
    r3 = o.assoc_ops();
    r4 = o.inner_ops();
    qsz = o.q.size();
    q0 = o.q[0];
    q1 = o.q[1];
    aax = o.aa["x"];
    aay = o.aa["y"];
    iqsz = o.in.iq.size();
    iak = o.in.ia["k"];
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    // bare_ops: 3 + (insert + delete(idx) shift) => 3 + 3 = 6
    assert_eq!(u(&sim, "r1"), 6);
    // this_ops: 3 + push_back(40) then pop_back -> 40 => 43
    assert_eq!(u(&sim, "r2"), 43);
    // assoc_ops: exists(x)=1 -> +100, num=2, delete(x), exists=0, ai.exists=1
    assert_eq!(u(&sim, "r3"), 103);
    // inner_ops: size 2 + exists 1 + num 1 = 4
    assert_eq!(u(&sim, "r4"), 4);
    // Final storage state.
    assert_eq!(u(&sim, "qsz"), 3);
    assert_eq!(u(&sim, "q0"), 99);
    assert_eq!(u(&sim, "q1"), 20);
    assert_eq!(u(&sim, "aax"), 0);
    assert_eq!(u(&sim, "aay"), 2);
    assert_eq!(u(&sim, "iqsz"), 2);
    assert_eq!(u(&sim, "iak"), 42);
}

/// The method_vm30 shape: a STATIC member queue called with bare-receiver
/// builtins inside instance methods (the `uvm_objection::m_scheduled_list`
/// mutation shape). Store resolution re-runs the interpreter's §8.10 rewrite
/// (per-spec key), so the static store is shared across instances exactly as
/// the AST path. Reference: sz0=3, sz1=2 e0=1 e1=3, sz2=0.
#[test]
fn static_member_queue_bare_builtins() {
    if !gate_on() {
        return;
    }
    let src = r#"
class obj_t;
  static int sq[$];
  function void seed();
    sq.push_back(1);
    sq.push_back(2);
    sq.push_back(3);
  endfunction
  function void drain_idx(int idx);
    sq.delete(idx);
  endfunction
  function void clear_all();
    sq.delete();
  endfunction
  function int sz();
    return sq.size();
  endfunction
endclass

module top;
  initial begin
    obj_t a, b;
    a = new();
    a.seed();
    a.drain_idx(1);
    b = new();
    b.clear_all();
  end
endmodule
"#;
    // The static store is shared across instances, so observe the sequence
    // through separate runs (each ends with a distinct observable copy):
    // stage 1 — seed + drain, store has [1, 3];
    let sim =
        simulate(src.replace("b.clear_all();", "").as_str(), 100).expect("simulation should run");
    let _ = &sim;
    // Final states are observed through dedicated result copies below.
    let src3 = r#"
class obj_t;
  static int sq[$];
  function void seed();
    sq.push_back(1);
    sq.push_back(2);
    sq.push_back(3);
  endfunction
  function void drain_idx(int idx);
    sq.delete(idx);
  endfunction
  function int sz();
    return sq.size();
  endfunction
  function int e0();
    return obj_t::sq[0];
  endfunction
  function int e1();
    return obj_t::sq[1];
  endfunction
endclass

module top;
  int w0, w1, w2, v0, v1;
  initial begin
    obj_t a;
    a = new();
    a.seed();
    w0 = a.sz();
    a.drain_idx(1);
    w1 = a.sz();
    v0 = a.e0();
    v1 = a.e1();
  end
endmodule
"#;
    let sim3 = simulate(src3, 100).expect("simulation should run");
    assert_eq!(u(&sim3, "w0"), 3);
    assert_eq!(u(&sim3, "w1"), 2);
    assert_eq!(u(&sim3, "v0"), 1);
    assert_eq!(u(&sim3, "v1"), 3);
    // And the full run (clear_all at the end) empties the shared store.
    let src4 = r#"
class obj_t;
  static int sq[$];
  function void seed();
    sq.push_back(1);
    sq.push_back(2);
    sq.push_back(3);
  endfunction
  function void drain_idx(int idx);
    sq.delete(idx);
  endfunction
  function void clear_all();
    sq.delete();
  endfunction
  function int sz();
    return sq.size();
  endfunction
endclass

module top;
  int w2;
  initial begin
    obj_t a, b;
    a = new();
    a.seed();
    a.drain_idx(1);
    b = new();
    b.clear_all();
    w2 = a.sz();
  end
endmodule
"#;
    let sim4 = simulate(src4, 100).expect("simulation should run");
    assert_eq!(u(&sim4, "w2"), 0);
}

/// Excluded shape: a USER method named like a builtin (`size`) on a class
/// whose instance the builtin path could otherwise capture. The AST funnel's
/// user-method precedence must be preserved — the compiled CallCollMethod
/// arm never fires for it (the receiver is a plain class HANDLE, not a
/// member collection), and `exec_method_call` inside the exec arm keeps the
/// same precedence for any shape that reaches it.
#[test]
fn user_method_named_like_builtin_wins() {
    if !gate_on() {
        return;
    }
    let src = r#"
class pair_t;
  int q[$];
  function new();
    q.push_back(7);
  endfunction
  function int size();
    return 4242;
  endfunction
endclass

module top;
  int s;
  initial begin
    pair_t p;
    p = new();
    s = p.size();
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation should run");
    assert_eq!(u(&sim, "s"), 4242);
}
