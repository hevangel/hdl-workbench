//! §13.5.3 (1800-2023): an omitted argument binds its formal's DEFAULT the way
//! a supplied actual binds, including for aggregate formals — fixed arrays,
//! dynamic arrays, queues and associative arrays — and for `output`/`inout`
//! formals, whose default names the variable the formal is copied back to.
//! An array default was stored as one packed value, so `v[0]` read a bit of
//! it; and an input array formal stayed registered after the call, so a
//! same-named formal of a later call behaved differently ("warming").
//! Issue #327 (reported by Dragon-Git); the expected lines are the reference
//! simulator's.

use std::process::Command;

/// Run `src` (top module `top`) and return its display lines: everything on
/// stdout but the banner and the end-of-run note.
fn run_lines(tag: &str, src: &str, env: &[(&str, &str)]) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!(
        "xezim_{}_{}_{}",
        "subdflt",
        tag,
        std::process::id()
    ));
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

/// The issue's reproducer: cold and after a bound-actual call.
#[test]
fn issue_327_reproducer() {
    let src = r#"// trunc.sv
module top;
    function automatic void fill(output byte v[2]);
        v[0] = 8'hA5;
        v[1] = 8'h5A;
    endfunction

    function automatic void probe(byte v[2] = '{8'h00, 8'h00});
        fill(v);
        $display("   %%h   : %h %h        (expect a5 5a)", v[0], v[1]);
        $display("   %%0d  : %0d %0d      (expect 165 90)", v[0], v[1]);
        $display("   ===  : %0d %0d", v[0] === 8'hA5, v[1] === 8'h5A);
    endfunction

    function automatic void warm(byte v[2]);
        fill(v);
    endfunction

    initial begin
        $display("cold (first call of the simulation):");
        probe();

        $display("after a bound-actual call to the same callee:");
        begin byte q[2]; warm(q); end
        probe();
    end
endmodule
"#;
    check(
        "issue_327_reproducer",
        src,
        &[
            "cold (first call of the simulation):",
            "   %h   : a5 5a        (expect a5 5a)",
            "   %0d  : -91 90      (expect 165 90)",
            "   ===  : 1 1",
            "after a bound-actual call to the same callee:",
            "   %h   : a5 5a        (expect a5 5a)",
            "   %0d  : -91 90      (expect 165 90)",
            "   ===  : 1 1",
        ],
    );
}

#[test]
fn default_array_formal_reads_its_elements() {
    let src = r#"// default-valued array formal read directly, no output call
module top;
  function automatic void probe(byte v[2] = '{8'hA5, 8'h5A});
    $display("T|read %h %h", v[0], v[1]);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_array_formal_reads_its_elements",
        src,
        &["T|read a5 5a"],
    );
}

#[test]
fn default_array_formal_element_writes() {
    let src = r#"// write the defaulted formal's element directly (no call)
module top;
  function automatic void probe(byte v[2] = '{8'h00, 8'h00});
    v[0] = 8'hA5; v[1] = 8'h5A;
    $display("T|elem %h %h", v[0], v[1]);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_array_formal_element_writes",
        src,
        &["T|elem a5 5a"],
    );
}

#[test]
fn default_array_formal_size_foreach_and_p() {
    let src = r#"// default-valued fixed array formal: $size / foreach / %p
module top;
  function automatic void probe(int v[3] = '{1, 2, 3});
    int s = 0;
    foreach (v[i]) s += v[i];
    $display("T|size=%0d sum=%0d p=%p", $size(v), s, v);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_array_formal_size_foreach_and_p",
        src,
        &["T|size=3 sum=6 p='{1, 2, 3}"],
    );
}

#[test]
fn default_array_formal_passed_to_inout() {
    let src = r#"module top;
  function automatic void fill(inout byte v[2]); v[0] = v[0] + 8'hA0; v[1] = 8'h5A; endfunction
  function automatic void probe(byte v[2] = '{8'h05, 8'h00});
    fill(v); $display("T|inout %h %h", v[0], v[1]);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_array_formal_passed_to_inout",
        src,
        &["T|inout a5 5a"],
    );
}

#[test]
fn default_array_formal_passed_to_ref() {
    let src = r#"module top;
  function automatic void fill(ref byte v[2]); v[0] = 8'hA5; v[1] = 8'h5A; endfunction
  function automatic void probe(byte v[2] = '{8'h00, 8'h00});
    fill(v); $display("T|ref %h %h", v[0], v[1]);
  endfunction
  initial probe();
endmodule
"#;
    check("default_array_formal_passed_to_ref", src, &["T|ref a5 5a"]);
}

#[test]
fn default_string_array_formal_passed_to_output() {
    let src = r#"module top;
  function automatic void fill(output string v[2]); v[0] = "AA"; v[1] = "BB"; endfunction
  function automatic void probe(string v[2] = '{"", ""});
    fill(v); $display("T|string %s %s", v[0], v[1]);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_string_array_formal_passed_to_output",
        src,
        &["T|string AA BB"],
    );
}

#[test]
fn default_empty_dynamic_array_formal() {
    let src = r#"// syoscb shape: dynamic array formal with default {} and an output dynamic array
module top;
  function automatic void get_q(output string o[]); o = new[2]; o[0] = "DUT"; o[1] = "REF"; endfunction
  function automatic void probe(string queues[] = {});
    get_q(queues); $display("T|dyn size=%0d [0]=%s [1]=%s", queues.size(), queues[0], queues[1]);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_empty_dynamic_array_formal",
        src,
        &["T|dyn size=2 [0]=DUT [1]=REF"],
    );
}

#[test]
fn default_empty_queue_formal() {
    let src = r#"module top;
  function automatic void get_q(output int o[$]); o = {32'hDEADBEEF, 32'h12345678}; endfunction
  function automatic void probe(int q[$] = {});
    get_q(q); $display("T|queue size=%0d %h %h", q.size(), q[0], q[1]);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_empty_queue_formal",
        src,
        &["T|queue size=2 deadbeef 12345678"],
    );
}

#[test]
fn default_array_formal_of_a_task() {
    let src = r#"module top;
  task automatic fill(output int v[2]); v[0] = 32'hDEADBEEF; v[1] = 32'h12345678; endtask
  task automatic probe(int v[2] = '{0, 0});
    fill(v); $display("T|task %h %h", v[0], v[1]);
  endtask
  initial probe();
endmodule
"#;
    check(
        "default_array_formal_of_a_task",
        src,
        &["T|task deadbeef 12345678"],
    );
}

#[test]
fn default_array_formal_of_a_method() {
    let src = r#"class C;
  function void fill(output shortint v[2]); v[0] = 16'hBEEF; v[1] = 16'h1234; endfunction
  function void probe(shortint v[2] = '{0, 0});
    fill(v); $display("T|class %h %h", v[0], v[1]);
  endfunction
endclass
module top;
  initial begin C c = new; c.probe(); end
endmodule
"#;
    check(
        "default_array_formal_of_a_method",
        src,
        &["T|class beef 1234"],
    );
}

#[test]
fn default_associative_formal() {
    let src = r#"module top;
  function automatic void fill(output int o[string]); o["A"] = 7; o["B"] = 8; endfunction
  function automatic void probe(int m[string] = '{default: 0});
    fill(m); $display("T|assoc size=%0d A=%0d", m.size(), m["A"]);
  endfunction
  initial probe();
endmodule
"#;
    check("default_associative_formal", src, &["T|assoc size=2 A=7"]);
}

#[test]
fn default_unpacked_struct_formal() {
    let src = r#"module top;
  typedef struct { byte a; int b; } s_t;
  function automatic void probe(s_t s = '{8'hA5, 32'hDEADBEEF});
    $display("T|struct %h %h", s.a, s.b);
  endfunction
  initial probe();
endmodule
"#;
    check(
        "default_unpacked_struct_formal",
        src,
        &["T|struct a5 deadbeef"],
    );
}

/// The clause's own example, `task t1(output logic [7:0] o = a)`.
#[test]
fn output_formal_default_copies_back() {
    let src = r#"// §13.5.3 example shape: an output formal whose default is a variable copies back to it
module top;
  logic [7:0] a;
  task automatic t1(output logic [7:0] o = a); o = 8'h3C; endtask
  initial begin a = 0; t1(); $display("T|outdefault a=%h", a); end
endmodule
"#;
    check(
        "output_formal_default_copies_back",
        src,
        &["T|outdefault a=3c"],
    );
}

#[test]
fn output_array_formal_default_copies_back() {
    let src = r#"module top;
  byte g[2];
  function automatic void t1(output byte o[2] = g); o[0] = 8'hA5; o[1] = 8'h5A; endfunction
  initial begin t1(); $display("T|outdefault_arr %h %h", g[0], g[1]); end
endmodule
"#;
    check(
        "output_array_formal_default_copies_back",
        src,
        &["T|outdefault_arr a5 5a"],
    );
}

/// Task, function and method output/inout defaults, with and without an actual.
#[test]
fn output_and_inout_defaults() {
    let src = r#"class C;
  int p;
  function void get(output int o = p); o = 41; endfunction
endclass
module top;
  integer b;
  int g;
  int arr[3];
  logic [7:0] a;
  task k(input integer i = 0, output integer j = b); j = i; endtask
  task automatic t_inout(inout int x = g); x += 5; endtask
  function automatic void f_out(output int o = arr[1]); o = 77; endfunction
  function automatic int f_ret(output logic [7:0] o = a); o = 8'h3C; return 9; endfunction
  initial begin
    C c = new;
    b = 99; k(); $display("T|k() b=%0d", b);
    k(5); $display("T|k(5) b=%0d", b);
    g = 10; t_inout(); t_inout(); $display("T|inout g=%0d", g);
    f_out(); $display("T|elem arr=%0d %0d %0d", arr[0], arr[1], arr[2]);
    a = 0; $display("T|ret %0d a=%h", f_ret(), a);
    c.get(); $display("T|method p=%0d", c.p);
  end
endmodule
"#;
    check(
        "output_and_inout_defaults",
        src,
        &[
            "T|k() b=0",
            "T|k(5) b=5",
            "T|inout g=20",
            "T|elem arr=0 77 0",
            "T|ret 9 a=3c",
            "T|method p=41",
        ],
    );
}
