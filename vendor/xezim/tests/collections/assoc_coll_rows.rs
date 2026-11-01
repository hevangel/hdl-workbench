//! IEEE 1800-2017 §7.8, §7.10, §21.2.1.7: an associative array whose
//! elements are collections. A write into an absent row (`aq[6][0] = 9`)
//! creates the row as its declared kind, `%p` prints each row as a nested
//! list, `foreach (aq[k][j])` walks the row, and a string-valued row takes
//! `m[k]["x"] = "hi"` as an element write (it was a character write and was
//! dropped). Expected values use the house `%p` spacing (no
//! space before an associative array's `}`).
use xezim::simulate;

/// The `$display` lines of `src` that start with one of `prefixes`.
fn shown(src: &str, prefixes: &[&str]) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate")
        .output
        .iter()
        .map(|l| l.message.clone())
        .filter(|m| prefixes.iter().any(|p| m.starts_with(p)))
        .collect()
}

/// Queue, associative and string-keyed rows.
#[test]
fn absent_rows_are_created_and_printed() {
    let src = r#"module tb;
  int aq [int][$];
  int aa [int][int];
  int ad [string][];
  string as [int][string];
  initial begin
    aq[6][0] = 9; $display("A %p size=%0d n=%0d", aq, aq[6].size(), aq.num());
    aq[6][1] = 4; aq[2].push_back(1); $display("B %p", aq);
    aa[1][2] = 3; aa[4][5] = 6; aa[1][7] = 8; $display("C %p %0d %0d", aa, aa.num(), aa[1].num());
    as[3]["x"] = "hi"; $display("D %p", as);
    $display("E %0d %0d", aa[1][7], aq[6][1]);
  end
endmodule
"#;
    let want = r#"A '{6:'{9}} size=1 n=1
B '{2:'{1}, 6:'{9, 4}}
C '{1:'{2:3, 7:8}, 4:'{5:6}} 2 2
D '{3:'{"x":"hi"}}
E 8 4"#;
    let got = shown(src, &["A ", "B ", "C ", "D ", "E "]);
    assert_eq!(got.join("\n"), want, "{got:#?}");
}

/// `foreach (aq[6][j])` binds `j` to the row's indices.
#[test]
fn foreach_walks_an_associative_row() {
    let src = r#"module tb;
  int aq [int][$];
  initial begin
    aq[6][0] = 9;
    $display("A %0d %0d %0d", aq[6].size(), aq[6][0], aq.num());
    foreach (aq[k]) $display("K %0d", k);
    foreach (aq[6][j]) $display("J %0d %0d", j, aq[6][j]);
  end
endmodule
"#;
    let want = r#"A 1 9 1
K 6
J 0 9"#;
    let got = shown(src, &["A ", "K ", "J "]);
    assert_eq!(got.join("\n"), want, "{got:#?}");
}

/// String elements under integer and string keys, and a queue row.
#[test]
fn string_rows_take_element_writes() {
    let src = r#"module tb;
  string a1 [int][string];
  int    a2 [int][string];
  string a3 [string][int];
  string a4 [int][int];
  string a5 [int][$];
  initial begin
    a1[3]["x"] = "hi"; a2[3]["x"] = 5; a3["k"][2] = "s3"; a4[1][2] = "s4"; a5[1].push_back("q");
    $display("A %s %0d | %0d %0d | %s %0d | %s %0d | %s %0d", a1[3]["x"], a1[3].num(), a2[3]["x"], a2[3].num(), a3["k"][2], a3["k"].num(), a4[1][2], a4[1].num(), a5[1][0], a5[1].size());
    $display("P %p %p %p %p %p", a1, a2, a3, a4, a5);
  end
endmodule
"#;
    let want = r#"A hi 1 | 5 1 | s3 1 | s4 1 | q 1
P '{3:'{"x":"hi"}} '{3:'{"x":5}} '{"k":'{2:"s3"}} '{1:'{2:"s4"}} '{1:'{"q"}}"#;
    let got = shown(src, &["A ", "P "]);
    assert_eq!(got.join("\n"), want, "{got:#?}");
}

/// String and integer keys at each level, three and four levels, `exists`,
/// `num`, `delete` and multi-variable `foreach`: each row keys its own
/// elements by its declared key type, and `%p` prints every level.
#[test]
fn nested_rows_keep_their_key_types() {
    let src = r#"module tb;
  int a1 [string][int];
  int a2 [int][string];
  int a3 [string][string];
  int a4 [int][int];
  byte a5 [string][int][string];
  initial begin
    a1["u"][2] = 8; a1["u"][300] = 1; a1["v"][-4] = 3;
    a2[5]["ab"] = 1; a2[5]["c"] = 2; a2[-1]["zz"] = 3;
    a3["k"]["m"] = 4; a3["k"]["n"] = 5;
    a4[1][2] = 6; a4[1][3] = 7; a4[9][0] = 8;
    a5["p"][7]["q"] = 9; a5["p"][8]["r"] = 10;
    foreach (a1[i, j]) $display("A %s %0d %0d", i, j, a1[i][j]);
    foreach (a2[i, j]) $display("B %0d %s %0d", i, j, a2[i][j]);
    foreach (a3[i, j]) $display("C %s %s %0d", i, j, a3[i][j]);
    foreach (a4[i, j]) $display("D %0d %0d %0d", i, j, a4[i][j]);
    foreach (a5[i, j, k]) $display("E %s %0d %s %0d", i, j, k, a5[i][j][k]);
    $display("F %0d %0d %0d %0d", a1["u"].exists(300), a1["u"].exists(3), a2[5].num(), a5["p"].num());
    a1["u"].delete(2); a2.delete(-1);
    $display("G %p", a1); $display("H %p", a2); $display("I %p", a3); $display("J %p", a4); $display("K %p", a5);
    foreach (a2[5][j]) $display("L %s", j);
  end
endmodule
"#;
    let want = r#"A u 2 8
A u 300 1
A v -4 3
B -1 zz 3
B 5 ab 1
B 5 c 2
C k m 4
C k n 5
D 1 2 6
D 1 3 7
D 9 0 8
E p 7 q 9
E p 8 r 10
F 1 0 2 2
G '{"u":'{300:1}, "v":'{-4:3}}
H '{5:'{"ab":1, "c":2}}
I '{"k":'{"m":4, "n":5}}
J '{1:'{2:6, 3:7}, 9:'{0:8}}
K '{"p":'{7:'{"q":9}, 8:'{"r":10}}}
L ab
L c"#;
    let got = shown(
        src,
        &[
            "A ", "B ", "C ", "D ", "E ", "F ", "G ", "H ", "I ", "J ", "K ", "L ",
        ],
    );
    assert_eq!(got.join("\n"), want, "{got:#?}");
}

/// Deeper rows (a queue under two keys, four string levels), middle-level
/// methods and deletes, and the same shapes as class members written from
/// a method and through a handle.
#[test]
fn deep_and_class_member_rows() {
    let src = r#"class H;
  int m [int][string];
  int q [string][int][$];
  function void fill(); m[1]["a"] = 2; m[3]["b"] = 4; q["x"][5][0] = 7; q["x"][5][1] = 8; endfunction
endclass
module tb;
  int b1 [int][int][$];
  string b2 [string][string][string][string];
  shortint b3 [int][string][int];
  initial begin
    H h = new;
    b1[1][2][0] = 5; b1[1][2][1] = 6; b1[1][3].push_back(9); b1[4][0][0] = 1;
    b2["a"]["b"]["c"]["d"] = "deep"; b2["a"]["b"]["e"]["f"] = "two";
    b3[-2]["k"][100] = 3; b3[-2]["k"][-100] = -4; b3[7]["z"][0] = 1;
    $display("A %p", b1); $display("B %p", b2); $display("C %p", b3);
    $display("D %0d %0d %0d %0d", b1[1].num(), b1[1][2].size(), b3[-2]["k"].num(), b2["a"]["b"].num());
    $display("E %0d %0d", b3[-2].exists("k"), b3[-2]["k"].exists(5));
    b3[-2]["k"].delete(100); b1[1].delete(3);
    $display("F %p %p", b3, b1);
    foreach (b3[i, j, k]) $display("G %0d %s %0d %0d", i, j, k, b3[i][j][k]);
    foreach (b1[i, j, k]) $display("H %0d %0d %0d %0d", i, j, k, b1[i][j][k]);
    h.fill(); $display("I %p %p %0d", h.m, h.q, h.q["x"][5].size());
  end
endmodule
"#;
    let want = r#"A '{1:'{2:'{5, 6}, 3:'{9}}, 4:'{0:'{1}}}
B '{"a":'{"b":'{"c":'{"d":"deep"}, "e":'{"f":"two"}}}}
C '{-2:'{"k":'{-100:-4, 100:3}}, 7:'{"z":'{0:1}}}
D 2 2 2 2
E 1 0
F '{-2:'{"k":'{-100:-4}}, 7:'{"z":'{0:1}}} '{1:'{2:'{5, 6}}, 4:'{0:'{1}}}
G -2 k -100 -4
G 7 z 0 1
H 1 2 0 5
H 1 2 1 6
H 4 0 0 1
I '{1:'{"a":2}, 3:'{"b":4}} '{"x":'{5:'{7, 8}}} 2"#;
    let got = shown(src, &["A ", "B ", "C ", "D ", "E ", "F ", "G ", "H ", "I "]);
    assert_eq!(got.join("\n"), want, "{got:#?}");
}
