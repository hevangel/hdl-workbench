//! IEEE 1800-2023 §6.16 / Table 6-9: equality (`==`/`!=`) and relational
//! (`<`/`<=`/`>`/`>=`) operators on the `string` data type are 2-STATE —
//! they compare the textual content and always yield a definite 0 or 1,
//! never X/Z.
//!
//! xezim stores a `string` value as a fixed-width 1024-bit packed vector
//! (128-char capacity) whose unused high bits are X when the string is
//! shorter than the capacity.  Routing a string comparison through the
//! integral 4-state equality path would therefore return X whenever
//! either side had X padding — e.g. comparing an empty string `""` to a
//! non-empty one returned X instead of 0, which silently broke UVM field
//! automation compare (`compare_string` was never called).  This test
//! pins the 2-state semantics for both module-scope string variables and
//! class string properties.
use std::process::Command;

fn xezim() -> String {
    env!("CARGO_BIN_EXE_xezim").to_string()
}

const SRC: &str = r#"
module top;
  // ---- module-scope string variables ----
  string s1, s2;
  initial begin
    // Two empty strings are equal (both width-0 / all-X storage).
    if ((s1 == s2) !== 1)  $display("FAIL empty==empty");
    if ((s1 != s2) !== 0)  $display("FAIL empty!=empty");
    // Empty vs non-empty.
    s1 = "hello";
    if ((s1 == s2) !== 0)  $display("FAIL hello==empty");
    if ((s1 != s2) !== 1)  $display("FAIL hello!=empty");
    // Equal after assignment.
    s2 = "hello";
    if ((s1 == s2) !== 1)  $display("FAIL hello==hello");
    // Relational: lexicographic, shorter prefix is "less".
    s2 = "hello!";
    if ((s1 <  s2) !== 1)  $display("FAIL hello < hello!");
    if ((s1 <= s2) !== 1)  $display("FAIL hello <= hello!");
    if ((s1 >  s2) !== 0)  $display("FAIL hello > hello!");
    if ((s2 >= s1) !== 1)  $display("FAIL hello! >= hello");
  end

  // ---- class string property ----
  class cls;
    string a;
    string b;
    function automatic int cmp_eq();
      return (a == b);
    endfunction
    function automatic int cmp_ne();
      return (a != b);
    endfunction
  endclass

  initial begin
    cls c;
    c = new();
    // Two empty class-property strings.
    if (c.cmp_eq() !== 1)  $display("FAIL cls empty==empty");
    if (c.cmp_ne() !== 0)  $display("FAIL cls empty!=empty");
    c.a = "bang";
    // a="bang", b="" must differ.
    if (c.cmp_eq() !== 0)  $display("FAIL cls bang==empty");
    if (c.cmp_ne() !== 1)  $display("FAIL cls bang!=empty");
    c.b = "bang";
    if (c.cmp_eq() !== 1)  $display("FAIL cls bang==bang");
  end

  initial begin
    $display("TAG_PASS");
  end
endmodule
"#;

fn run(src: &str) -> String {
    let path = "/tmp/streq2state.sv";
    std::fs::write(path, src).unwrap();
    let out = Command::new(xezim())
        .args(["--simulate", "-s", "top", path])
        .output()
        .expect("run xezim");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn string_eq_relational_are_2state() {
    let out = run(SRC);
    assert!(
        !out.contains("FAIL"),
        "string comparison returned X (4-state) instead of 0/1:\n{out}"
    );
    assert!(out.contains("TAG_PASS"), "missing TAG_PASS:\n{out}");
}
