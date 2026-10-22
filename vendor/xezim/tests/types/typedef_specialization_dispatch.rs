// Regression test for GAP-B: typedef specialization dispatch (IEEE 1800-2023
// §8.25.1). A typedef alias to a parameterized-class specialization must
// dispatch static methods on the underlying class, with value/type parameters
// correctly bound.
//
// Two patterns:
//   (1) Module-level typedef specialization:
//         typedef Common#(int, "alpha") AlphaT;
//         AlphaT::type_name()   // static -> "alpha"
//         AlphaT::get().get_type_name()  // virtual -> "alpha"
//
//   (2) Class-local typedef specialization (the UVM factory delegation core):
//         class Wrapper #(type T, string Tname);
//           typedef Common#(this_type, Tname) common_type;
//           virtual function string get_type_name();
//             common_type common = common_type::get();  // static on typedef spec
//             return common.get_type_name();            // virtual on result
//           endfunction
//         endclass
//
// Before the fix, `AliasT::method()` (where AliasT is a typedef) was silently
// dropped — the static-dispatch guards required a class name in module.classes,
// so typedef names fell through and returned empty/null.

use std::process::Command;

fn run_xezim(src: &str, tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_gap_b_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv_path = dir.join(format!("typedef_spec_{tag}.sv"));
    std::fs::write(&sv_path, src).unwrap();
    let bin = env!("CARGO_BIN_EXE_xezim");
    let out = Command::new(bin)
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(sv_path.to_str().unwrap())
        .output()
        .expect("failed to run xezim");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn module_level_typedef_specialization_static_and_virtual() {
    let src = r#"class Common #(type Treg = int, string Tname = "x");
  typedef Common#(Treg, Tname) this_type;
  static this_type m_inst;
  static function this_type get();
    if (m_inst == null) m_inst = new();
    return m_inst;
  endfunction
  static function string type_name();
    return Tname;
  endfunction
  virtual function string get_type_name();
    return type_name();
  endfunction
endclass

module top;
  typedef Common#(int, "alpha") AlphaT;
  initial begin
    AlphaT a;
    // static method via typedef'd specialization
    if (AlphaT::type_name() == "alpha") $display("PASS static");
    else $display("FAIL static got='%s'", AlphaT::type_name());
    // virtual method via instance returned from typedef'd static call
    a = AlphaT::get();
    if (a != null && a.get_type_name() == "alpha") $display("PASS virtual");
    else $display("FAIL virtual got='%s'", a==null?"null":a.get_type_name());
  end
endmodule
"#;
    let out = run_xezim(src, "modlevel");
    assert!(
        out.contains("PASS static"),
        "static typedef dispatch failed:\n{out}"
    );
    assert!(
        out.contains("PASS virtual"),
        "virtual typedef dispatch failed:\n{out}"
    );
    assert!(!out.contains("FAIL"), "unexpected FAIL:\n{out}");
}

#[test]
fn class_local_typedef_specialization_factory_delegation() {
    // Faithful reproducer of the UVM factory get_type_name delegation:
    // Wrapper::get().get_type_name() where the body calls common_type::get() —
    // a static call on a class-local typedef'd specialization whose first arg
    // is `this_type` (the enclosing typedef) and whose second arg is a string
    // value parameter.
    let src = r#"class Base;
  virtual function string get_type_name();
    return "<unknown>";
  endfunction
endclass

class Common #(type Treg=int, string Tname="<unknown>");
  typedef Common#(Treg, Tname) this_type;
  static this_type m_g;
  static function this_type get();
    if (m_g == null) m_g = new();
    return m_g;
  endfunction
  static function string type_name();
    return Tname;
  endfunction
  virtual function string get_type_name();
    return type_name();
  endfunction
endclass

class Wrapper #(type T=int, string Tname="<unknown>") extends Base;
  typedef Wrapper#(T, Tname) this_type;
  typedef Common#(this_type, Tname) common_type;
  virtual function string get_type_name();
    common_type common = common_type::get();
    return common.get_type_name();
  endfunction
  static function this_type get();
    static this_type m_inst;
    if (m_inst == null) m_inst = new();
    return m_inst;
  endfunction
endclass

module top;
  initial begin
    Wrapper#(int, "my_type") inst;
    inst = Wrapper#(int, "my_type")::get();
    if (inst.get_type_name() == "my_type") $display("PASS delegation");
    else $display("FAIL delegation got='%s'", inst.get_type_name());
  end
endmodule
"#;
    let out = run_xezim(src, "delegation");
    assert!(
        out.contains("PASS delegation"),
        "factory delegation chain failed:\n{out}"
    );
    assert!(!out.contains("FAIL"), "unexpected FAIL:\n{out}");
}

#[test]
fn class_local_typedef_in_spec_params_static_field_read() {
    let src = r#"class Registry #(type T = int);
  static string name = "unnamed";
  static function string get_name();
    return name;
  endfunction
  static function void set_name(string s);
    name = s;
  endfunction
endclass

class Component #(type T = int);
  typedef Component#(T) this_type;
  typedef Registry#(this_type) reg_type;

  static function void init(string s);
    reg_type::set_name(s);
  endfunction

  static function string check();
    return reg_type::get_name();
  endfunction
endclass

module top;
  initial begin
    Component#(int)::init("comp_int");
    Component#(bit)::init("comp_bit");

    if (Component#(int)::check() == "comp_int" && Component#(bit)::check() == "comp_bit")
      $display("PASS static_field");
    else
      $display("FAIL static_field int='%s' bit='%s'", Component#(int)::check(), Component#(bit)::check());
  end
endmodule
"#;
    let out = run_xezim(src, "static_field");
    assert!(
        out.contains("PASS static_field"),
        "static field access failed:\n{out}"
    );
    assert!(!out.contains("FAIL"), "unexpected FAIL:\n{out}");
}
