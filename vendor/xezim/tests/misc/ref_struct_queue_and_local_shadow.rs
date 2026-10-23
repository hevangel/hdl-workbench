//! Audit follow-ups to the class-member scope / queue-formal work.
//!
//! * §8.10: a method-LOCAL collection shadows a same-named class member,
//!   including for element construction (`m[k] = new()`), and the member is
//!   unaffected afterwards.
//! * §13.5.2: a formal carrying an unpacked QUEUE dimension is a COLLECTION
//!   of structs, not a member-wise struct formal. Binding it as the latter
//!   made `ref rec_t items[$]` a scalar, so the callee's `push_back`es never
//!   reached the caller; the formal also has to record its ELEMENT type so
//!   struct members are written as leaves (size came back right with every
//!   member blank).
//!
//! Both predate the register-model backdoor work and were found auditing
//! it. All expected values reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_rsqls_{}_{}", name, std::process::id()));
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
fn method_local_collection_shadows_same_named_member() {
    let text = run(
        "local_shadow_elem",
        r#"class leaf_c;  function string kind(); return "leaf"; endfunction endclass
class alt_c;   function string kind(); return "alt";  endfunction endclass
class holder_c;
  leaf_c m[string];
  function string via_member();
    m["k"] = new();
    return m["k"].kind();
  endfunction
  function string via_local();
    alt_c m[string];
    m["k"] = new();
    return m["k"].kind();
  endfunction
endclass
module test;
  holder_c h = new();
  initial begin
    $display("T|member=%s", h.via_member());
    $display("T|local=%s", h.via_local());
    $display("T|member_after=%s", h.via_member());
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|member=leaf"),
        "member element class:\n{text}"
    );
    assert!(
        text.contains("T|local=alt"),
        "local shadows the member:\n{text}"
    );
    assert!(
        text.contains("T|member_after=leaf"),
        "member unaffected by the local:\n{text}"
    );
}

#[test]
fn inherited_and_static_member_collections_construct_their_element_type() {
    let text = run(
        "inherited_elem",
        r#"class leaf_c;  function string kind(); return "leaf"; endfunction endclass
class base_c;
  leaf_c tab[string];
  static leaf_c stab[string];
endclass
class deriv_c extends base_c;
  function string fill();
    tab["a"] = new();
    stab["b"] = new();
    return {tab["a"].kind(), "/", stab["b"].kind()};
  endfunction
endclass
module test;
  deriv_c d = new();
  initial begin
    $display("T|inh=%s", d.fill());
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|inh=leaf/leaf"),
        "inherited + static members:\n{text}"
    );
}

#[test]
fn ref_queue_of_unpacked_structs_writes_back() {
    let text = run(
        "ref_struct_q",
        r#"typedef struct { string nm; int v; } rec_t;
class b_c;
  function void inner(ref rec_t items[$]);
    rec_t t;
    t.nm = "inner"; t.v = 5;
    items.push_back(t);
  endfunction
endclass
module test;
  b_c b = new();
  initial begin
    rec_t got[$];
    string nm;
    b.inner(got);
    nm = got.size() ? got[0].nm : "";
    $display("T|n=%0d nm='%s' v=%0d", got.size(), nm, got.size() ? got[0].v : -1);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|n=1"), "size written back:\n{text}");
    assert!(
        text.contains("nm='inner' v=5"),
        "struct members written back:\n{text}"
    );
}

#[test]
fn nested_ref_struct_queues_compose() {
    let text = run(
        "nested_struct_q",
        r#"typedef struct { string nm; int v; } rec_t;
class b_c;
  function void inner(ref rec_t items[$]);
    rec_t t;
    items.delete();
    t.nm = "inner"; t.v = 5;
    items.push_back(t);
  endfunction
  function void outer(ref rec_t items[$]);
    rec_t sub[$];
    inner(sub);
    foreach (sub[i]) begin
      rec_t t;
      t.nm = {sub[i].nm, "+out"};
      t.v = sub[i].v + 1;
      items.push_back(t);
    end
  endfunction
endclass
module test;
  b_c b = new();
  initial begin
    rec_t got[$];
    string nm;
    b.outer(got);
    nm = got.size() ? got[0].nm : "";
    $display("T|n=%0d v=%0d", got.size(), got.size() ? got[0].v : -1);
    $display("T|nm_has_chain=%0d", nm.len() >= 9);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|n=1 v=6"), "nested compose:\n{text}");
    assert!(
        text.contains("T|nm_has_chain=1"),
        "member string survives nesting:\n{text}"
    );
}
