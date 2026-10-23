//! §8.26.4: an interface class shall not extend a type parameter, even one
//! whose type is an interface class. The reference rejects it at compile
//! time ("Illegal reference to type parameter B in interface class extends
//! clause"). The check was a `--compile`-only lint that saw only the first
//! base the AST keeps, so `extends ic_a, B` passed on every path; it is now
//! a parse error on every base, so each mode exits 1 (the #107 exit-code
//! contract, tests/misc/exit_codes.rs).

use std::process::Command;

/// Run xezim on `src` (top `tb`) with `args`; return (exit code, all output).
fn run(tag: &str, src: &str, args: &[&str]) -> (i32, String) {
    let dir = std::env::temp_dir().join(format!("xezim_icext_{}_{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("t.sv");
    std::fs::write(&sv, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(args)
        .arg("-s")
        .arg("tb")
        .arg(sv.to_str().unwrap())
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().unwrap_or(-1), all)
}

fn assert_rejected(tag: &str, src: &str) {
    for mode in [&["--simulate"][..], &[][..], &["--compile"][..]] {
        let (code, all) = run(tag, src, mode);
        assert_eq!(code, 1, "{mode:?}: expected exit 1:\n{all}");
        assert!(
            all.contains("extends type parameter 'B'")
                && all.contains("shall not extend a type parameter"),
            "{mode:?}: missing diagnostic:\n{all}"
        );
        assert!(!all.contains("T| ran"), "{mode:?}: the design ran:\n{all}");
    }
}

/// The single-base form, with and without a default, in `$unit`, a package
/// and a module.
#[test]
fn interface_class_extends_type_param_rejected_everywhere() {
    assert_rejected(
        "unit",
        r#"interface class ic_base;
  pure virtual function int f();
endclass
interface class ic_w #(type B = ic_base) extends B;
  pure virtual function int g();
endclass
module tb;
  initial $display("T| ran");
endmodule
"#,
    );
    assert_rejected(
        "nodefault",
        r#"interface class ic_w #(type B) extends B;
endclass
module tb;
  initial $display("T| ran");
endmodule
"#,
    );
    assert_rejected(
        "pkg",
        r#"package pk;
interface class ic_base;
  pure virtual function int f();
endclass
interface class ic_w #(type B = ic_base) extends B;
endclass
endpackage
module tb;
  initial $display("T| ran");
endmodule
"#,
    );
    assert_rejected(
        "module",
        r#"module tb;
interface class ic_base;
  pure virtual function int f();
endclass
interface class ic_w #(type B = ic_base) extends B;
endclass
  initial $display("T| ran");
endmodule
"#,
    );
}

/// An interface class may extend several interface classes; a type parameter
/// in any position is rejected, not only the first.
#[test]
fn interface_class_extends_type_param_second_base_rejected() {
    assert_rejected(
        "second",
        r#"interface class ic_a;
  pure virtual function int f();
endclass
interface class ic_b;
  pure virtual function int h();
endclass
interface class ic_w #(type B = ic_b) extends ic_a, B;
  pure virtual function int g();
endclass
module tb;
  initial $display("T| ran");
endmodule
"#,
    );
}

/// The legal neighbours stay accepted: extending a specialization that
/// passes the type parameter on, and a class (not an interface class)
/// extending a type parameter (§8.25).
#[test]
fn interface_class_extends_specialization_still_accepted() {
    let (code, all) = run(
        "legal",
        r#"interface class ic_p #(type T = int);
  pure virtual function T get();
endclass
interface class ic_w #(type B = int) extends ic_p #(B);
endclass
class base_c;
endclass
class wrap_c #(type B = base_c) extends B;
endclass
class impl implements ic_w #(byte);
  virtual function byte get(); return 5; endfunction
endclass
module tb;
  impl i; wrap_c w;
  initial begin i = new; w = new; $display("T| ran %0d", i.get()); end
endmodule
"#,
        &["--simulate"],
    );
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("T| ran 5"), "{all}");
}
