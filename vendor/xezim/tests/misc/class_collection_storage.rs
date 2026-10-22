//! Storage-resolution fixes for class-member collections, found while
//! bringing up register-model backdoor access (all expected values
//! reference-verified):
//! - a string-keyed pool whose KEY is a type parameter bound to `string`
//!   must key consistently (value-width once picked numeric vs string keys);
//! - `T x = new;` / `pool[key] = new(key)` where the element type resolves
//!   to a NESTED specialization (`q#(string)`);
//! - struct-element dyn arrays as class members: element leaves seed on
//!   `new[n]`, member writes/reads resolve `<h>#arr[i].field`, whole-struct
//!   element assignment from a formal copies member-wise;
//! - `foreach` over a subroutine-local string queue walks ELEMENTS (the
//!   string-chars arm shadowed it) and over renamed locals;
//! - a member collection shadows a same-named global-by-bare-name local
//!   from another subroutine (§8.10).

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_ccs_{}_{}", name, std::process::id()));
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
fn param_string_pool_exists_consistent() {
    let text = run(
        "pool_exists",
        r#"class item_c;
  string nm;
  function new(string name = ""); nm = name; endfunction
endclass
class my_pool #(type KEY=int, T=int);
  protected T pool[KEY];
  function new(string name=""); endfunction
  virtual function T get(KEY key);
    if (!pool.exists(key)) begin
      T default_value;
      pool[key] = default_value;
    end
    return pool[key];
  endfunction
  virtual function int exists(KEY key);
    return pool.exists(key);
  endfunction
endclass
class my_str_pool #(type T=int) extends my_pool #(string, T);
  function new(string name=""); super.new(name); endfunction
  virtual function T get(string key);
    if (!pool.exists(key))
      pool[key] = new(key);
    return pool[key];
  endfunction
endclass
class blk_c;
  string default_hdl_path = "RTL";
  function string get_default_hdl_path();
    return default_hdl_path;
  endfunction
endclass
class reg_c;
  my_str_pool #(item_c) m_pool;
  blk_c m_parent;
  function new(blk_c p);
    m_pool = new("hdl_paths");
    m_parent = p;
  endfunction
  function void add_slice(string kind = "RTL");
    item_c it = m_pool.get(kind);
  endfunction
  function bit has_path(string kind = "");
    if (kind == "")
      kind = m_parent.get_default_hdl_path();
    return m_pool.exists(kind);
  endfunction
endclass
module test;
  blk_c blk = new();
  reg_c r = new(blk);
  initial begin
    r.add_slice();
    $display("T|has()=%0d has(RTL)=%0d", r.has_path(), r.has_path("RTL"));
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|has()=1 has(RTL)=1"),
        "pool key consistency:\n{text}"
    );
}

#[test]
fn nested_spec_new_in_declinit_and_assoc_elem() {
    let text = run(
        "spec_new",
        r#"class q_c #(type T=int);
  T q[$];
  string nm;
  function new(string name=""); nm = name; endfunction
  function int size(); return q.size(); endfunction
endclass
class user_c #(type T=int);
  function void go();
    begin
      T t1 = new("di");
      $display("T|declinit null=%0d", (t1 == null));
    end
  endfunction
endclass
class pool_c #(type T=int);
  T pool[string];
  function T get(string key);
    if (!pool.exists(key))
      pool[key] = new(key);
    return pool[key];
  endfunction
endclass
module test;
  user_c #(q_c#(string)) u = new();
  pool_c #(q_c#(string)) p = new();
  initial begin
    q_c#(string) e;
    u.go();
    e = p.get("RTL");
    $display("T|elem null=%0d nm='%s'", (e == null), (e == null) ? "" : e.nm);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|declinit null=0"),
        "T x = new with nested spec:\n{text}"
    );
    assert!(
        text.contains("T|elem null=0 nm='RTL'"),
        "pool[key] = new(key):\n{text}"
    );
}

#[test]
fn member_dyn_struct_array_add_slice() {
    let text = run(
        "add_slice",
        r#"typedef struct { string path; int offset; int size; } slice_t;
class concat_c;
  slice_t slices[];
  function void add_slice(slice_t slice);
    slices = new [slices.size()+1] (slices);
    slices[slices.size()-1] = slice;
  endfunction
  function void add_path(string path, int unsigned offset = -1, int unsigned size = -1);
    slice_t t;
    t.offset = offset;
    t.path   = path;
    t.size   = size;
    add_slice(t);
  endfunction
endclass
module test;
  concat_c c = new();
  initial begin
    c.add_path("top.dut", 0, 32);
    c.add_path("x.y", 8, 16);
    $display("T|n=%0d", c.slices.size());
    foreach (c.slices[k]) begin
      string s = c.slices[k].path;
      $display("T|k=%0d path='%s' off=%0d size=%0d", k, s, c.slices[k].offset, c.slices[k].size);
    end
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|n=2"), "two slices:\n{text}");
    assert!(
        text.contains("path='top.dut' off=0 size=32") || text.contains("top.dut' off=0 size=32"),
        "slice 0 content:\n{text}"
    );
    assert!(text.contains("off=8 size=16"), "slice 1 content:\n{text}");
}

#[test]
fn local_string_queue_foreach_walks_elements() {
    let text = run(
        "strq_fe",
        r#"class c;
  function void go();
    string pp[$];
    pp.push_back("top.dut");
    pp.push_back("x");
    foreach (pp[j])
      $display("T|j=%0d '%s'", j, pp[j]);
  endfunction
endclass
module test;
  c cc = new();
  initial begin cc.go(); $finish; end
endmodule
"#,
    );
    assert!(text.contains("T|j=0 'top.dut'"), "element 0:\n{text}");
    assert!(text.contains("T|j=1 'x'"), "element 1:\n{text}");
}

#[test]
fn member_collection_shadows_global_bare_name() {
    let text = run(
        "shadow",
        r#"class item_c;
  int v;
  function new(int x); v = x; endfunction
endclass
class my_q #(type T=int);
  T queue[$];
  function void push_back(T t); queue.push_back(t); endfunction
  function int size(); return queue.size(); endfunction
  function T get(int i); return queue[i]; endfunction
endclass
module test;
  int queue[$];
  my_q #(item_c) a = new();
  initial begin
    item_c x;
    queue.push_back(77);
    queue.push_back(88);
    x = new(11); a.push_back(x);
    $display("T|mod=%0d inst=%0d v0=%0d", queue.size(), a.size(),
             a.get(0) == null ? -1 : a.get(0).v);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains("T|mod=2 inst=1 v0=11"),
        "member shadows global:\n{text}"
    );
}

#[test]
fn hdl_path_composition_end_to_end() {
    let text = run(
        "gfhp",
        r#"typedef struct { string path; int offset; int size; } slice_t;
class concat_c;
  slice_t slices[];
  function void add_slice(slice_t slice);
    slices = new [slices.size()+1] (slices);
    slices[slices.size()-1] = slice;
  endfunction
  function void add_path(string path, int unsigned offset = -1, int unsigned size = -1);
    slice_t t;
    t.offset = offset;
    t.path   = path;
    t.size   = size;
    add_slice(t);
  endfunction
endclass
class holder_c;
  concat_c q[$];
  function concat_c get(int i); return q[i]; endfunction
  function int size(); return q.size(); endfunction
endclass
class reg_c;
  holder_c pool;
  function new(); pool = new(); endfunction
  function void gfhp(ref concat_c paths[$], input string kind = "", input string separator = ".");
    string parent_paths[$];
    parent_paths.push_back("top.dut");
    for (int i = 0; i < pool.size(); i++) begin
      concat_c hdl_concat = pool.get(i);
      foreach (parent_paths[j]) begin
        concat_c t = new;
        foreach (hdl_concat.slices[k]) begin
          if (hdl_concat.slices[k].path == "")
            t.add_path(parent_paths[j]);
          else
            t.add_path({parent_paths[j], separator, hdl_concat.slices[k].path},
                       hdl_concat.slices[k].offset, hdl_concat.slices[k].size);
        end
        paths.push_back(t);
      end
    end
  endfunction
endclass
module test;
  reg_c r = new();
  initial begin
    concat_c c = new();
    concat_c outp[$];
    string s;
    c.add_path("SCRATCH", 0, 32);
    r.pool.q.push_back(c);
    r.gfhp(outp);
    $display("T|out n=%0d", outp.size());
    if (outp.size() > 0) begin
      s = outp[0].slices[0].path;
      $display("T|path='%s' off=%0d size=%0d", s, outp[0].slices[0].offset, outp[0].slices[0].size);
    end
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|out n=1"), "one composed path:\n{text}");
    assert!(
        text.contains("path='top.dut.SCRATCH' off=0 size=32"),
        "full path composed:\n{text}"
    );
}
