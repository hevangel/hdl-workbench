//! IEEE 1800-2023 §8.25.4 / nested-class static-method dispatch: a TYPE
//! PARAMETER used as the receiver of `type_id::create` must resolve to its
//! concrete specialization argument and construct THAT class.
//!
//! UVM models its factory with a per-class nested `type_id` class whose
//! static `create` method returns a fresh instance, e.g. inside
//! `uvm_reg_predictor#(BUSTYPE)::type_name()`:
//!
//! ```sv
//! BUSTYPE t;
//! t = BUSTYPE::type_id::create("t");
//! m_type_name = {"uvm_reg_predictor #(", t.get_type_name(), ")"};
//! ```
//!
//! xezim does not elaborate nested classes (they are never registered in
//! `module.classes`); it instead INTERCEPTS `ClassName::type_id::create`
//! and constructs the class directly. That interception previously only
//! handled a direct class name (and a typedef alias) as the receiver —
//! NOT a type parameter (`BUSTYPE`). The call returned null, so the
//! constructed object's virtual `get_type_name()` returned "", and the
//! factory's `m_type_name` cache was built as `"uvm_reg_predictor #()"`
//! instead of `"uvm_reg_predictor #(uvm_sequence_item)"`. This pins the
//! type-parameter receiver case.

use std::process::Command;

fn xezim() -> String {
    env!("CARGO_BIN_EXE_xezim").to_string()
}

#[test]
fn type_param_type_id_create_constructs_concrete_class() {
    let src = r#"
class uvm_object_base;
  function new(string name = ""); endfunction
  virtual function string get_type_name();
    return "uvm_object_base";
  endfunction
endclass

class uvm_sequence_item extends uvm_object_base;
  function new(string name = "");
    super.new(name);
  endfunction
  virtual function string get_type_name();
    return "uvm_sequence_item";
  endfunction
endclass

// Enclosing parameterized class: BUSTYPE is a TYPE PARAMETER. Its static
// type_name() calls BUSTYPE::type_id::create — the receiver is a type
// parameter, the exact UVM pattern from uvm_reg_predictor.
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
  virtual function string get_type_name();
    return type_name();
  endfunction
  function new(string name = "");
    super.new(name);
  endfunction
endclass

module top;
  initial begin
    predictor #(uvm_sequence_item) p;
    p = new("p");
    if (p.get_type_name() == "predictor #(uvm_sequence_item)")
      $display("TAG_PASS");
    else
      $display("TAG_FAIL got=[%s]", p.get_type_name());
  end
endmodule
"#;

    let tmp = std::env::temp_dir().join("xezim_typeparam_typeid_create.sv");
    std::fs::write(&tmp, src).unwrap();

    let output = Command::new(xezim())
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(tmp.to_str().unwrap())
        .output()
        .expect("Failed to execute xezim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        !combined.contains("Parse errors"),
        "Parse error:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Type-parameter type_id::create did not construct the concrete class.\nOutput:\n{combined}"
    );
}
