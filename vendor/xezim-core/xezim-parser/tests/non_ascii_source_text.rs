//! IEEE 1800-2017 §5.9: a string literal is the sequence of its source
//! BYTES. The preprocessor's comment stripper re-encoded every non-ASCII
//! source byte as the Latin-1 char of the same value, so UTF-8 text came out
//! double-encoded: `"é"` (bytes C3 A9) reached the lexer as `"Ã©"`, and the
//! string-literal decoder then produced four bytes (C3 83 C2 A9) instead of
//! two. Ordinary text, in and out of string literals, is now copied intact.

use sv_parser::preprocessor::Preprocessor;

fn pp(src: &str) -> String {
    Preprocessor::new().preprocess(src)
}

#[test]
fn non_ascii_text_survives_preprocessing_unchanged() {
    let src = "module top; // café\n  initial $display(\"é\\xff\\é\"); /* ü */\nendmodule\n";
    let out = pp(src);
    assert!(out.contains("$display(\"é\\xff\\é\")"), "{out:?}");
    assert!(!out.contains('Ã'), "double-encoded text: {out:?}");
    // Comments still blank out byte for byte, keeping lines intact.
    assert_eq!(out.lines().count(), src.lines().count(), "{out:?}");
}

#[test]
fn non_ascii_string_literal_decodes_to_its_source_bytes() {
    let src = "module top; initial $display(\"é\\xff\"); endmodule\n";
    let parsed = sv_parser::parse(&pp(src));
    let dump = format!("{:?}", parsed.source);
    // Decoded literal text holds one Latin-1 char per byte: C3 A9 FF.
    assert!(dump.contains("StringLiteral(\"Ã©ÿ\")"), "{dump}");
}
