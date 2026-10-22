//! §15.3/§15.4 — every way of constructing a `semaphore` or `mailbox` must
//! yield a live object. Cross-checked against the reference simulator: all
//! forms below construct in both the LRM and the reference.
//!
//! A bare `new` (no parentheses) arrives as an identifier rather than a
//! call, and a block-local declaration initializer takes the declaration
//! path rather than the assignment path; both used to skip the built-in
//! constructor, leaving a mailbox null ("null mailbox handle") and a
//! semaphore without its keys.

use xezim::simulate;

fn out(src: &str) -> String {
    match simulate(src, 1000) {
        Ok(sim) => sim
            .output
            .iter()
            .map(|o| o.message.clone())
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => format!("simulate failed: {e}"),
    }
}

/// `put(1); get(x)` through a mailbox declared/constructed as `decl` (module
/// scope) and `setup` (first statements of the initial block).
fn mailbox_roundtrip(decl: &str, setup: &str) -> String {
    out(&format!(
        "module t;\n  {decl}\n  int x;\n  initial begin\n    {setup}\n    mb.put(7); mb.get(x);\n    $display(\"X=%0d\", x);\n  end\nendmodule\n"
    ))
}

#[test]
fn module_mailbox_new_with_bound() {
    let o = mailbox_roundtrip("mailbox mb = new(2);", "");
    assert!(o.contains("X=7"), "{o}");
}

#[test]
fn module_mailbox_new_with_empty_parens() {
    let o = mailbox_roundtrip("mailbox mb = new();", "");
    assert!(o.contains("X=7"), "{o}");
}

#[test]
fn module_parameterized_mailbox_new_with_bound() {
    let o = mailbox_roundtrip("mailbox #(int) mb = new(2);", "");
    assert!(o.contains("X=7"), "{o}");
}

#[test]
fn procedural_mailbox_new_with_bound() {
    let o = mailbox_roundtrip("mailbox mb;", "mb = new(2);");
    assert!(o.contains("X=7"), "{o}");
}

#[test]
fn module_mailbox_bare_new() {
    let o = mailbox_roundtrip("mailbox mb = new;", "");
    assert!(o.contains("X=7"), "{o}");
}

#[test]
fn procedural_mailbox_bare_new() {
    let o = mailbox_roundtrip("mailbox mb;", "mb = new;");
    assert!(o.contains("X=7"), "{o}");
}

#[test]
fn block_local_mailbox_initializer() {
    let o = out("module t;
  int x;
  initial begin
    automatic mailbox #(int) mb = new(2);
    mb.put(4); mb.get(x);
    $display(\"X=%0d\", x);
  end
endmodule
");
    assert!(o.contains("X=4"), "{o}");
}

#[test]
fn module_semaphore_initializer_holds_its_key() {
    let o = out(
        "module t;\n  semaphore sem = new(1);\n  initial $display(\"K=%0d\", sem.try_get(1));\nendmodule\n",
    );
    assert!(o.contains("K=1"), "{o}");
}

#[test]
fn procedural_semaphore_holds_its_key() {
    let o = out(
        "module t;\n  semaphore sem;\n  initial begin\n    sem = new(1);\n    $display(\"K=%0d\", sem.try_get(1));\n  end\nendmodule\n",
    );
    assert!(o.contains("K=1"), "{o}");
}

#[test]
fn block_local_semaphore_initializer_holds_its_key() {
    let o = out(
        "module t;\n  initial begin\n    automatic semaphore sem = new(1);\n    $display(\"K=%0d\", sem.try_get(1));\n  end\nendmodule\n",
    );
    assert!(o.contains("K=1"), "{o}");
}
