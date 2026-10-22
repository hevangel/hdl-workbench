//! Regression: `$size`/`$left`/etc. on a FIXED-SIZE STRING ARRAY must report
//! the DECLARED element count, not the current string value's length.
//!
//! `$size(s)` on a scalar `string s` tracks the current length (§7.11), but
//! the same query on a fixed array of `string` elements (`string[16]`) reports
//! the array dimension. xezim's string-length shortcut fired for BOTH, reading
//! the (empty, all-zero default) array as a single string and returning 0 for
//! something like `$size(sarray_string)` — which broke UVM config_db matching
//! on `sarray_string[13]` ("Index '13' is not valid for static array of size
//! '0'") and missed the configured value.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// A fixed-size string-array CLASS MEMBER reports its declared count; the
/// sibling `int` member matches, and a scalar string still reports its length.
#[test]
fn size_of_fixed_string_array_class_member() {
    let src = r#"
class S;
  string a[16];
  int    b[16];
  string sc;
  function void show();
    sz = $size(a);
    sb = $size(b);
    ssc = $size(sc);
    sl = $left(sc); sr = $right(sc);
  endfunction
  int sz, sb, ssc, sl, sr;
endclass
module tb;
  int sz, sb, ssc, sl, sr;
  initial begin
    S s = new();
    s.sc = "Hello";
    s.show();
    sz = s.sz; sb = s.sb; ssc = s.ssc; sl = s.sl; sr = s.sr;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "sz"), 16, "$size(string[16]) is the declared count");
    assert_eq!(u(&sim, "sb"), 16, "$size(int[16]) is the declared count");
    assert_eq!(u(&sim, "ssc"), 5, "scalar string $size is its length");
    assert_eq!(u(&sim, "sl"), 0, "scalar string $left is 0");
    assert_eq!(u(&sim, "sr"), 4, "scalar string $right is len-1");
}

/// The string-length-shortcut veto must hold for EVERY array-query function
/// ($left/$right/$low/$high), which share the string shortcut the fix guards.
/// On a fixed `string[16]` member they report the DECLARED bounds (0 15 0 15)
/// — the string-length fallback would have leaked -1/0 for a fresh empty array.
#[test]
fn bounds_of_fixed_string_array_class_member() {
    let src = r#"
class S;
  string a[16];
  int    b[16];
  function void show();
    l0=$left(a); r0=$right(a); lo0=$low(a); hi0=$high(a);
    lb=$left(b); rb=$right(b); lob=$low(b); hib=$high(b);
  endfunction
  int l0,r0,lo0,hi0,lb,rb,lob,hib;
endclass
module tb;
  int l0,r0,lo0,hi0,lb,rb,lob,hib;
  initial begin
    S s = new();
    s.show();
    l0=s.l0; r0=s.r0; lo0=s.lo0; hi0=s.hi0;
    lb=s.lb; rb=s.rb; lob=s.lob; hib=s.hib;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "l0"), 0, "$left(string[16]) is 0");
    assert_eq!(u(&sim, "r0"), 15, "$right(string[16]) is 15");
    assert_eq!(u(&sim, "lo0"), 0, "$low(string[16]) is 0");
    assert_eq!(u(&sim, "hi0"), 15, "$high(string[16]) is 15");
    assert_eq!(u(&sim, "lb"), 0, "$left(int[16]) is 0");
    assert_eq!(u(&sim, "rb"), 15, "$right(int[16]) is 15");
    assert_eq!(u(&sim, "lob"), 0, "$low(int[16]) is 0");
    assert_eq!(u(&sim, "hib"), 15, "$high(int[16]) is 15");
}

/// A module-scope fixed `string[8]` also reports its declared count.
#[test]
fn size_of_fixed_string_array_module_scope() {
    let src = r#"
module tb;
  string g[8];
  int s;
  initial begin
    s = $size(g);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "s"), 8, "$size(module string[8]) is 8");
}
