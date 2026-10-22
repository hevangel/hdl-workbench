//! §22.5.1: a user macro name may merely START with a compiler-directive
//! keyword. The preprocessor matched directives by PREFIX, so invoking a
//! macro named `include_*` failed preprocessing outright ("malformed
//! `include"), and other directive-prefixed names risked the same. Also
//! covers a macro whose BODY is an `ifdef-guarded block expanding to
//! nothing when the guard is undefined. Reference-verified.

use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_mdpn_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb_top",
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
fn guarded_macro_with_include_prefix_expands_empty() {
    let text = run(
        "guarded",
        r#"`undef _XLINK_

`define include_default_link_error_task \
`ifdef _XLINK_ \
   export "DPI-C" function tb_report_from_link; \
   function void tb_report_from_link (input int err_code); \
      $display("link error %0d", err_code); \
      $finish(); \
   endfunction \
`endif

module tb_top;
   `include_default_link_error_task
   bit macro_was_empty = 0;
   generate
      if (1) begin : g_check
         initial macro_was_empty = 1;
      end
   endgenerate
   initial begin
      #10;
      if (macro_was_empty) $display("TEST_PASS");
      else $display("TEST_FAIL");
      $finish();
   end
endmodule
"#,
    );
    assert!(
        text.contains("TEST_PASS"),
        "guarded include_* macro:\n{text}"
    );
    assert!(
        !text.contains("malformed"),
        "no directive misparse:\n{text}"
    );
}

#[test]
fn directive_prefixed_macro_names_expand() {
    let text = run(
        "names",
        r#"`define undefined_marker 7
`define ifdef_guard_val 3
`define elsewhere 11
`define endif_tag 13
`define definitely 17
`define includes_all 19
module tb_top;
  initial begin
    int a;
    a = `undefined_marker + `ifdef_guard_val + `elsewhere + `endif_tag + `definitely + `includes_all;
    if (a == 70) $display("TEST_PASS");
    else $display("TEST_FAIL a=%0d", a);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("TEST_PASS"), "all names expand:\n{text}");
}
