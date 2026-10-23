//! `$countones` in constraints (§20.9) and the axi4 AVIP's write strobes.
//!
//! `$countones(x)` of a whole rand variable was modelled only by splitting
//! the variable into single bits whose sum is the count. A relation whose
//! other side is not linear (`$countones(strb[i]) == 2**size`, from the
//! axi4 AVIP's `axi4_master_tx`) then left every bit to be guessed against
//! the evaluator, so `randomize()` failed; and a queue of 256 64-bit strobes
//! did not fit the solver as single bits at all.
//!
//! The count is now an `int` solver variable in [0, width] (§20.9), tied to
//! its variable. A relation on it is linear or, with a non-linear other
//! side, evaluated once the variables it reads are fixed; the variable is
//! decided after its count and drawn with exactly that many ones.
//!
//! The reference simulator solves every class below except `c10` (no 64-bit
//! value has 70 ones). Random values differ between simulators, so each line
//! reports the `randomize()` result and whether the constraints hold.

use xezim::simulate;

const SRC: &str = r#"
typedef enum bit [2:0] {S1, S2, S4, S8, S16, S32, S64, S128} sz_e;
class c1; rand bit [63:0] x; constraint k { $countones(x) == 4; } function int ok(); return $countones(x) == 4; endfunction endclass
class c2; rand bit [63:0] x; rand bit [7:0] n; constraint k { $countones(x) == n; n < 3; } function int ok(); return $countones(x) == n && n < 3; endfunction endclass
class c3; rand bit [31:0] x; constraint k { $countones(x) < 3; } function int ok(); return $countones(x) < 3; endfunction endclass
class c4; rand bit [15:0] x; rand bit [2:0] sz; constraint k { $countones(x) == 2**sz; } function int ok(); return $countones(x) == 2**sz && sz <= 4; endfunction endclass
class c5; rand bit [63:0] x; constraint k { x < 1000; $countones(x) == 5; } function int ok(); return x < 1000 && $countones(x) == 5; endfunction endclass
class c6; rand int x; constraint k { $countones(x) == 31; } function int ok(); return $countones(x) == 31; endfunction endclass
class c7; rand bit [7:0] a[4]; rand sz_e sz; constraint k { foreach (a[i]) { a[i] != 0; $countones(a[i]) == 2**sz; } } function int ok(); ok = sz <= S8; foreach (a[i]) ok &= $countones(a[i]) == 2**sz; endfunction endclass
class c8; rand bit [7:0] x; constraint k { $countones(x) inside {[2:3]}; } function int ok(); return $countones(x) inside {[2:3]}; endfunction endclass
class c9; rand bit [15:0] x, y; constraint k { $countones(x) == $countones(y); y == 16'h00ff; } function int ok(); return $countones(x) == 8; endfunction endclass
class c10; rand bit [63:0] x; constraint k { $countones(x) == 70; } function int ok(); return 0; endfunction endclass
class c11; rand bit [7:0] x; rand bit m; constraint k { if (m) $countones(x) == 1; else $countones(x) == 8; } function int ok(); return m ? $countones(x) == 1 : x == 8'hff; endfunction endclass
class c12; rand bit [7:0] len; rand sz_e sz; rand bit [63:0] strb[$]; rand bit [511:0] data[$];
  constraint k { len inside {[0:15]}; data.size() == len + 1; strb.size() == len + 1; foreach (strb[i]) strb[i] != 0; foreach (strb[i]) $countones(strb[i]) == 2**sz; }
  function int ok(); ok = strb.size() == len + 1 && data.size() == len + 1 && sz <= S64; foreach (strb[i]) ok &= $countones(strb[i]) == 2**sz; endfunction endclass
class c13; rand bit [31:0] x; rand bit [3:0] p; constraint k { x[p] == 1; $countones(x) == 1; p > 9; } function int ok(); return x[p] == 1 && $countones(x) == 1 && p > 9; endfunction endclass
class c14; rand bit [15:0] x; constraint k { $countones(x[7:0]) == 2; $countones(x) == 3; } function int ok(); return $countones(x[7:0]) == 2 && $countones(x) == 3; endfunction endclass
module top;
  c1 o1 = new(); c2 o2 = new(); c3 o3 = new(); c4 o4 = new(); c5 o5 = new(); c6 o6 = new(); c7 o7 = new();
  c8 o8 = new(); c9 o9 = new(); c10 o10 = new(); c11 o11 = new(); c12 o12 = new(); c13 o13 = new(); c14 o14 = new();
  initial begin
    int r;
    for (int t = 0; t < 3; t++) begin
      r = o1.randomize(); $display("c1 r=%0d ok=%0d", r, o1.ok());
      r = o2.randomize(); $display("c2 r=%0d ok=%0d", r, o2.ok());
      r = o3.randomize(); $display("c3 r=%0d ok=%0d", r, o3.ok());
      r = o4.randomize(); $display("c4 r=%0d ok=%0d", r, o4.ok());
      r = o5.randomize(); $display("c5 r=%0d ok=%0d", r, o5.ok());
      r = o6.randomize(); $display("c6 r=%0d ok=%0d", r, o6.ok());
      r = o7.randomize(); $display("c7 r=%0d ok=%0d", r, o7.ok());
      r = o8.randomize(); $display("c8 r=%0d ok=%0d", r, o8.ok());
      r = o9.randomize(); $display("c9 r=%0d ok=%0d", r, o9.ok());
      r = o10.randomize(); $display("c10 r=%0d", r);
      r = o11.randomize(); $display("c11 r=%0d ok=%0d", r, o11.ok());
      r = o12.randomize(); $display("c12 r=%0d ok=%0d", r, o12.ok());
      r = o13.randomize(); $display("c13 r=%0d ok=%0d", r, o13.ok());
      r = o14.randomize(); $display("c14 r=%0d ok=%0d", r, o14.ok());
    end
  end
endmodule
"#;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.contains(" r="))
        .collect()
}

#[test]
fn countones_count_variable_randomize() {
    let mut expected = Vec::new();
    for _ in 0..3 {
        for c in 1..=14 {
            expected.push(if c == 10 {
                "c10 r=0".to_string()
            } else {
                format!("c{c} r=1 ok=1")
            });
        }
    }
    assert_eq!(lines(SRC), expected);
}

/// §18.5.10: every solution is equally likely. The count of an 8-bit
/// variable follows C(8, n); a size whose strobe must have 2**s ones comes
/// up in proportion to C(8, 2**s) (8 : 28 : 70 : 1); a dist on the counted
/// variable keeps its weights. The reference simulator over 2000 calls:
/// zeros=1808, counts {0, 72, 226, 426, 539, 457, 216, 64, 0}, sizes {159,
/// 530, 1299, 12, 0, 0, 0, 0}; the bounds below hold with a wide margin.
#[test]
fn countones_distribution() {
    let src = r#"
class d1; rand bit [15:0] z; constraint k { z dist {0 :/ 9, [1:16'hFFFF] :/ 1}; $countones(z) <= 15; } endclass
class d2; rand bit [7:0] x; constraint k { $countones(x) inside {[1:7]}; } endclass
class d3; rand bit [7:0] x; rand bit [2:0] s; constraint k { $countones(x) == 2**s; } endclass
module top; d1 a = new(); d2 b = new(); d3 c = new(); int zeros, ok, h[9], hs[8];
  initial begin
    for (int t = 0; t < 2000; t++) begin
      ok += a.randomize(); zeros += (a.z == 0);
      void'(b.randomize()); h[$countones(b.x)]++;
      void'(c.randomize()); hs[c.s]++;
    end
    $display("d1 ok=%0d zeros=%0d", ok, zeros);
    $display("d2 %p", h);
    $display("d3 %p", hs);
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let nums = |prefix: &str| -> Vec<i64> {
        let l = out
            .iter()
            .find(|m| m.starts_with(prefix))
            .unwrap_or_else(|| panic!("{prefix}: {out:?}"));
        l[prefix.len()..]
            .split(|c: char| !c.is_ascii_digit())
            .filter(|t| !t.is_empty())
            .map(|t| t.parse().unwrap())
            .collect()
    };
    let d1 = nums("d1 ");
    assert_eq!(d1[0], 2000, "{out:?}");
    assert!(d1[1] > 1700, "{out:?}");
    let d2 = nums("d2 ");
    let want = [0, 63, 220, 441, 551, 441, 220, 63, 0];
    for (n, (&got, &w)) in d2.iter().zip(want.iter()).enumerate() {
        assert!((got - w).abs() <= 120, "count {n}: {out:?}");
    }
    let d3 = nums("d3 ");
    assert!((90..=230).contains(&d3[0]), "{out:?}");
    assert!((420..=640).contains(&d3[1]), "{out:?}");
    assert!((1170..=1430).contains(&d3[2]), "{out:?}");
    assert!(d3[3] <= 45 && d3[4..].iter().all(|&x| x == 0), "{out:?}");
}
