//! §8.25: a class property declared with a TYPE PARAMETER
//! (`class pairi #(type T=int); T first; T second;`) is an unpacked struct
//! only for the specializations that bind `T` to an actual struct type. All
//! specializations of one class share the same `class_name` (they differ only
//! in their type bindings), so the class-property struct NEGATIVE cache —
//! keyed only by `(class_name, prop)` — used to poison one binding with the
//! other's answer.
//!
//! Two bugs came out of that:
//!   * writing `a1.first = 5` (binding `T -> int`) cached "`pairi.first` is
//!     not a struct", so `a6.first` (binding `T -> struct s`) resolved the
//!     struct decomposition against the poisoned cache and read back `x` for
//!     `a6.first.a`/`a6.first.b`.
//!   * `%p`/`convert2string` of the struct-bound `first`/`second` then printed
//!     the raw packed cell (`pair : 0, 0`) instead of the reference simulator's
//!     struct notation (`pair : '{a:0, b:0}, '{a:0, b:0}`).
//!
//! Reference-verified byte-for-byte on the printed lines below.

use std::path::PathBuf;
use std::process::Command;

const DESIGN: &str = r#"
typedef struct { int a; int b; } s_t;
class pairi #(type T = int);
  T first;
  T second;
  function string spr();
    return $sformatf("pair : %p, %p", first, second);
  endfunction
endclass

module top;
  pairi #(int) a1;
  pairi #(s_t) a6;
  pairi #(string) a2;
  int bad = 0;
  initial begin
    a1 = new; a6 = new; a2 = new;
    // a6.second set FIRST so that a write to the int-bound property must
    // NOT corrupt it.
    a6.first.a = 3; a6.first.b = 4;
    a6.second.a = 30; a6.second.b = 40;
    a1.first = 5; a1.second = 6;
    a2.first = "y"; a2.second = "z";
    if (a6.first.a !== 3 || a6.first.b !== 4) bad++;
    if (a6.second.a !== 30 || a6.second.b !== 40) bad++;
    if (a1.first !== 5 || a1.second !== 6) bad++;
    if (a2.first !== "y" || a2.second !== "z") bad++;
    $display("BAD %0d", bad);
    $display("S %0s", a6.spr());
    $display("STR %0s", a2.spr());
    $finish;
  end
endmodule
"#;

fn run() -> String {
    // The three tests below share one design; run it ONCE and hand every
    // test the same text. Writing one shared `t.sv` from each test and
    // spawning xezim on it raced under the parallel harness (a test
    // rewrote the file while another's xezim was reading it) and failed
    // about one run in three.
    static OUT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    OUT.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("class_type_param_struct_prop");
        std::fs::create_dir_all(&dir).unwrap();
        let sv = dir.join("t.sv");
        std::fs::write(&sv, DESIGN).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
            .args([
                "--simulate",
                "-s",
                "top",
                "--no-cache",
                sv.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        let mut text = String::from_utf8_lossy(&output.stdout).to_string();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        assert!(output.status.success(), "run failed:\n{text}");
        text
    })
    .clone()
}

#[test]
fn type_param_struct_prop_not_poisoned_by_sibling_binding() {
    let text = run();
    // The int-bound write must not corrupt the struct-bound property's fields.
    assert!(
        text.contains("BAD 0"),
        "cross-binding contamination:\n{text}"
    );
}

#[test]
fn type_param_struct_prop_renders_as_struct() {
    let text = run();
    // `%p` of the struct-bound property must be the struct notation, matching
    // the reference simulator exactly.
    assert!(
        text.contains("S pair : '{a:3, b:4}, '{a:30, b:40}"),
        "struct rendering off:\n{text}"
    );
}

#[test]
fn type_param_string_prop_renders_quoted() {
    let text = run();
    // `%p` of a STRING-bound type-parameter property must be the quoted string
    // (reference simulator prints `"y", "z"`), not the raw first byte of the
    // packed cell (`121, 122`).
    assert!(
        text.contains("STR pair : \"y\", \"z\""),
        "string rendering off:\n{text}"
    );
}
