//! IEEE 1800-2017 §23.11 + §6.21 — a sim-time system call in a MODULE-SCOPE
//! variable initializer of a `bind`-inserted instance.
//!
//! `$sformatf("%m")` (and `$urandom*`, `$test$plusargs`, `$time`, ...) in a
//! module-scope initializer must run at time 0 under the DECLARING instance's
//! scope. Elaboration const-folds such calls to 0/"" and
//! `defer_static_syscall_inits` re-issues them as time-0 `static_init_blocks`
//! assignments — but that pass walked AST module instantiations only, and a
//! `bind`-inserted instance is not an AST instantiation in its host (it is
//! injected from `ElaboratedModule::resolved_binds`). A bound harness's
//! `string inst_path = $sformatf("%m")` therefore kept the folded "" and every
//! config_db path built from it lost its instance hierarchy. Verified against
//! the reference simulator.

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

const HARNESS: &str = r#"
module pctm_harness;
  string inst_path = $sformatf("%m");
  string pctm_direct = $sformatf("%m");
  int    urand_init  = $urandom_range(1, 9999);
  pctm_child u_nested(); // a module instantiated UNDER the bound instance
  function automatic string get_inst_hier(string suffix);
    return {inst_path, ".", suffix};
  endfunction
  initial begin
    $display("BOUND_HIER=%s", get_inst_hier(""));
    $display("BOUND_DIRECT=%s", pctm_direct);
    $display("BOUND_RAND=%0d", urand_init);
    $display("BOUND_NESTED=%s", u_nested.child_path);
  end
endmodule

module pctm_child;
  string child_path = $sformatf("%m");
endmodule

module pctm_plain;
  string inst_path = $sformatf("%m");
  initial $display("PLAIN=%s", inst_path);
endmodule

module host;
  pctm_plain u_plain();
endmodule

bind host pctm_harness v_harness (.*);
bind host pctm_child   u_child (.*);
"#;

/// The bound harness's module-scope `%m` initializers must report the bound
/// instance's own path, not "" — and a module instantiated *under* the bound
/// instance must resolve too.
#[test]
fn bind_inserted_module_scope_syscall_init_uses_bound_scope() {
    let src = format!(
        "{HARNESS}\nmodule test;\n  host u_host();\n  initial begin #1; \
         $display(\"BOUND_CHILD=%s\", u_host.u_child.child_path); end\nendmodule\n"
    );
    let out = lines(&src, "test").join("\n");
    assert!(out.contains("BOUND_HIER=test.u_host.v_harness."), "{out}");
    assert!(out.contains("BOUND_DIRECT=test.u_host.v_harness"), "{out}");
    assert!(out.contains("BOUND_CHILD=test.u_host.u_child"), "{out}");
    assert!(
        out.contains("BOUND_NESTED=test.u_host.v_harness.u_nested"),
        "{out}"
    );
    assert!(out.contains("PLAIN=test.u_host.u_plain"), "{out}");
    let rand: i64 = out
        .lines()
        .find(|l| l.starts_with("BOUND_RAND="))
        .and_then(|l| l.trim_start_matches("BOUND_RAND=").parse().ok())
        .expect("BOUND_RAND printed");
    assert!(
        (1..=9999).contains(&rand),
        "$urandom_range folded to {rand}"
    );
}

/// Two hosts bound from one `bind` directive must each get their own path —
/// the re-issued initializer is per-instance, not shared.
#[test]
fn bind_inserted_static_init_is_per_instance() {
    let src = format!(
        "{HARNESS}\nmodule test;\n  host u_a();\n  host u_b();\n  initial begin #1; \
         $display(\"BOUND_CHILD_A=%s\", u_a.u_child.child_path); \
         $display(\"BOUND_CHILD_B=%s\", u_b.u_child.child_path); end\nendmodule\n"
    );
    let out = lines(&src, "test").join("\n");
    assert!(out.contains("BOUND_HIER=test.u_a.v_harness."), "{out}");
    assert!(out.contains("BOUND_HIER=test.u_b.v_harness."), "{out}");
    assert!(out.contains("BOUND_DIRECT=test.u_a.v_harness"), "{out}");
    assert!(out.contains("BOUND_DIRECT=test.u_b.v_harness"), "{out}");
    assert!(out.contains("BOUND_CHILD_A=test.u_a.u_child"), "{out}");
    assert!(out.contains("BOUND_CHILD_B=test.u_b.u_child"), "{out}");
    assert!(
        out.contains("BOUND_NESTED=test.u_a.v_harness.u_nested"),
        "{out}"
    );
    assert!(
        out.contains("BOUND_NESTED=test.u_b.v_harness.u_nested"),
        "{out}"
    );
}
#[test]
fn elaborated_init_generate() {
    let source = r#"module leaf #(int P=2);string path=$sformatf("%m");string val=$sformatf("P=%0d",P);endmodule
module top;
for(genvar g=0;g<2;g++)begin:bank
leaf#(g+3) u();
end
if(0)begin:disabled_branch leaf u();end
initial begin #1;
$display("T|gen %s %s %s %s",bank[0].u.path,bank[0].u.val,bank[1].u.path,bank[1].u.val);
end endmodule
"#;
    let got = lines(source, "top");
    assert!(
        got.iter()
            .any(|line| line == "T|gen top.bank[0].u P=3 top.bank[1].u P=4"),
        "{got:?}"
    );
}

#[test]
fn elaborated_init_instance_array() {
    let source = r#"module leaf;string path=$sformatf("%m");endmodule
module top;leaf u[1:0]();initial begin #1;
$display("T|array %s %s",u[0].path,u[1].path);end endmodule
"#;
    let got = lines(source, "top");
    assert!(
        got.iter().any(|line| line == "T|array top.u[0] top.u[1]"),
        "{got:?}"
    );
}

#[test]
fn elaborated_init_interface() {
    let source = r#"interface iface;string path=$sformatf("%m");endinterface
module top;iface a();iface b();initial begin #1;
$display("T|if %s %s",a.path,b.path);end endmodule
"#;
    let got = lines(source, "top");
    assert!(got.iter().any(|line| line == "T|if top.a top.b"), "{got:?}");
}

#[test]
fn elaborated_init_program() {
    let source = r#"program punit;string path=$sformatf("%m");endprogram
module top;punit a();punit b();initial begin #1;
$display("T|prog %s %s",a.path,b.path);end endmodule
"#;
    let got = lines(source, "top");
    assert!(
        got.iter().any(|line| line == "T|prog top.a top.b"),
        "{got:?}"
    );
}

#[test]
fn elaborated_init_once() {
    let source = r#"package p;int calls=0;function int bump();calls++;return calls;endfunction endpackage
module leaf;string val=$sformatf("%0d",p::bump());endmodule
module top;leaf a();leaf b();initial begin #1;
$display("T|once %0d",p::calls);
end endmodule
"#;
    let got = lines(source, "top");
    assert!(got.iter().any(|line| line == "T|once 2"), "{got:?}");
}

#[test]
fn elaborated_init_bind_generate() {
    let source = r#"module host;endmodule
module harness #(int P=7);string path=$sformatf("%m");string val=$sformatf("P=%0d",P);int rnd=$urandom_range(1,9999);endmodule
bind host harness#(8) h();
module top;
for(genvar g=0;g<2;g++)begin:bank host u();end
initial begin #1;
$display("T|bindgen %s %s %s %s",bank[0].u.h.path,bank[0].u.h.val,bank[1].u.h.path,bank[1].u.h.val);
$display("T|rand %0d %0d",bank[0].u.h.rnd,bank[1].u.h.rnd);
end endmodule
"#;
    let got = lines(source, "top");
    assert!(
        got.iter()
            .any(|line| line == "T|bindgen top.bank[0].u.h P=8 top.bank[1].u.h P=8"),
        "{got:?}"
    );
    let rand = got
        .iter()
        .find(|line| line.starts_with("T|rand "))
        .expect("random values printed");
    for value in rand.split_whitespace().skip(1) {
        assert!((1..=9999).contains(&value.parse::<u32>().expect("random integer")));
    }
}
