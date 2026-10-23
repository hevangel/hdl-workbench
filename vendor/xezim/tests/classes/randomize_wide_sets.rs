//! §18.5.4 / §11.4.13: `dist` and `inside` on members wider than 64 bits,
//! the neighbours of #301.
//!
//! The joint solver splits such a member into segments of at most 64 bits.
//! A `dist` over the whole member was not modelled at all (`randomize()`
//! returned 0); it now constrains the member to its items, drawn uniformly
//! among them. `x inside {K1, K2, ...}` over constants was only the
//! disjunction of whole-value equalities, which acts once all but one
//! alternative are excluded, so with a 64-bit segment drawn at random the
//! search ran out of budget; each segment is now also limited to the bits
//! the values hold there.
//!
//! Expected values come from the reference simulator: every probe prints
//! whether its constraints hold, not the random values.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn wide_dist_and_inside_sets() {
    let src = include_str!("randomize_wide_sets.sv");
    let expected: Vec<String> = src
        .lines()
        .filter_map(|l| {
            let i = l.find("$display(\"T|")? + "$display(\"".len();
            let name = l[i..].split_whitespace().next()?;
            Some(format!("{name} r=8 ok=8"))
        })
        .collect();
    assert_eq!(expected.len(), 70);
    assert_eq!(t_lines(src), expected);
}

fn t_lines_of(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// The count after `key=` in a `T|` line.
fn count(line: &str, key: &str) -> u32 {
    line.split_whitespace()
        .find_map(|t| t.strip_prefix(key)?.parse().ok())
        .unwrap_or(u32::MAX)
}

/// §18.5.4: a `dist` over a wide member honours its weights (`:=` per
/// value, `:/` per item). The reference simulator draws 102/298 and
/// 106/294 of 400 for weights 1:3; any split near 100/300 is right.
#[test]
fn wide_dist_honours_its_weights() {
    let src = r#"
module top;
  class A; rand bit [71:0] x; constraint c { x dist {72'h9A0000000000000001 := 1, 72'h80FFFFFFFFFFFFFFFE := 3}; } endclass
  class B; rand bit [71:0] x; constraint c { x dist {[72'h400000000000000000:72'h4000000000000000FF] :/ 1, 72'h010000000000000000 := 3}; } endclass
  initial begin
    A a = new; B b = new; int n1, n2, n3, r, ok;
    n1 = 0; n2 = 0; ok = 0;
    repeat (400) begin r = a.randomize(); ok += r; if (a.x == 72'h9A0000000000000001) n1++; else if (a.x == 72'h80FFFFFFFFFFFFFFFE) n2++; end
    $display("T|a ok=%0d k1=%0d k2=%0d", ok, n1, n2);
    n1 = 0; n2 = 0; ok = 0;
    repeat (400) begin r = b.randomize(); ok += r; if (b.x >= 72'h400000000000000000 && b.x <= 72'h4000000000000000FF) n1++; else if (b.x == 72'h010000000000000000) n2++; end
    $display("T|b ok=%0d rng=%0d k3=%0d", ok, n1, n2);
  end
endmodule
"#;
    let got = t_lines_of(src);
    assert_eq!(got.len(), 2, "{got:?}");
    assert_eq!(count(&got[0], "ok="), 400, "{got:?}");
    assert_eq!(
        count(&got[0], "k1=") + count(&got[0], "k2="),
        400,
        "{got:?}"
    );
    assert!((60..=140).contains(&count(&got[0], "k1=")), "{got:?}");
    assert_eq!(count(&got[1], "ok="), 400, "{got:?}");
    assert_eq!(
        count(&got[1], "rng=") + count(&got[1], "k3="),
        400,
        "{got:?}"
    );
    assert!((60..=140).contains(&count(&got[1], "rng=")), "{got:?}");
}

/// `unique` over wide `std::randomize` locals and `y == x + K` over wide
/// members: every call succeeds (some trials gave up before).
#[test]
fn wide_unique_and_sums_never_give_up() {
    let src = r#"
module top;
  function automatic void p_std_72_uniq(); bit [71:0] x, y, z; bit m; int r = 0, k = 0; repeat (40) begin if (std::randomize(x, y, z, m) with { unique {x, y, z}; x inside {72'h9A0000000000000001, 72'h80FFFFFFFFFFFFFFFE, 72'h010000000000000000}; y inside {72'h9A0000000000000001, 72'h80FFFFFFFFFFFFFFFE, 72'h010000000000000000}; z inside {72'h9A0000000000000001, 72'h80FFFFFFFFFFFFFFFE, 72'h010000000000000000};  }) r++; if ((x != y && y != z && x != z && (x == 72'h9A0000000000000001 || x == 72'h80FFFFFFFFFFFFFFFE || x == 72'h010000000000000000) && (y == 72'h9A0000000000000001 || y == 72'h80FFFFFFFFFFFFFFFE || y == 72'h010000000000000000) && (z == 72'h9A0000000000000001 || z == 72'h80FFFFFFFFFFFFFFFE || z == 72'h010000000000000000))) k++; end $display("T|std_72_uniq r=%0d ok=%0d", r, k); endfunction
  function automatic void p_std_128_uniq(); bit [127:0] x, y, z; bit m; int r = 0, k = 0; repeat (40) begin if (std::randomize(x, y, z, m) with { unique {x, y, z}; x inside {128'h9A000000000000000000000000000001, 128'h80FFFFFFFFFFFFFFFFFFFFFFFFFFFFFE, 128'h01000000000000000000000000000000}; y inside {128'h9A000000000000000000000000000001, 128'h80FFFFFFFFFFFFFFFFFFFFFFFFFFFFFE, 128'h01000000000000000000000000000000}; z inside {128'h9A000000000000000000000000000001, 128'h80FFFFFFFFFFFFFFFFFFFFFFFFFFFFFE, 128'h01000000000000000000000000000000};  }) r++; if ((x != y && y != z && x != z && (x == 128'h9A000000000000000000000000000001 || x == 128'h80FFFFFFFFFFFFFFFFFFFFFFFFFFFFFE || x == 128'h01000000000000000000000000000000) && (y == 128'h9A000000000000000000000000000001 || y == 128'h80FFFFFFFFFFFFFFFFFFFFFFFFFFFFFE || y == 128'h01000000000000000000000000000000) && (z == 128'h9A000000000000000000000000000001 || z == 128'h80FFFFFFFFFFFFFFFFFFFFFFFFFFFFFE || z == 128'h01000000000000000000000000000000))) k++; end $display("T|std_128_uniq r=%0d ok=%0d", r, k); endfunction
  class C_uarr_128_arith; rand bit [127:0] v[3]; rand bit m; constraint c { v[1] == v[0] + 128'h01000000000000000000000000000000; v[0] < 128'h01000000000000000000000000000000; } function new();  endfunction function bit ok(); return (v[1] == v[0] + 128'h01000000000000000000000000000000 && v[0] < 128'h01000000000000000000000000000000); endfunction endclass
  class C_plain_128_arith; rand bit [127:0] x, y, z; rand bit m; constraint c { y == x + 128'h01000000000000000000000000000000; x < 128'h01000000000000000000000000000000; } function new();  endfunction function bit ok(); return (y == x + 128'h01000000000000000000000000000000 && x < 128'h01000000000000000000000000000000); endfunction endclass
  class B_sub_128_arith; rand bit [127:0] x, y, z; rand bit m; endclass
  class C_sub_128_arith; rand B_sub_128_arith a; constraint c { a.y == a.x + 128'h01000000000000000000000000000000; a.x < 128'h01000000000000000000000000000000; } function new(); a = new(); endfunction function bit ok(); return (a.y == a.x + 128'h01000000000000000000000000000000 && a.x < 128'h01000000000000000000000000000000); endfunction endclass
  initial begin
    p_std_72_uniq();
    p_std_128_uniq();
    begin C_uarr_128_arith o = new(); int r = 0, k = 0; repeat (40) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|uarr_128_arith r=%0d ok=%0d", r, k); end
    begin C_plain_128_arith o = new(); int r = 0, k = 0; repeat (40) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|plain_128_arith r=%0d ok=%0d", r, k); end
    begin C_sub_128_arith o = new(); int r = 0, k = 0; repeat (40) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|sub_128_arith r=%0d ok=%0d", r, k); end
  end
endmodule
"#;
    let got = t_lines_of(src);
    assert_eq!(got.len(), 5, "{got:?}");
    for l in &got {
        assert!(l.ends_with("r=40 ok=40"), "{got:?}");
    }
}
