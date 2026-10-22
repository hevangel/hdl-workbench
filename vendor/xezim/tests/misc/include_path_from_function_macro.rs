//! §22.4/§22.5.1: an `include whose filename comes from a function-like macro
//! that stringifies its arguments (`` `define F(d, f) `"d/f`" ``) — sv-tests
//! `22.5.1--include-define-expansion`, ivtest `br_gh484`. The directive's
//! text was expanded with a single rescan, which leaves the body's `` `" ``
//! delimiters unconverted, and the include failed as "malformed". The
//! reference simulator accepts both forms.

use sv_parser::preprocessor::Preprocessor;

fn include_via(defines: &str, directive: &str) -> (String, Vec<String>) {
    let dir = std::env::temp_dir().join(format!(
        "xezim_inc_fn_macro_{}_{}",
        std::process::id(),
        directive.len()
    ));
    std::fs::create_dir_all(dir.join("sub")).expect("temp dir");
    std::fs::write(
        dir.join("sub").join("inc.vh"),
        "localparam int FROM_INC = 7;\n",
    )
    .expect("write include");
    let mut p = Preprocessor::new();
    p.add_include_dir(dir.clone());
    let src = format!("{defines}\nmodule top;\n{directive}\nendmodule\n");
    let out = p.preprocess_file(&src, Some(&dir.join("top.sv")));
    let errs = p.errors().to_vec();
    let _ = std::fs::remove_dir_all(&dir);
    (out, errs)
}

#[test]
fn two_argument_stringifying_macro_names_the_file() {
    let (out, errs) = include_via(
        "`define PATH(d, f) `\"d/f`\"",
        "`include `PATH(sub, inc.vh)",
    );
    assert!(errs.is_empty(), "{errs:?}");
    assert!(out.contains("FROM_INC = 7"), "{out}");
}

#[test]
fn one_argument_stringifying_macro_names_the_file() {
    let (out, errs) = include_via("`define Q(f) `\"f`\"", "`include `Q(sub/inc.vh)");
    assert!(errs.is_empty(), "{errs:?}");
    assert!(out.contains("FROM_INC = 7"), "{out}");
}

#[test]
fn plain_text_macro_still_names_the_file() {
    let (out, errs) = include_via("`define P \"sub/inc.vh\"", "`include `P");
    assert!(errs.is_empty(), "{errs:?}");
    assert!(out.contains("FROM_INC = 7"), "{out}");
}
