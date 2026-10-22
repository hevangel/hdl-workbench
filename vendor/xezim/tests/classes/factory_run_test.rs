// Regression test: `new()` called inside a static method must allocate a real
// instance, not silently return null.  Before the fix, `new()` inside a static
// method (where `this` = 0) was dispatched via `exec_static_method`, which
// correctly returned `None` for `"new"` — but there was no fallback to
// `instantiate_class`, so the result was 0 (null handle).
//
// Related but distinct from `constructor_new_not_static.rs` (which tests
// `ClassName::new(args)` from a non-static context).  This test specifically
// stresses the `static function` path where no `this` exists.
//
// Verified byte-for-byte against reference simulators.

use std::process::Command;

#[test]
fn new_in_static_method_allocates_instance() {
    let sv = r#"
module top;
    class Foo;
        static int count;
        static function Foo create();
            Foo f;
            f = new();
            f.val = 42;
            return f;
        endfunction
        int val;
    endclass

    initial begin
        Foo f = Foo::create();
        if (f == null) $display("FAIL: f is null");
        else if (f.val != 42) $display("FAIL: f.val = %0d", f.val);
        else $display("PASS");
    end
endmodule
"#;

    let dir = std::env::temp_dir().join(format!("xezim_factory_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv_path = dir.join("factory_new.sv");
    std::fs::write(&sv_path, sv).unwrap();

    let bin = env!("CARGO_BIN_EXE_xezim");
    let out = Command::new(bin)
        .args(["--simulate", "-s", "top", sv_path.to_str().unwrap()])
        .output()
        .expect("xezim failed to start");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("PASS"), "Expected PASS, got:\n{}", stdout);
}
