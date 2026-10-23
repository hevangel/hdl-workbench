//! §13.5.2 + §8.10: a queue/dynamic-array `ref` FORMAL lives under its bare
//! name, so nested calls whose formals share a name — and callees that
//! declare a LOCAL named like the caller's actual — must not corrupt each
//! other.
//!
//! Two independent defects, both reference-verified:
//!  * the inner call's formal overwrote the outer frame's formal storage,
//!    so the outer function kept appending to whatever the inner call left
//!    behind (a stale leading element of the wrong element type);
//!  * the writeback and the callee-frame restore raced: writing back first
//!    let the restore of a same-named callee LOCAL wipe the result, while
//!    restoring first discarded the formal before it could be read. The
//!    fix captures the formal, restores, then applies.
//!
//! Both shapes appear together in a register model's nested
//! `get_full_hdl_path`, where they left every backdoor access with an empty
//! path list.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_qrfs_{}_{}", name, std::process::id()));
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
fn nested_same_named_ref_queue_formals_stay_isolated() {
    // `outer_walk`'s formal `items` calls `inner_walk`, whose formal is ALSO
    // `items` and which additionally declares its own local `sub`. The outer
    // result must contain exactly its own two entries.
    let text = run(
        "nested_formals",
        r#"class walker_c;
  function void inner_walk(ref string items[$]);
    items.delete();
    items.push_back("base.unit");
  endfunction
  function void outer_walk(ref string items[$]);
    string sub[$];
    inner_walk(sub);
    $display("T|inner gave %0d '%s'", sub.size(), sub.size() ? sub[0] : "");
    foreach (sub[j])
      items.push_back({sub[j], ".", "LEAF"});
  endfunction
endclass
module test;
  walker_c w = new();
  initial begin
    string got[$];
    w.outer_walk(got);
    $display("T|outer n=%0d", got.size());
    foreach (got[i]) $display("T|out[%0d]='%s'", i, got[i]);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|inner gave 1 'base.unit'"),
        "inner writeback:\n{text}"
    );
    assert!(
        text.contains("T|outer n=1"),
        "no phantom leading entry:\n{text}"
    );
    assert!(
        text.contains("T|out[0]='base.unit.LEAF'"),
        "composed entry:\n{text}"
    );
}

#[test]
fn callee_local_named_like_the_actual_does_not_wipe_writeback() {
    // The callee declares a local with the SAME name as the caller's actual
    // (`slots`), so the callee-frame restore must not clobber the ref
    // writeback.
    let text = run(
        "local_shadow",
        r#"class filler_c;
  function void fill(ref int slots[$]);
    int scratch[$];
    scratch.push_back(-1);
    slots.delete();
    slots.push_back(7);
    slots.push_back(9);
  endfunction
endclass
class driver_c;
  filler_c f = new();
  function void go();
    int slots[$];
    f.fill(slots);
    $display("T|n=%0d", slots.size());
    foreach (slots[i]) $display("T|s[%0d]=%0d", i, slots[i]);
  endfunction
endclass
module test;
  driver_c d = new();
  initial begin d.go(); $finish; end
endmodule
"#,
    );
    assert!(
        text.contains("T|n=2"),
        "writeback survives the frame restore:\n{text}"
    );
    assert!(
        text.contains("T|s[0]=7") && text.contains("T|s[1]=9"),
        "values:\n{text}"
    );
}

#[test]
fn class_member_collection_element_new_uses_declared_element_type() {
    // §8.10: `store[key] = new(key)` inside a method must construct the
    // MEMBER's declared element type, not a same-named collection declared
    // elsewhere in the design (which made a pool hand back an object of the
    // pool's own class).
    let text = run(
        "elem_new",
        r#"class leaf_c;
  string nm;
  function new(string name=""); nm = name; endfunction
  virtual function string kind(); return "leaf"; endfunction
endclass
class other_c;
  string nm;
  function new(string name=""); nm = name; endfunction
  virtual function string kind(); return "other"; endfunction
endclass
class store_c #(type T=int);
  T store[string];
  function T get(string key);
    if (!store.exists(key)) store[key] = new(key);
    return store[key];
  endfunction
endclass
module test;
  // a same-named collection of a DIFFERENT element class, declared globally
  other_c store[string];
  store_c #(leaf_c) s = new();
  initial begin
    leaf_c e;
    other_c o = new("global");
    store["X"] = o;
    e = s.get("RTL");
    $display("T|null=%0d nm='%s' kind='%s'", (e == null), (e == null) ? "" : e.nm,
             (e == null) ? "" : e.kind());
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|null=0 nm='RTL' kind='leaf'"),
        "member's declared element type wins:\n{text}"
    );
}
