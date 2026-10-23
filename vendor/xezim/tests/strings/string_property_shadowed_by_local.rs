//! Regression test: a method-local that shadows a `string` class property must
//! be treated as its declared (non-string) type, not as the shadowed property.
//!
//! Before the fix, `class_member_is_string` matched a bare `Ident` against
//! `this`'s properties without first checking the local scope, so an `int val`
//! local shadowing a `string val` property was mis-typed as string. That made
//! `$display("tag", val)` of the `int` local (65) print the character 'A' (65
//! as an ASCII code) instead of the number 65, because the string-typing also
//! drives $display's "leading string argument is the format spec" path.

use xezim::simulate;

const SRC: &str = r#"
class C;
  string val = "abc";
  function void show();
    int val;        // method-local shadows the string property
    val = 65;
    if (val == 65) $display("TAG_PASS"); else $display("TAG_FAIL val=%0d", val);
  endfunction
endclass

module top;
  initial begin
    C c = new();
    c.show();
  end
endmodule
"#;

#[test]
fn test_string_property_shadowed_by_local() {
    let sim = simulate(SRC, 10_000).expect("simulation failed");
    assert!(
        sim.output
            .iter()
            .any(|line| line.message.contains("TAG_PASS")),
        "expected TAG_PASS in output, got: {:?}",
        sim.output
    );
}
