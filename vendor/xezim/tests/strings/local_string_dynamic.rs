//! §6.16 — a `string` declared INSIDE a procedural block is as dynamic as a
//! module-level one. GitHub issue #64.
//!
//! The batch-18 fix exempted string-typed SIGNALS from the width fit via the
//! id-indexed `signal_is_string` vec — but a procedural local has no signal-
//! table id. Its writes land on the widths-map fallback store in
//! `assign_value`, whose `widths` entry is the 1024-bit placeholder
//! `resolve_type_width` hands the dynamic type, and `val.resize(1024)`
//! truncates from the TOP — for a packed string that is the FRONT of the
//! text. So a local capped at 128 characters AND shifted one byte left on the
//! 129th append: `$system(cmd)` ran "cho …" instead of "echo …", failing with
//! a shell error while the testbench looked correct.
//!
//! Verified byte-identical to a reference simulator (probes g1/g2), including
//! `$bits` of the string tracking its dynamic size.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The reported case: build past 128 chars in a LOCAL, front intact.
#[test]
fn local_string_grows_past_128_characters() {
    let src = r#"
module top;
  int llen, cat_len, front_ok, tail_ok;
  initial begin
    string loc;
    string cat;
    loc = "echo SAFE";
    repeat (130) loc = {loc, "Y"};
    llen = loc.len();
    front_ok = (loc.substr(0, 3) == "echo");
    tail_ok  = (loc.substr(loc.len()-1, loc.len()-1) == "Y");
    cat = {loc, "!"};
    cat_len = cat.len();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "llen"), 139, "9 + 130 appends");
    assert_eq!(u(&sim, "front_ok"), 1, "the FRONT of the text survives");
    assert_eq!(u(&sim, "tail_ok"), 1, "and the tail");
    assert_eq!(u(&sim, "cat_len"), 140, "concat keeps growing");
}

/// The exact boundary from the issue: 127 / 128 / 129 characters.
#[test]
fn the_128_character_boundary() {
    let src = r#"
module top;
  int l127, l128, l129, first_129;
  initial begin
    string s;
    string b;
    string o;
    s = "echo SAFE";
    repeat (118) s = {s, "Y"};
    l127 = s.len();
    b = {s, "Z"};
    l128 = b.len();
    o = {b, "!"};
    l129 = o.len();
    first_129 = o.getc(0);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "l127"), 127);
    assert_eq!(u(&sim, "l128"), 128);
    assert_eq!(u(&sim, "l129"), 129, "129th char must not be dropped");
    assert_eq!(
        u(&sim, "first_129"),
        b'e' as u64,
        "\"echo\" must not become \"cho\""
    );
}

/// The guards: a non-string local still fits to its declared width, and a
/// same-named non-string local after a string local is not misclassified.
#[test]
fn non_string_locals_still_fit_to_declared_width() {
    let src = r#"
module top;
  int narrow, wide_in;
  initial begin
    logic [7:0] v;
    v = 16'hABCD;          // must truncate to 8 bits
    narrow = v;
    wide_in = 32'h1234_5678;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "narrow"), 0xCD, "a sized local still truncates");
    assert_eq!(u(&sim, "wide_in") as u32, 0x1234_5678);
}

/// Issue #65 (follow-up to #64): a MODULE-level `string` DECLARATION
/// initializer past 128 characters. The runtime stores were already exempt
/// from the placeholder-width fit; the declaration-elaboration path still fit
/// the initial value to 1024 bits, so the front of the text was lost before
/// time 0 — while the identical procedural assignment was correct, which is
/// exactly what made it look "already fixed".
#[test]
fn module_level_declaration_initializer_past_128_chars() {
    let long_a = format!("echo_SAFE_{}_END", "A".repeat(130));
    let src = format!(
        r#"
module top;
  string cmd1 = "{long_a}";
  int l1, p1, tail1;
  initial begin
    l1 = cmd1.len();
    p1 = (cmd1.substr(0, 9) == "echo_SAFE_");
    tail1 = (cmd1.substr(cmd1.len()-4, cmd1.len()-1) == "_END");
  end
endmodule
"#
    );
    let sim = simulate(&src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "l1"), 144, "the full initializer survives");
    assert_eq!(u(&sim, "p1"), 1, "the FRONT of the text survives");
    assert_eq!(u(&sim, "tail1"), 1, "and the tail");
}
