//! IEEE 1800-2023 §6.20.2 + §8.25: an `extends` clause that omits a parameter
//! argument leaves it at its DECLARED DEFAULT, on the task path as well as the
//! function path.
//!
//! `class d extends pbase;` — the shape a UVM base test uses
//! (`class base_test extends base_test_param;`) — is a specialization of
//! `pbase` with every parameter at its default: §8.25 makes a parameterized
//! class's generic declaration its default specialization, and §6.20.2 gives an
//! omitted parameter its declared default value. A method declared in `pbase`
//! and called on such an instance must therefore see `T` = `int` and `W` = 8.
//!
//! Before the fix the omitted arguments leaked their own bare NAMES into the
//! specialization signature (`pbase#(T,W)`), a key no resolution can bind, so a
//! reference to `T` landed on the unknown-type fallback. Measured without the
//! fix — `["TP|f 1 8", "TP|t 1 8", "TP|name logic 1", ...]` — i.e. `$bits(T)`
//! was 1 and the parameter named `logic` on BOTH paths. In a UVM testbench that
//! made `T::type_id::create` return null, so the run completed reporting
//! `UVM_ERROR: 0` while driving no transactions at all.
//! Cross-checked against the reference simulator.

#[test]
fn omitted_extends_arguments_use_declared_defaults_on_both_paths() {
    let sim = xezim::simulate(
        r#"
class pbase #(type T = int, int W = 8);
  virtual function void show_f();    $display("TP|f %0d %0d", $bits(T), W); endfunction
  virtual task         show_t();      $display("TP|t %0d %0d", $bits(T), W); endtask
  virtual function void show_name();  $display("TP|name %s %0d", $typename(T), $bits(T)); endfunction
endclass

// §6.20.2: no argument supplied -> T = int, W = 8.
class dimplicit extends pbase; endclass

// Control: an explicit specialization keeps its own arguments.
class dexplicit extends pbase #(bit [15:0], 4); endclass

module tb;
  dimplicit a;
  dexplicit b;
  initial begin
    a = new();
    b = new();
    a.show_f();
    a.show_t();
    a.show_name();
    b.show_f();
    b.show_t();
  end
endmodule
"#,
        10,
    )
    .expect("simulate");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    // omitted arguments -> declared defaults, on the FUNCTION path ...
    assert!(o.iter().any(|l| l == "TP|f 32 8"), "implicit/func: {o:?}");
    // ... and on the TASK path (§8.25 receiver specialization)
    assert!(o.iter().any(|l| l == "TP|t 32 8"), "implicit/task: {o:?}");
    // the type parameter names its declared default type, not `logic`
    assert!(
        o.iter().any(|l| l == "TP|name int 32"),
        "implicit/type: {o:?}"
    );
    // the same class spelled explicitly is unaffected
    assert!(o.iter().any(|l| l == "TP|f 16 4"), "explicit/func: {o:?}");
    assert!(o.iter().any(|l| l == "TP|t 16 4"), "explicit/task: {o:?}");
}

const LEAVES: &str = r#"
class pbase #(type T = int, int W = 8, int D = W*2);
  virtual function void show_f(string who); $display("T|%s F %0d %0d %0d", who, $bits(T), W, D); endfunction
  virtual task show_t(string who); $display("T|%s T %0d %0d %0d", who, $bits(T), W, D); endtask
endclass
class dimp extends pbase; endclass
class dpar #(int X = 1) extends pbase; endclass
class dmid extends pbase #(bit [15:0]); endclass
class dmid2 extends pbase #(byte, 4); endclass
class dgrand extends dimp; endclass
module tb;
  dimp a; dpar b; dmid c; dmid2 e; dgrand g;
  initial begin
    a = new; b = new; c = new; e = new; g = new;
    a.show_f("imp"); a.show_t("imp");
    b.show_f("par"); b.show_t("par");
    c.show_f("mid"); c.show_t("mid");
    e.show_f("mid2"); e.show_t("mid2");
    g.show_f("grand"); g.show_t("grand");
  end
endmodule
"#;

fn leaves() -> Vec<String> {
    let sim = xezim::simulate(LEAVES, 10).expect("simulate");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// The first two parameters through an implicit, explicit, partial and
/// grandchild specialization, on both method paths.
#[test]
fn leaf_specializations_see_type_and_value_parameters() {
    let o = leaves();
    for (who, bits, w) in [
        ("imp", 32, 8),
        ("mid", 16, 8),
        ("mid2", 8, 4),
        ("grand", 32, 8),
    ] {
        for path in ["F", "T"] {
            let want = format!("T|{who} {path} {bits} {w} ");
            assert!(o.iter().any(|l| l.starts_with(&want)), "{want}: {o:?}");
        }
    }
}

/// §6.20.2: a default may name an earlier parameter; `D = W*2` follows `W`.
#[test]
fn dependent_parameter_default_follows_the_specialization() {
    let o = leaves();
    for line in [
        "T|imp F 32 8 16",
        "T|imp T 32 8 16",
        "T|mid F 16 8 16",
        "T|mid2 F 8 4 8",
        "T|mid2 T 8 4 8",
        "T|grand T 32 8 16",
    ] {
        assert!(o.iter().any(|l| l == line), "missing `{line}`: {o:?}");
    }
}

/// A parameterized class that extends `pbase` without arguments still gets
/// `T` = `int`.
#[test]
fn parameterized_leaf_keeps_the_base_type_default() {
    let o = leaves();
    for path in ["F", "T"] {
        let want = format!("T|par {path} 32 8 ");
        assert!(o.iter().any(|l| l.starts_with(&want)), "{want}: {o:?}");
    }
}
