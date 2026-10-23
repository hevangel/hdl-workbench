//! §7.9.9 / §7.10: a whole associative array copied into a class property
//! from a plain module, local or formal variable (and back), and into a queue
//! element (`push_back`/`push_front`/`insert`, `q = {src}`, and the hand-off
//! of a popped element). Each path recognized an associative array on only
//! one side and fell to a scalar assignment that copied no entries.
//! Issue #331 (reported by Dragon-Git); the expected lines are the reference
//! simulator's.

use std::process::Command;

/// Run `src` (top module `top`) and return its display lines: everything on
/// stdout but the banner and the end-of-run note.
fn run_lines(tag: &str, src: &str, env: &[(&str, &str)]) -> Vec<String> {
    let dir =
        std::env::temp_dir().join(format!("xezim_{}_{}_{}", "aacopy", tag, std::process::id()));
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
fn issue_331_reproducer() {
    let src = r#"// aa_copy.sv
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

    int src[string];                 // one-level source
    int src2[string][string];        // two-level source

    class holder;
        int m[string];
        int m2[string][string];
    endclass

    holder h;

    int q1[$][string];
    int q2[$][string][string];

    int f1[2][string];               // control: fixed-size array element
    int d1[][string];                // control: dynamic array element
    int p1[string];                  // control: plain variable

    typedef struct { int m[string]; } st_t;
    st_t st;                         // control: struct member

    initial begin
        src["A"]  = 7;
        src2["A"]["B"] = 7;

        // controls: these are correct today
        p1 = src;
        report("control: assign to a plain variable",
               p1.size() == 1 && p1["A"] == 7);

        f1[0] = src;
        report("control: assign to a fixed-size array element",
               f1[0].size() == 1 && f1[0]["A"] == 7);

        d1 = new[1];
        d1[0] = src;
        report("control: assign to a dynamic array element",
               d1[0].size() == 1 && d1[0]["A"] == 7);

        st.m = src;
        report("control: assign to a struct member",
               st.m.size() == 1 && st.m["A"] == 7);

        // the two failing shapes
        h = new();
        h.m = src;
        $display("       class property    : size=%0d exists(A)=%0d", h.m.size(), h.m.exists("A"));
        report("assign to a class property keeps the entries",
               h.m.size() == 1 && h.m["A"] == 7);

        h = new();
        h.m2 = src2;
        $display("       class property, 2 : size=%0d", h.m2.size());
        report("assign to a two-level class property keeps the entries",
               h.m2.size() == 1 && h.m2["A"]["B"] == 7);

        q1.push_back(src);
        $display("       queue element     : size=%0d exists(A)=%0d", q1[0].size(), q1[0].exists("A"));
        report("push_back of an associative array keeps the entries",
               q1.size() == 1 && q1[0].size() == 1 && q1[0]["A"] == 7);

        q2.push_back(src2);
        $display("       queue element, 2  : size=%0d", q2[0].size());
        report("push_back of a two-level associative array keeps the entries",
               q2.size() == 1 && q2[0].size() == 1);

        $display("=== %0d check(s), %0d failure(s) ===", checked, failures);
    end

endmodule
"#;
    check(
        "issue_331_reproducer",
        src,
        &[
            "PASS: control: assign to a plain variable",
            "PASS: control: assign to a fixed-size array element",
            "PASS: control: assign to a dynamic array element",
            "PASS: control: assign to a struct member",
            "       class property    : size=1 exists(A)=1",
            "PASS: assign to a class property keeps the entries",
            "       class property, 2 : size=1",
            "PASS: assign to a two-level class property keeps the entries",
            "       queue element     : size=1 exists(A)=1",
            "PASS: push_back of an associative array keeps the entries",
            "       queue element, 2  : size=1",
            "PASS: push_back of a two-level associative array keeps the entries",
            "=== 8 check(s), 0 failure(s) ===",
        ],
    );
}

#[test]
fn variable_to_class_property() {
    let src = r#"class H; int m[string]; endclass
module top;
  int src[string];
  initial begin
    H h = new; src["A"] = 7; src["B"] = 8;
    h.m = src;
    $display("T|classprop size=%0d A=%0d", h.m.size(), h.m["A"]);
  end
endmodule
"#;
    check(
        "variable_to_class_property",
        src,
        &["T|classprop size=2 A=7"],
    );
}

#[test]
fn variable_to_int_keyed_class_property() {
    let src = r#"class H; string m[int]; endclass
module top;
  string src[int];
  initial begin
    H h = new; src[3] = "x"; src[9] = "y";
    h.m = src;
    $display("T|intkey size=%0d 9=%s", h.m.size(), h.m[9]);
  end
endmodule
"#;
    check(
        "variable_to_int_keyed_class_property",
        src,
        &["T|intkey size=2 9=y"],
    );
}

#[test]
fn class_property_to_variable() {
    let src = r#"class H; int m[string]; endclass
module top;
  int dst[string];
  initial begin
    H a = new; a.m["A"] = 7;
    dst = a.m;
    $display("T|prop2var size=%0d A=%0d", dst.size(), dst["A"]);
  end
endmodule
"#;
    check(
        "class_property_to_variable",
        src,
        &["T|prop2var size=1 A=7"],
    );
}

#[test]
fn formal_to_property_inside_a_method() {
    let src = r#"class H; int m[string];
  function void set(int s[string]); this.m = s; endfunction
  function void set2(int s[string]); m = s; endfunction
endclass
module top;
  int src[string];
  initial begin
    H h = new; H g = new; src["A"] = 7;
    h.set(src); g.set2(src);
    $display("T|method this.m=%0d bare m=%0d", h.m.size(), g.m.size());
  end
endmodule
"#;
    check(
        "formal_to_property_inside_a_method",
        src,
        &["T|method this.m=1 bare m=1"],
    );
}

#[test]
fn property_to_property() {
    let src = r#"class H; int m[string]; endclass
module top;
  initial begin
    H a = new; H b = new; a.m["A"] = 7; a.m["B"] = 8;
    b.m = a.m;
    $display("T|prop2prop size=%0d", b.m.size());
  end
endmodule
"#;
    check("property_to_property", src, &["T|prop2prop size=2"]);
}

#[test]
fn variable_to_static_property() {
    let src = r#"class H; static int m[string]; endclass
module top;
  int src[string];
  initial begin
    src["A"] = 7; H::m = src;
    $display("T|staticprop size=%0d", H::m.size());
  end
endmodule
"#;
    check("variable_to_static_property", src, &["T|staticprop size=1"]);
}

#[test]
fn push_back_of_an_associative_array() {
    let src = r#"module top;
  int src[string]; int q[$][string];
  initial begin
    src["A"] = 7; q.push_back(src);
    $display("T|push_back size=%0d A=%0d", q[0].size(), q[0]["A"]);
  end
endmodule
"#;
    check(
        "push_back_of_an_associative_array",
        src,
        &["T|push_back size=1 A=7"],
    );
}

#[test]
fn queue_concatenation_of_an_associative_array() {
    let src = r#"module top;
  int src[string]; int q[$][string];
  initial begin
    src["A"] = 7; q = {src};
    $display("T|qconcat qsize=%0d size=%0d", q.size(), q[0].size());
  end
endmodule
"#;
    check(
        "queue_concatenation_of_an_associative_array",
        src,
        &["T|qconcat qsize=1 size=1"],
    );
}

#[test]
fn queue_element_assignment() {
    let src = r#"module top;
  int src[string]; int q[$][string]; int e[string];
  initial begin
    src["A"] = 7; q.push_back(e); q[0] = src;
    $display("T|qelem_assign size=%0d A=%0d", q[0].size(), q[0]["A"]);
  end
endmodule
"#;
    check(
        "queue_element_assignment",
        src,
        &["T|qelem_assign size=1 A=7"],
    );
}

#[test]
fn push_front_insert_and_pop_back() {
    let src = r#"module top;
  int src[string]; int q[$][string]; int got[string];
  initial begin
    src["A"] = 7; q.push_front(src); q.insert(0, src);
    got = q.pop_back();
    $display("T|qops qsize=%0d q0=%0d got=%0d", q.size(), q[0].size(), got.size());
  end
endmodule
"#;
    check(
        "push_front_insert_and_pop_back",
        src,
        &["T|qops qsize=1 q0=1 got=1"],
    );
}

#[test]
fn push_back_into_a_class_property_queue() {
    let src = r#"class H; int m[$][string]; endclass
module top;
  int src[string];
  initial begin
    H h = new; src["A"] = 7; h.m.push_back(src);
    $display("T|class_q_of_assoc size=%0d", h.m[0].size());
  end
endmodule
"#;
    check(
        "push_back_into_a_class_property_queue",
        src,
        &["T|class_q_of_assoc size=1"],
    );
}
