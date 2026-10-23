//! Preprocessor and parser defects reported as GitHub issues #60, #62, #61 and
//! #59. Each reproducer here is the one from the issue, reduced to what the
//! library API can run.
//!
//! All four were silent or misleading rather than loud: the preprocessor
//! emitted text that looked plausible, and the failure only surfaced later as a
//! syntax error pointing at a column the user never wrote.

use sv_parser::preprocessor::Preprocessor;

/// Normalise whitespace so a test asserts on TOKENS, not on the blank lines the
/// preprocessor emits to keep line numbers aligned.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn pp(src: &str) -> String {
    Preprocessor::new().preprocess(src)
}

// ---------------------------------------------------------------------------
// #60 — macro definition whose last line ends with `\`
// ---------------------------------------------------------------------------

/// A `\`define` whose final body line ends with a continuation backslash, with
/// no line after it to continue onto — the source (here an `include file) ends
/// mid-continuation.
///
/// The join loop took the text before the backslash, tried `lines.next()`, got
/// `None`, and then fell through to the unconditional "no continuation here"
/// tail — which appended the SAME text a second time, backslash included. The
/// body came out doubled (`{1, arg} {1, arg} \`), and the stray `\` made the
/// statement unparseable.
///
/// Two `preprocess` calls on ONE preprocessor reproduce that boundary exactly:
/// definitions carry over, and the first source ends while the body is still
/// continued. (Within a single source the backslash legitimately continues onto
/// whatever line follows — that is `ordinary_multiline_define_is_unaffected`.)
#[test]
fn trailing_backslash_at_end_of_source_does_not_double_the_body() {
    let mut p = Preprocessor::new();
    p.preprocess("`define FOO(arg) \\\n  {1, arg} \\\n");
    let out = p.preprocess("`FOO(1'b0)\n");
    assert_eq!(
        out.matches("{1, 1'b0}").count(),
        1,
        "body must expand once, not twice: {out:?}"
    );
    assert!(
        !out.contains('\\'),
        "the dangling continuation backslash must not survive: {out:?}"
    );
}

/// The issue as filed: the definition lives in an `include`d `.svh` whose last
/// line is the backslash, and the invocation is in the parent file. The macro
/// body must not run on into the includer.
#[test]
fn trailing_backslash_at_end_of_an_include_file() {
    let dir = std::env::temp_dir().join("xezim_pp_issue60");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let svh = dir.join("defs.svh");
    // No trailing newline after the final `\`, exactly as the issue specifies.
    std::fs::write(&svh, "`define FOO(arg) \\\n  {1, arg} \\").expect("write defs.svh");

    let mut p = Preprocessor::new();
    p.add_include_dir(dir.clone());
    let top = dir.join("top.sv");
    let out = p.preprocess_file(
        "`include \"defs.svh\"\nmodule top;\n  wire [1:0] x;\n  assign x = `FOO(1'b0);\nendmodule\n",
        Some(&top),
    );

    assert_eq!(
        out.matches("{1, 1'b0}").count(),
        1,
        "body must expand once, not twice: {out:?}"
    );
    assert!(!out.contains('\\'), "no stray backslash: {out:?}");
    assert!(
        squash(&out).contains("assign x = {1, 1'b0};"),
        "the assignment must come out well-formed: {out:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Ordinary multi-line macros — the overwhelmingly common case — keep working:
/// every continued line contributes its text exactly once.
#[test]
fn ordinary_multiline_define_is_unaffected() {
    let out = pp(
        "`define MULTI(a,b) \\\n   begin \\\n     x = a; \\\n     y = b; \\\n   end\n\
         `MULTI(1,2)\n",
    );
    let s = squash(&out);
    assert_eq!(s, "begin x = 1; y = 2; end", "got {s:?}");
}

// ---------------------------------------------------------------------------
// #62 — `` inside a string literal. Reported as a bug; it is not one.
// ---------------------------------------------------------------------------
//
// §22.5.1 says argument substitution shall not occur within a string literal,
// and gives `\`"` as the construct for building a string out of an argument. A
// plain-quoted `"``x``"` is one opaque string-literal token, so the `x` inside
// stays literal. A reference simulator agrees byte-for-byte. These tests pin
// the boundary so it is not "fixed" into a divergence later.

/// The issue's reproducer: `"``x``"` does NOT expand. Both delimiters and the
/// formal name survive as text.
#[test]
fn double_backtick_inside_a_plain_string_literal_stays_literal() {
    let out = pp("`define STRINGIFY(x) \"``x``\"\n`STRINGIFY(hello)\n");
    assert_eq!(squash(&out), "\"``x``\"", "got {out:?}");
}

/// `\`"…`"` is the construct that DOES build a string from an argument, and a
/// `` inside it joins tokens as expected — this is what #62's reporter should
/// use, and it already worked.
#[test]
fn backtick_quote_is_the_way_to_stringify_an_argument() {
    let plain = pp("`define TYPENAME(T) `\"T`\"\n`TYPENAME(my_class)\n");
    assert_eq!(squash(&plain), "\"my_class\"", "got {plain:?}");

    let joined = pp("`define BTQJOIN(a,b) `\"a``b`\"\n`BTQJOIN(foo,bar)\n");
    assert_eq!(squash(&joined), "\"foobar\"", "got {joined:?}");
}

/// `` outside a string joins tokens — the case the issue confirmed was fine.
#[test]
fn double_backtick_outside_a_string_joins_tokens() {
    let out = pp("`define CAT(a,b) ``a``_``b``\n`CAT(foo,bar)\n");
    assert_eq!(squash(&out), "foo_bar", "got {out:?}");
}

/// IEEE 1800-2023 §22.5.1: Token pasting `` `` `` occurs before macro lookup on the resulting merged token.
#[test]
fn double_backtick_token_pasting_before_macro_lookup() {
    let out = pp("`define FOO_BAR 42\n`define CONCAT(x) `FOO_``x\n`CONCAT(BAR)\n");
    assert_eq!(squash(&out), "42", "got {out:?}");

    let out2 =
        pp("`define A 1\n`define B 2\n`define AB 99\n`define PASTE(x,y) `x``y\n`PASTE(A,B)\n");
    assert_eq!(squash(&out2), "99", "got {out2:?}");
}

/// The rule that makes the opaque-string treatment necessary: a formal whose
/// name also appears inside a format string is NOT substituted there.
/// Substituting it would corrupt the message text.
#[test]
fn a_formal_named_in_a_plain_format_string_is_left_alone() {
    let out = pp("`define MSG(actual) $display(\"actual=%0d\", actual)\n`MSG(q)\n");
    assert_eq!(squash(&out), "$display(\"actual=%0d\", q)", "got {out:?}");
}

// ---------------------------------------------------------------------------
// #59 — an undefined macro was passed through with no diagnostic
// ---------------------------------------------------------------------------

/// `\`uvm_do_with` lives behind `UVM_ENABLE_DEPRECATED_API`, so without that
/// define it is genuinely undefined — but xezim passed the text through
/// silently and the user saw only ten "expected RParen, found Comma" errors at
/// a column of expanded text, naming neither the macro nor the reason.
///
/// The text still passes through (an unrecognised tool pragma has to survive to
/// the lexer), so what this asserts is that expansion is unchanged; the added
/// value is the stderr diagnostic naming the macro, which a unit test cannot
/// capture.
#[test]
fn an_undefined_macro_is_passed_through_unchanged() {
    let out = pp("`no_such_macro(a, {b==1;})\n");
    assert!(
        out.contains("`no_such_macro"),
        "text must survive for the lexer to report: {out:?}"
    );
}

/// A defined macro of the same shape expands normally — the diagnostic path is
/// not reached for anything that resolves.
#[test]
fn the_same_shape_expands_once_defined() {
    let out = pp(
        "`define do_with(a, c) begin a.randomize() with c; end\n`do_with(req, {req.wr_en==1;})\n",
    );
    assert_eq!(
        squash(&out),
        "begin req.randomize() with {req.wr_en==1;}; end",
        "got {out:?}"
    );
}

// ---------------------------------------------------------------------------
// #61 — out-of-class method definition with no explicit return type
// ---------------------------------------------------------------------------

/// §13.4 lets an out-of-class definition omit the return type. Seeing the `::`,
/// the parser assumed `pkg::type_t name(...)`, consumed `my_class::set_default`
/// as a scoped return TYPE, and then found `(` where the name should be:
/// "expected identifier, found LParen".
#[test]
fn out_of_class_function_without_a_return_type_parses() {
    let r = sv_parser::parse(
        "package pkg;\n\
           class my_class;\n\
             function void set_default();\n\
             endfunction\n\
           endclass\n\
           function my_class::set_default();\n\
             $display(\"set default\");\n\
           endfunction\n\
         endpackage\n",
    );
    assert!(r.errors.is_empty(), "unexpected errors: {:?}", r.errors);
}

/// The non-ANSI spelling, where a `;` rather than a `(` follows the name.
#[test]
fn out_of_class_function_without_ports_or_return_type_parses() {
    let r = sv_parser::parse(
        "class C;\n  function void m();\n  endfunction\nendclass\n\
         function C::m;\n  $display(\"m\");\nendfunction\n",
    );
    assert!(r.errors.is_empty(), "unexpected errors: {:?}", r.errors);
}

/// A genuine scoped return type must still be read as a TYPE, not as the name —
/// the token after the scoped name is the method name, not `(` or `;`.
#[test]
fn a_scoped_return_type_is_still_a_return_type() {
    let r = sv_parser::parse(
        "package pkg;\n  typedef enum {E0, E1} t_e;\nendpackage\n\
         module m;\n  import pkg::*;\n  function pkg::t_e f(); return E0; endfunction\n\
         endmodule\n",
    );
    assert!(r.errors.is_empty(), "unexpected errors: {:?}", r.errors);
}

/// An out-of-class constructor and an explicitly-typed out-of-class method,
/// both of which already worked, keep working alongside the untyped form.
#[test]
fn out_of_class_new_and_explicitly_typed_methods_still_parse() {
    let r = sv_parser::parse(
        "class C;\n  function void m2();\n  endfunction\n  function new();\n  endfunction\nendclass\n\
         function void C::m2();\nendfunction\n\
         function C::new();\nendfunction\n",
    );
    assert!(r.errors.is_empty(), "unexpected errors: {:?}", r.errors);
}

/// §22.5.1: `` is DELETED, not whitespace-eating glue. A `` with real
/// whitespace next to it keeps that whitespace (ivtest br979:
/// `localparam `` i``a``b``j` must yield `localparam i01j`, not
/// `localparami01j`); gluing only happens where the tokens were already
/// adjacent.
#[test]
fn double_backtick_does_not_eat_surrounding_whitespace() {
    let out = pp("`define my_macro(a,b) localparam `` i``a``b``j = 8'h``a``b;\n`my_macro(0,1)\n");
    assert_eq!(squash(&out), "localparam i01j = 8'h01;", "got {out:?}");
}
