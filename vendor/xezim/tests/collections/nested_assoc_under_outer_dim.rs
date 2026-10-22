//! §7.4.5 / §7.8.1: an associative array of associative arrays under a
//! non-associative outer dimension (`int v[2][string][string]`, `[][string]
//! [string]`, `[$][string][string]`, a class property, an automatic local, a
//! struct member). Elaboration registered element arrays only when the
//! innermost dimension was the sole collection, and the compound-key path
//! (`v[0]["A"]["B"]`) required an associative ROOT, so the write was lost and
//! the read gave x. Issue #328 (reported by Dragon-Git); the expected lines
//! are the reference simulator's.

use std::process::Command;

/// Run `src` (top module `top`) and return its display lines: everything on
/// stdout but the banner and the end-of-run note.
fn run_lines(tag: &str, src: &str, env: &[(&str, &str)]) -> Vec<String> {
    let dir =
        std::env::temp_dir().join(format!("xezim_{}_{}_{}", "nestaa", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("top.sv");
    std::fs::write(&sv, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .args(["--no-cache", "-s", "top"])
        .arg(&sv)
        .current_dir(&dir)
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    text.lines()
        .filter(|l| {
            !(l.starts_with('[')
                || l.starts_with("=== xezim")
                || l.starts_with("git ")
                || l.starts_with("Simulation finished")
                || l.starts_with("$finish"))
        })
        .map(str::to_string)
        .collect()
}

/// Both method tiers: interpreted, and compiled on the first call.
fn check(tag: &str, src: &str, want: &[&str]) {
    assert_eq!(run_lines(tag, src, &[]), want, "{tag}");
    assert_eq!(
        run_lines(&format!("{tag}_t0"), src, &[("XEZIM_METHOD_TIER", "0")]),
        want,
        "{tag} (XEZIM_METHOD_TIER=0)"
    );
}

/// The issue's reproducer.
#[test]
fn issue_328_reproducer() {
    let src = r#"// nested_assoc.sv
module top;

    int checked  = 0;
    int failures = 0;

    function automatic void report(string what, bit ok);
        checked++;
        if (ok) $display("PASS: %s", what);
        else begin
            failures++;
            $display("FAIL: %s", what);
        end
    endfunction

    // one associative level under a fixed-size outer dimension: control
    int one_level[2][string];

    // two associative levels under a fixed-size outer dimension
    int two_fixed[2][string][string];

    // two associative levels under a dynamic outer dimension
    int two_dyn[][string][string];

    // two associative levels under an associative outer dimension: control
    int two_assoc[int][string][string];

    initial begin
        one_level[0]["A"] = 7;
        report("one associative level under fixed[2]: size()==1",
               one_level[0].size() == 1 && one_level[0].exists("A") && one_level[0]["A"] == 7);

        two_fixed[0]["A"]["B"] = 7;
        $display("       two_fixed[0].size()=%0d exists(A)=%0d", two_fixed[0].size(),
                 two_fixed[0].exists("A"));
        report("two associative levels under fixed[2]: outer entry created",
               two_fixed[0].exists("A"));
        report("two associative levels under fixed[2]: inner value readable",
               two_fixed[0].exists("A") && two_fixed[0]["A"].exists("B")
               && two_fixed[0]["A"]["B"] == 7);

        two_dyn = new[2];
        two_dyn[0]["A"]["B"] = 7;
        $display("       two_dyn[0].size()=%0d exists(A)=%0d", two_dyn[0].size(),
                 two_dyn[0].exists("A"));
        report("two associative levels under dyn[]: outer entry created",
               two_dyn[0].exists("A"));
        report("two associative levels under dyn[]: inner value readable",
               two_dyn[0].exists("A") && two_dyn[0]["A"].exists("B")
               && two_dyn[0]["A"]["B"] == 7);

        two_assoc[0]["A"]["B"] = 7;
        report("two associative levels under an associative outer: readable",
               two_assoc[0].exists("A") && two_assoc[0]["A"]["B"] == 7);

        $display("=== %0d check(s), %0d failure(s) ===", checked, failures);
    end

endmodule
"#;
    check(
        "issue_328_reproducer",
        src,
        &[
            "PASS: one associative level under fixed[2]: size()==1",
            "       two_fixed[0].size()=1 exists(A)=1",
            "PASS: two associative levels under fixed[2]: outer entry created",
            "PASS: two associative levels under fixed[2]: inner value readable",
            "       two_dyn[0].size()=1 exists(A)=1",
            "PASS: two associative levels under dyn[]: outer entry created",
            "PASS: two associative levels under dyn[]: inner value readable",
            "PASS: two associative levels under an associative outer: readable",
            "=== 6 check(s), 0 failure(s) ===",
        ],
    );
}

#[test]
fn two_levels_under_a_fixed_dimension() {
    let src = r#"module top;
  int v[2][string][string];
  initial begin
    v[0]["A"]["B"] = 7;
    $display("T|fixed2 size=%0d exists=%0d inner=%0d val=%0d", v[0].size(), v[0].exists("A"), v[0]["A"].size(), v[0]["A"]["B"]);
  end
endmodule
"#;
    check(
        "two_levels_under_a_fixed_dimension",
        src,
        &["T|fixed2 size=1 exists=1 inner=1 val=7"],
    );
}

#[test]
fn three_levels_under_a_fixed_dimension() {
    let src = r#"module top;
  int v[2][string][string][string];
  initial begin
    v[1]["A"]["B"]["C"] = 9;
    $display("T|fixed3 size=%0d val=%0d", v[1].size(), v[1]["A"]["B"]["C"]);
  end
endmodule
"#;
    check(
        "three_levels_under_a_fixed_dimension",
        src,
        &["T|fixed3 size=1 val=9"],
    );
}

#[test]
fn int_keys_under_a_fixed_dimension() {
    let src = r#"module top;
  int v[3][int][int];
  initial begin
    v[2][5][6] = 11; v[2][5][7] = 12;
    $display("T|intkeys size=%0d inner=%0d val=%0d", v[2].size(), v[2][5].size(), v[2][5][7]);
  end
endmodule
"#;
    check(
        "int_keys_under_a_fixed_dimension",
        src,
        &["T|intkeys size=1 inner=2 val=12"],
    );
}

#[test]
fn two_levels_under_a_queue_dimension() {
    let src = r#"module top;
  int q[$][string][string];
  int e[string][string];
  initial begin
    q.push_back(e);
    q[0]["A"]["B"] = 7;
    $display("T|queue size=%0d val=%0d", q[0].size(), q[0]["A"]["B"]);
  end
endmodule
"#;
    check(
        "two_levels_under_a_queue_dimension",
        src,
        &["T|queue size=1 val=7"],
    );
}

#[test]
fn two_levels_under_a_dynamic_dimension() {
    let src = r#"module top;
  int d[][string][string];
  initial begin
    d = new[2];
    d[1]["A"]["B"] = 7;
    $display("T|dyn size=%0d val=%0d", d[1].size(), d[1]["A"]["B"]);
  end
endmodule
"#;
    check(
        "two_levels_under_a_dynamic_dimension",
        src,
        &["T|dyn size=1 val=7"],
    );
}

#[test]
fn two_levels_in_a_class_property() {
    let src = r#"class C; int v[2][string][string]; endclass
module top;
  initial begin
    C c = new;
    c.v[0]["A"]["B"] = 7;
    $display("T|classprop size=%0d val=%0d", c.v[0].size(), c.v[0]["A"]["B"]);
  end
endmodule
"#;
    check(
        "two_levels_in_a_class_property",
        src,
        &["T|classprop size=1 val=7"],
    );
}

#[test]
fn foreach_over_an_inner_level() {
    let src = r#"module top;
  int v[2][string][string];
  initial begin
    v[0]["A"]["B"] = 7; v[0]["A"]["C"] = 8; v[0]["Z"]["B"] = 9;
    foreach (v[0][k]) $display("T|foreach key=%s n=%0d", k, v[0][k].size());
  end
endmodule
"#;
    check(
        "foreach_over_an_inner_level",
        src,
        &["T|foreach key=A n=2", "T|foreach key=Z n=1"],
    );
}

#[test]
fn two_levels_in_an_automatic_local() {
    let src = r#"module top;
  initial begin
    automatic int v[2][string][string];
    v[0]["A"]["B"] = 7;
    $display("T|local size=%0d val=%0d", v[0].size(), v[0]["A"]["B"]);
  end
endmodule
"#;
    check(
        "two_levels_in_an_automatic_local",
        src,
        &["T|local size=1 val=7"],
    );
}

#[test]
fn two_levels_in_a_struct_member_of_an_array() {
    let src = r#"module top;
  typedef struct { int m[string][string]; } s_t;
  s_t a[2];
  initial begin
    a[0].m["A"]["B"] = 7;
    $display("T|structelem size=%0d val=%0d", a[0].m.size(), a[0].m["A"]["B"]);
  end
endmodule
"#;
    check(
        "two_levels_in_a_struct_member_of_an_array",
        src,
        &["T|structelem size=1 val=7"],
    );
}

#[test]
fn associative_of_queue_under_a_fixed_dimension() {
    let src = r#"// assoc of queue under a fixed dim
module top;
  int v[2][string][$];
  initial begin
    v[0]["A"].push_back(7);
    $display("T|aofq size=%0d qsize=%0d val=%0d", v[0].size(), v[0]["A"].size(), v[0]["A"][0]);
  end
endmodule
"#;
    check(
        "associative_of_queue_under_a_fixed_dimension",
        src,
        &["T|aofq size=1 qsize=1 val=7"],
    );
}

/// exists/size/num on inner levels, a missing leaf, a struct variable, and
/// automatic locals that are fixed arrays of queues / associative arrays.
#[test]
fn nested_methods_and_local_arrays_of_collections() {
    let src = r#"module top;
  int v[2][string][string];
  int d[][string][string];
  typedef struct { int m[string][string]; } s_t;
  s_t s1;
  initial begin
    v[0]["A"]["B"] = 7;
    $display("T|v ex=%0d exB=%0d sz=%0d num=%0d val=%0d", v[0].exists("A"), v[0]["A"].exists("B"), v[0]["A"].size(), v[0].num(), v[0]["A"]["B"]);
    $display("T|v missing=%0d other=%0d", v[0]["A"]["Q"], v[1].size());
    d = new[2];
    d[1]["A"]["B"] = 7;
    $display("T|d ex=%0d sz=%0d val=%0d", d[1].exists("A"), d[1].size(), d[1]["A"]["B"]);
    s1.m["A"]["B"] = 5;
    $display("T|s1 sz=%0d val=%0d", s1.m.size(), s1.m["A"]["B"]);
  end
  initial begin
    automatic int q3[3][$];
    automatic int a2[2][int];
    automatic int v1[2][string];
    q3[1].push_back(4);
    a2[1][9] = 2;
    v1[0]["A"] = 3;
    #1;
    $display("T|localq size=%0d val=%0d", q3[1].size(), q3[1][0]);
    $display("T|locala size=%0d val=%0d", a2[1].size(), a2[1][9]);
    $display("T|local1 size=%0d val=%0d", v1[0].size(), v1[0]["A"]);
  end
endmodule
"#;
    check(
        "nested_methods_and_local_arrays_of_collections",
        src,
        &[
            "T|v ex=1 exB=1 sz=1 num=1 val=7",
            "T|v missing=0 other=0",
            "T|d ex=1 sz=1 val=7",
            "T|s1 sz=1 val=5",
            "T|localq size=1 val=4",
            "T|locala size=1 val=2",
            "T|local1 size=1 val=3",
        ],
    );
}
