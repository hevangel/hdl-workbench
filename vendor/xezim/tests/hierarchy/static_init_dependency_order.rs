//! IEEE 1800-2017 §6.21 — a MODULE-SCOPE variable whose initializer merely
//! READS a deferred initializer must be re-issued after it.
//!
//! Elaboration const-folds a module-scope initializer that calls a sim-time
//! system function (`$sformatf("%m")`, `$urandom_range`, ...) to 0/"" and
//! `defer_static_syscall_inits` re-issues it as a time-0 `static_init_blocks`
//! assignment.  Re-issuing only the declarations whose OWN initializer text
//! contains the call leaves every declaration that depends on one behind: it was
//! folded while its dependency still held the placeholder, so
//!
//! ```systemverilog
//! string a = $sformatf("%m");   // re-issued  -> "tb.u_leaf"
//! string b = {a, ".chain"};     // never re-issued -> ".chain"
//! int    r = $urandom_range(1,9999);
//! int    d = r + 1;             // never re-issued -> 1
//! ```
//!
//! The dependency pass in `walk_module_static_inits` closes the re-issue set
//! transitively over same-scope reads and emits in declaration order, so each
//! reader is assigned after the names it reads.  Plain constant folding (the
//! `*_const` control) must stay exactly as it was — those declarations are not
//! re-issued at all.  Verified against the reference simulator.

fn lines(src: &str, top: &str) -> Vec<String> {
    xezim::simulate_multi(
        &[src.to_string()],
        1000,
        Some(top),
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &[],
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
    )
    .expect("sim")
    .output
    .iter()
    .map(|o| o.message.clone())
    .collect()
}

const SRC: &str = r#"
module j_leaf;
  string a_pctm   = $sformatf("%m");
  string b_chain  = {a_pctm, ".chain"};   // one hop
  string t1       = $sformatf("%m");
  string t2       = {t1, ".b"};           // two hops
  string t3       = {t2, ".c"};           // three hops
  int    r1       = $urandom_range(1, 9999);
  int    d1       = r1 + 1;
  int    r2       = $urandom_range(1, 9999);
  int    dsum     = r1 + r2;
  string base     = "B";
  string from_const = {base, $sformatf("%m")}; // const prefix + call
  string late;
  initial late = a_pctm;                       // read from an initial block
endmodule

module j_leaf_const;
  string base     = "Z";
  string b_chain  = {base, "-tail"};        // pure folding — must stay folded
  int    k        = 7;
  int    d1       = k + 1;
endmodule

module tb;
  j_leaf       u_leaf();
  j_leaf_const u_const();
  initial begin
    #1;
    $display("CHAIN=%s",   u_leaf.b_chain);
    $display("T3=%s",      u_leaf.t3);
    $display("BASE=%s",    u_leaf.a_pctm);
    $display("D1=%0d",     u_leaf.d1);
    $display("DSUM=%0d",   u_leaf.dsum);
    $display("FROM_CONST=%s", u_leaf.from_const);
    $display("LATE=%s",    u_leaf.late);
    $display("C_CHAIN=%s", u_const.b_chain);
    $display("C_D1=%0d",   u_const.d1);
    $finish;
  end
endmodule
"#;

/// A declaration that reads a deferred declaration is re-issued after it —
/// directly, transitively, and for integer chains off `$urandom_range`.
#[test]
fn module_scope_reader_of_deferred_init_is_re_issued() {
    let out = lines(SRC, "tb").join("\n");
    assert!(out.contains("BASE=tb.u_leaf"), "{out}");
    assert!(out.contains("CHAIN=tb.u_leaf.chain"), "{out}");
    assert!(out.contains("T3=tb.u_leaf.b.c"), "{out}");
    assert!(
        out.contains("FROM_CONST=Btb.u_leaf"),
        "const prefix + sim-time call regressed: {out}"
    );
    assert!(out.contains("LATE=tb.u_leaf"), "{out}");
    for key in ["D1", "DSUM"] {
        let v: i64 = out
            .lines()
            .find(|l| l.starts_with(&format!("{key}=")))
            .and_then(|l| l.trim_start_matches(&format!("{key}=")).parse().ok())
            .unwrap_or_else(|| panic!("{key} printed: {out}"));
        assert!(v >= 2, "{key} kept the folded value: {v}");
    }
}

/// Declarations with no sim-time dependency anywhere keep elaboration's
/// compile-time folding — the dependency pass must not turn them into
/// time-0 assignments (their values are identical either way, but the control
/// pins that plain initializers are untouched and still ordered by text).
#[test]
fn plain_module_scope_folding_is_unchanged() {
    let out = lines(SRC, "tb").join("\n");
    assert!(out.contains("C_CHAIN=Z-tail"), "{out}");
    assert!(out.contains("C_D1=8"), "{out}");
}