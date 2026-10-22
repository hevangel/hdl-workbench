//! §8.25 — a class TYPE PARAMETER used as the receiver of a class-scoped
//! STATIC PROPERTY read (`T::static_prop`, where `T` is a type parameter of
//! the enclosing parameterized class) must resolve `T` to its concrete bound
//! class and read that class's shared static cell.
//!
//! This is the receiver pattern at the heart of UVM's factory/registry: a
//! parameterized class like `uvm_reg_predictor #(BUSTYPE)` declares
//!
//! ```text
//!   static function string type_name();
//!     static string m_type_name;
//!     if (m_type_name == "")
//!       m_type_name = {"predictor #(", BUSTYPE::factory_name, ")"};
//!     return m_type_name;
//!   endfunction
//! ```
//!
//! and `BUSTYPE::factory_name` is exactly `T::static_prop`. Static *methods*
//! (`T::method()`) and *typedefs* (`T::type_id::create()`) already worked
//! because their dispatch paths call `resolve_type_param_binding`, but the
//! static-PROPERTY read path only consulted `module.classes.contains_key(cls)`,
//! which is false for a type-parameter name — so `T::prop` silently read as
//! zero/empty. The fix resolves the leading segment through the active
//! specialization (`current_spec`) and, for instance methods, the `this`
//! instance's `type_bindings`, before the class lookup.
//!
//! All cases below print `TAG_PASS` on the reference simulator and (after the
//! fix) on xezim. Verified byte-for-byte against reference simulators.

use std::process::Command;

fn xezim() -> String {
    env!("CARGO_BIN_EXE_xezim").to_string()
}

fn run(src: &str, tag: &str) -> String {
    let path = format!("/tmp/tp_static_{tag}.sv");
    std::fs::write(&path, src).unwrap();
    let out = Command::new(xezim())
        .args(["--simulate", "-s", "top", &path])
        .output()
        .expect("run xezim");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `T::static_string` inside a STATIC method of a parameterized class — the
/// most direct form of the bug and the exact UVM `m_type_name` shape.
#[test]
fn static_string_through_type_param() {
    let src = r#"module top;
  class C;
    static string factory_name = "C-item";
    function new(string n = ""); endfunction
  endclass
  class P #(type T = int);
    static function string get(); return T::factory_name; endfunction
  endclass
  initial begin
    if (P #(C)::get() == "C-item")
      $display("TAG_PASS");
    else
      $display("TAG_FAIL got=[%s]", P #(C)::get());
  end
endmodule
"#;
    let out = run(src, "str");
    assert!(
        out.contains("TAG_PASS"),
        "T::static_string must resolve\n{out}"
    );
}

/// `T::static_int` through a type parameter — same path, integral type.
#[test]
fn static_int_through_type_param() {
    let src = r#"module top;
  class C;
    static int count = 7;
    function new(string n = ""); endfunction
  endclass
  class P #(type T = int);
    static function int get(); return T::count; endfunction
  endclass
  initial begin
    if (P #(C)::get() == 7)
      $display("TAG_PASS");
    else
      $display("TAG_FAIL got=%0d", P #(C)::get());
  end
endmodule
"#;
    let out = run(src, "int");
    assert!(
        out.contains("TAG_PASS"),
        "T::static_int must resolve\n{out}"
    );
}

/// The full UVM `predictor` shape: an INHERITED static that the bound class
/// SHADOWS, read through the type parameter, used to build a cached type name.
/// Exercises inheritance + shadowing + the `{"...", BUSTYPE::prop}` concat.
#[test]
fn inherited_shadowed_static_through_type_param() {
    let src = r#"module top;
  class uvm_object_base;
    static string factory_name = "uvm_object_base";
    function new(string name = ""); endfunction
  endclass
  class item extends uvm_object_base;
    static string factory_name = "item";
    function new(string name = ""); super.new(name); endfunction
  endclass
  class predictor #(type BUSTYPE = int) extends uvm_object_base;
    static function string type_name();
      static string m_type_name;
      if (m_type_name == "")
        m_type_name = {"predictor #(", BUSTYPE::factory_name, ")"};
      return m_type_name;
    endfunction
    function new(string name = ""); super.new(name); endfunction
  endclass
  initial begin
    if (predictor #(item)::type_name() == "predictor #(item)")
      $display("TAG_PASS");
    else
      $display("TAG_FAIL got=[%s]", predictor #(item)::type_name());
  end
endmodule
"#;
    let out = run(src, "inherit");
    assert!(
        out.contains("TAG_PASS"),
        "shadowed inherited static via T must resolve\n{out}"
    );
}

/// The genuine UVM factory idiom: `BUSTYPE::type_id::create("t")` where
/// `type_id` is a typedef to a proxy class — nested `::` resolved through a
/// type parameter (typedef + static-method dispatch combined).
#[test]
fn factory_typedef_create_through_type_param() {
    let src = r#"module top;
  class uvm_object_base;
    function new(string name = ""); endfunction
    virtual function string get_type_name(); return "uvm_object_base"; endfunction
  endclass
  class proxy #(type T = int);
    static function T create(string name);
      T obj;
      obj = new(name);
      return obj;
    endfunction
  endclass
  class item extends uvm_object_base;
    typedef proxy #(item) type_id;
    function new(string name = ""); super.new(name); endfunction
    virtual function string get_type_name(); return "item"; endfunction
  endclass
  class predictor #(type BUSTYPE = int) extends uvm_object_base;
    static function string type_name();
      static string m_type_name;
      if (m_type_name == "") begin
        BUSTYPE t;
        t = BUSTYPE::type_id::create("t");
        m_type_name = {"predictor #(", t.get_type_name(), ")"};
      end
      return m_type_name;
    endfunction
    function new(string name = ""); super.new(name); endfunction
  endclass
  initial begin
    if (predictor #(item)::type_name() == "predictor #(item)")
      $display("TAG_PASS");
    else
      $display("TAG_FAIL got=[%s]", predictor #(item)::type_name());
  end
endmodule
"#;
    let out = run(src, "factory");
    assert!(
        out.contains("TAG_PASS"),
        "BUSTYPE::type_id::create via T must resolve\n{out}"
    );
}

/// The INSTANCE-METHOD path: `T::static_prop` resolved through the instance's
/// `type_bindings` (the `this` handle), not a static `current_spec`. Reads in
/// both the constructor and a regular instance method.
#[test]
fn static_through_type_param_in_instance_method() {
    let src = r#"module top;
  class C;
    static string nm = "C";
    function new(string n = ""); endfunction
  endclass
  class P #(type T = int);
    string cached;
    function new(string n = "");
      cached = T::nm;          // read inside the instance constructor
    endfunction
    function string get();
      return T::nm;            // read inside an instance method
    endfunction
  endclass
  initial begin
    P #(C) p;
    p = new("p");
    if (p.cached == "C" && p.get() == "C")
      $display("TAG_PASS");
    else
      $display("TAG_FAIL cached=[%s] get=[%s]", p.cached, p.get());
  end
endmodule
"#;
    let out = run(src, "inst");
    assert!(
        out.contains("TAG_PASS"),
        "T::static via this must resolve\n{out}"
    );
}
