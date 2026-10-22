//! §7.12.1 array locator methods on CLASS-MEMBER queues. The parse-level
//! receiver name is not the storage key: a member queue lives at
//! `<handle>#member` (this-relative bare name, or reached through object
//! handles for `obj.member`), and a subroutine-local queue under its
//! per-process rename. `locator_call` resolved neither, so every
//! `find*`/`unique`/`min`/`max` inside a class method returned EMPTY even
//! though `.size()` was right — and the module-scope `obj.member` form fell
//! through to a legacy path that ignored the `with` predicate entirely
//! (collected all non-zero elements). UVM's `grant_queued_locks` then
//! mistook a REQ entry for a leading lock: it granted arbitration no driver
//! asked for and drained the arbitration queue, starving the driver forever
//! (start_item returned with no grant). All expected values
//! reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_cql_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "test",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

#[test]
fn locators_on_member_queues_inside_class_methods() {
    let text = run(
        "in_method",
        r#"typedef enum { KIND_REQ, KIND_LOCK } kind_e;
class req_c;
  kind_e kind;
  int tag;
endclass
class holder_c;
  req_c arb_q[$];
  int iq[$];
  function void probe();
    int q1[$];
    req_c q2[$];
    q1 = arb_q.find_first_index(item) with (item.kind != KIND_LOCK);
    $display("T|neq-enum n=%0d first=%0d", q1.size(), q1.size() ? q1[0] : -1);
    q1 = arb_q.find_first_index(item) with (item.tag == 3);
    $display("T|eq-int n=%0d first=%0d", q1.size(), q1.size() ? q1[0] : -1);
    q1 = arb_q.find_first_index(item) with (1);
    $display("T|always n=%0d first=%0d size=%0d", q1.size(), q1.size() ? q1[0] : -1, arb_q.size());
    q1 = arb_q.find_first_index(item) with (item.tag == 999);
    $display("T|none n=%0d", q1.size());
    q1 = iq.find_first_index(item) with (item > 10);
    $display("T|int-q n=%0d first=%0d", q1.size(), q1.size() ? q1[0] : -1);
    q2 = arb_q.find_first(item) with (item.tag == 7);
    $display("T|ff n=%0d tag=%0d", q2.size(), q2.size() ? q2[0].tag : -1);
  endfunction
endclass
module test;
  holder_c h = new();
  initial begin
    req_c r;
    r = new(); r.kind = KIND_REQ;  r.tag = 3; h.arb_q.push_back(r);
    r = new(); r.kind = KIND_LOCK; r.tag = 7; h.arb_q.push_back(r);
    h.iq.push_back(5); h.iq.push_back(15);
    h.probe();
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|neq-enum n=1 first=0"),
        "enum != predicate:\n{text}"
    );
    assert!(
        text.contains("T|eq-int n=1 first=0"),
        "int == predicate:\n{text}"
    );
    assert!(
        text.contains("T|always n=1 first=0 size=2"),
        "with(1) sees elements:\n{text}"
    );
    assert!(
        text.contains("T|none n=0"),
        "impossible predicate yields empty:\n{text}"
    );
    assert!(
        text.contains("T|int-q n=1 first=1"),
        "int member queue:\n{text}"
    );
    assert!(
        text.contains("T|ff n=1 tag=7"),
        "find_first returns the element:\n{text}"
    );
}

#[test]
fn locator_through_object_handle_at_module_scope() {
    // `obj.member.find_first_index` — the receiver reaches the member queue
    // through an object handle from module scope (nested MemberAccess parse
    // shape). Must honour the predicate, not collect all non-zero handles.
    let text = run(
        "obj_member",
        r#"class req_c;
  int tag;
endclass
class holder_c;
  req_c arb_q[$];
endclass
module test;
  holder_c h = new();
  initial begin
    req_c r;
    int q1[$];
    r = new(); r.tag = 3; h.arb_q.push_back(r);
    r = new(); r.tag = 7; h.arb_q.push_back(r);
    q1 = h.arb_q.find_first_index(item) with (item.tag == 999);
    $display("T|none n=%0d", q1.size());
    q1 = h.arb_q.find_first_index(item) with (item.tag == 7);
    $display("T|seven n=%0d first=%0d", q1.size(), q1.size() ? q1[0] : -1);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|none n=0"),
        "impossible predicate yields empty:\n{text}"
    );
    assert!(
        text.contains("T|seven n=1 first=1"),
        "match is at index 1, not 0:\n{text}"
    );
}
