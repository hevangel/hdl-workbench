//! Two elaboration-semantics pins from the UVM "fix all" round:
//!
//! 1. §8.25: a SPECIALIZED parameterized class's static member initialized
//!    from a string VALUE parameter (`const static string type_name =
//!    Tname;`) must hold that specialization's value — the lazy seed used
//!    the no-spec elaboration value (a blank), so every UVM registry
//!    registered under the same blank name and the factory's by-name table
//!    was corrupted.
//! 2. §26.3: an EXPLICITLY package-qualified queue mutation
//!    (`pkg::q.push_back(v)`) from a class-method context must mutate the
//!    package queue — it was a silent no-op (bare imported names worked),
//!    which broke UVM 1800.2's entire deferred factory registration.
//! Reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_sspq_{}_{}", name, std::process::id()));
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
            "--max-time",
            "100",
        ])
        .output()
        .expect("run xezim");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn spec_static_string_param_value() {
    let text = run(
        "specstr",
        r#"class reg_c #(type T = int, string Tname = "unset");
  const static string type_name = Tname;
  static function string get_name_s(); return type_name; endfunction
endclass
module test;
  typedef reg_c#(int, "int_reg") int_reg_t;
  typedef reg_c#(bit, "bit_reg") bit_reg_t;
  initial begin
    $display("T|a='%s' b='%s'", int_reg_t::get_name_s(), bit_reg_t::get_name_s());
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|a='int_reg' b='bit_reg'"), "{text}");
}

#[test]
fn pkg_qualified_queue_mutation_from_class_methods() {
    let text = run(
        "pkgq",
        r#"package p2;
  int q[$];
endpackage
package p3;
  class helper_c;
    static function bit rt_push();
      p2::q.push_back(7);
      return 1;
    endfunction
    function bit inst_push();
      p2::q.push_back(9);
      return 1;
    endfunction
  endclass
endpackage
module test;
  initial begin
    p3::helper_c h = new();
    void'(p3::helper_c::rt_push());
    $display("T|after static: %0d front=%0d", p2::q.size(), p2::q[0]);
    void'(h.inst_push());
    $display("T|after inst: %0d back=%0d", p2::q.size(), p2::q[1]);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|after static: 1 front=7"), "{text}");
    assert!(text.contains("T|after inst: 2 back=9"), "{text}");
}
