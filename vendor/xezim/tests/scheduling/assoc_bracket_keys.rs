//! §7.11 associative arrays keyed by STRING values that themselves contain
//! `[` and `]` — legal component/hierarchy names (e.g. UVM's
//! `special-:{ chars{}[0123456789] _`). `num()/first()/foreach` must keep the
//! FULL key, not stop at the first `]`.
//!
//! Bug: the assoc iteration key was extracted as the text from `arr[` up to the FIRST `]`. For a 1-D string key containing
//! `]`/`[` that slice truncated the key (`special-:{ chars{}[0123456789]`),
//! so `exists()`/`get_child` on the stored full key (and foreach) saw null and
//! UVM's component-tree walk fell into a NOCHILD → stack overflow. The
//! extraction now distinguishes a MULTIDIM/assoc-of-collection entry (`k1][k2]`,
//! inner `[` after the first `]`) from a 1-D string key (whole text to the
//! final `]`).

use xezim::simulate;

fn read_int(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn assoc_string_keys_with_square_brackets_iterate() {
    let src = r#"
`timescale 1ns/1ns
module tb;
  string cnt = "special-:{ chars{}[0123456789] _";
  string map[string];
  int iter_ok = 0;
  int nkeys = 0;
  int seen_bracket = 0;
  int seen_normal = 0;
  string kk;

  initial begin
    map["normal"] = "a";
    map[cnt] = "b";
    // foreach over the assoc must yield the FULL bracket key, and the value
    // must be found back for both keys.
    foreach (map[i]) begin
      if (i == cnt) seen_bracket = 1;
      if (i == "normal") seen_normal = 1;
      if (i == cnt && map.exists(i) && map[i] == "b") iter_ok = 1;
      if (i == "normal" && map.exists(i) && map[i] == "a") iter_ok = 1;
      nkeys++;
    end
    // first() must give the full key too (so exists() on it succeeds).
    if (map.first(kk) && map.exists(kk)) begin
      iter_ok = 1;
    end
    $display("ASSOC iter_ok=%0d seen_bracket=%0d seen_normal=%0d nkeys=%0d",
      iter_ok, seen_bracket, seen_normal, nkeys);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 2000).expect("simulate failed");
    assert_eq!(
        read_int(&sim, "iter_ok"),
        1,
        "assoc with a string key containing `]`/`[` must round-trip through foreach/first()/exists()"
    );
    assert_eq!(
        read_int(&sim, "seen_bracket"),
        1,
        "foreach must yield the full bracket-containing key"
    );
    assert_eq!(read_int(&sim, "seen_normal"), 1);
    assert_eq!(
        read_int(&sim, "nkeys"),
        2,
        "assoc must have exactly the two distinct keys"
    );
}
