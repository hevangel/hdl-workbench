//! §21.2.1.3: `%s` of a `string` member reached through a NESTED struct path
//! (`o.in.s`) must print the string, not the container padded to its
//! declared width. Only the OUTERMOST struct's direct members were consulted,
//! so the same member printed correctly through a one-level path and padded
//! through a two-level one. Reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_nssmd_{}_{}", name, std::process::id()));
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
fn nested_struct_string_member_prints_unpadded() {
    let text = run(
        "nested_str",
        r#"typedef struct { int a; string s; } inner_t;
typedef struct { inner_t in; int b; } outer_t;
class c;
  function outer_t mk();
    outer_t o;
    o.in.a = 7; o.in.s = "deep"; o.b = 9;
    return o;
  endfunction
  function void go();
    outer_t src;
    src = mk();
    begin
      outer_t d1 = src;
      inner_t d3 = src.in;
      $display("T|nested='%s'", d1.in.s);
      $display("T|onelevel='%s'", d3.s);
      $display("T|direct='%s'", src.in.s);
    end
  endfunction
endclass
module test;
  c cc = new();
  initial begin cc.go(); $finish; end
endmodule
"#,
    );
    for pin in ["T|nested='deep'", "T|onelevel='deep'", "T|direct='deep'"] {
        assert!(text.contains(pin), "missing `{pin}`:\n{text}");
    }
}
