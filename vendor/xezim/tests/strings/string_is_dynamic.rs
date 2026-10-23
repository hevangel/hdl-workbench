//! §6.16 — `string` is a DYNAMIC type: it has no declared length, and an
//! assignment never truncates it.
//!
//! xezim stores every variable in a fixed-width signal table, and
//! `resolve_type_width` hands `string` a placeholder of 1024 bits — exactly
//! 128 characters. Assignment then fit the value to that width, so any longer
//! string was silently cut. Worse, a packed string keeps its text in the LOW
//! bits, so resizing down dropped the HIGH bits — the FRONT of the text. A
//! 130-character build-up came back as 128 characters, and
//! `result = {base, "TAIL"}` lost the suffix it was appending.
//!
//! String ids are now exempt from the width fit and keep whatever length they
//! are given. Verified byte-identical to a reference simulator, including
//! indexing, `substr`, `getc`/`putc`, comparison, and a 1000-character value.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The reported case: iterative concat past 128 characters, and a suffix
/// appended to an already-long string.
#[test]
fn concat_grows_past_the_backing_width() {
    let src = r#"
module top;
  string base, result;
  int base_len, res_len, first_c, last_c;
  initial begin
    base = "";
    repeat (130) base = {base, "X"};
    result = {base, "TAIL"};
    base_len = base.len();
    res_len  = result.len();
    first_c  = result[0];
    last_c   = result[133];
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "base_len"),
        130,
        "130 appends must give 130 chars, not 128"
    );
    assert_eq!(u(&sim, "res_len"), 134, "appending TAIL gives 134");
    assert_eq!(
        u(&sim, "first_c"),
        b'X' as u64,
        "the FRONT of the text survives"
    );
    assert_eq!(
        u(&sim, "last_c"),
        b'L' as u64,
        "and so does the appended suffix"
    );
}

/// The appended suffix is intact as text, not merely as a length.
#[test]
fn appended_suffix_is_preserved() {
    let src = r#"
module top;
  string base, result, tail;
  int ok;
  initial begin
    base = "";
    repeat (130) base = {base, "X"};
    result = {base, "TAIL"};
    tail = result.substr(130, 133);
    ok = (tail == "TAIL");
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "ok"), 1, "substr of the tail must read back TAIL");
}

/// Well past any plausible backing width, and copying a long string keeps it.
#[test]
fn long_strings_survive_assignment_and_copy() {
    let src = r#"
module top;
  string big, copy;
  int big_len, copy_len, same;
  initial begin
    big = "";
    repeat (1000) big = {big, "a"};
    copy = big;
    big_len  = big.len();
    copy_len = copy.len();
    same     = (copy == big);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "big_len"),
        1000,
        "1000-char string keeps its length"
    );
    assert_eq!(u(&sim, "copy_len"), 1000, "and survives a plain assignment");
    assert_eq!(u(&sim, "same"), 1, "the copy compares equal");
}

/// §6.16.2 methods still behave on ordinary short strings — the exemption must
/// not disturb the common case.
#[test]
fn short_string_methods_unaffected() {
    let src = r#"
module top;
  string s;
  int len_n, gc, sub_ok, put_ok, up_ok, atoi_n;
  initial begin
    s = "Hello";
    len_n = s.len();
    gc    = s.getc(1);
    sub_ok = (s.substr(1, 3) == "ell");
    up_ok  = (s.toupper() == "HELLO");
    s.putc(0, "J");
    put_ok = (s == "Jello");
    s = "12345";
    atoi_n = s.atoi();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "len_n"), 5);
    assert_eq!(u(&sim, "gc"), b'e' as u64, "getc(1) is 'e'");
    assert_eq!(u(&sim, "sub_ok"), 1, "substr");
    assert_eq!(u(&sim, "up_ok"), 1, "toupper");
    assert_eq!(u(&sim, "put_ok"), 1, "putc");
    assert_eq!(u(&sim, "atoi_n"), 12345, "atoi");
}

/// Elements of string COLLECTIONS are strings too. `string_signals` holds the
/// base name (`q`, `aa`) while the table holds `q[2]`, so the subscript has to
/// be stripped when marking ids — and `push_back` stores through
/// `set_signal_value_by_name`, a different path from ordinary assignment.
/// Missing either left a `string q[$]` element clamped to 128 characters after
/// scalar strings had already been fixed.
#[test]
fn string_collection_elements_are_dynamic_too() {
    let src = r#"
module top;
  string big;
  string q [$];
  string aa [string];
  string arr [2];
  string q_copy;
  int q_len, aa_len, arr_len, q_tail_ok;
  initial begin
    big = "";
    repeat (200) big = {big, "y"};
    q.push_back(big);
    aa["k"]  = big;
    arr[1]   = big;
    q_len    = q[0].len();
    aa_len   = aa["k"].len();
    arr_len  = arr[1].len();
    // Copy out before inspecting: string METHODS other than `len()` do not
    // work directly on a queue element (pre-existing, unrelated to the width
    // fix — a 5-char element behaves the same way). Copying exercises the
    // stored text end-to-end, which is what this test is about.
    q_copy    = q[0];
    q_tail_ok = (q_copy.substr(197, 199) == "yyy");
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "q_len"),
        200,
        "queue element keeps 200 chars (push_back path)"
    );
    assert_eq!(u(&sim, "aa_len"), 200, "assoc element keeps 200 chars");
    assert_eq!(
        u(&sim, "arr_len"),
        200,
        "fixed-array element keeps 200 chars"
    );
    assert_eq!(u(&sim, "q_tail_ok"), 1, "the end of the text is intact");
}
