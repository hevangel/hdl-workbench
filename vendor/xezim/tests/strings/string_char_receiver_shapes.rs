//! §6.16: a character-index write `s[i] = c` splices one character only
//! when the receiver `s` is a SCALAR string. A select on a string collection,
//! or on a partial element of a multi-dimensional one, writes a whole element
//! (§7.4, §7.8, §7.10), however the collection is reached. Expected values
//! come from the reference simulator.

use xezim::simulate;

/// Run `src` and compare its `T|` lines with `expected`.
fn assert_t_lines(src: &str, expected: &[&str]) {
    let sim = simulate(src, 100).expect("simulate failed");
    let got: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(got, expected, "T| lines differ");
}

/// `c.b.q[1]`, `c.b.aa["k"]`, `this.b.sa[1]` / `this.b.aa["t"]` in a method
/// and `h[0].q[0]` write whole elements of a string collection.
#[test]
fn whole_element_writes_through_nested_handles() {
    let src = r#"
class B;
  string sa[3];
  string q[$];
  string aa[string];
endclass
class C;
  B b;
  function new(); b = new(); endfunction
  function void m(); this.b.sa[1] = "this_b"; this.b.aa["t"] = "thisaa"; endfunction
endclass
module top;
  C c;
  B h[2];
  initial begin
    c = new();
    c.b.q.push_back("q0"); c.b.q.push_back("q1");
    c.b.q[1] = "Q1new";
    c.b.aa["k"] = "aaval";
    $display("T|c.b q '%s' '%s' aa '%s' n=%0d", c.b.q[0], c.b.q[1], c.b.aa["k"], c.b.aa.num());
    c.m();
    $display("T|m '%s' '%s'", c.b.sa[1], c.b.aa["t"]);
    h[0] = new(); h[0].q.push_back("x"); h[0].q[0] = "hq";
    $display("T|h '%s' n=%0d", h[0].q[0], h[0].q.size());
  end
endmodule
"#;
    assert_t_lines(
        src,
        &[
            "T|c.b q 'q0' 'Q1new' aa 'aaval' n=1",
            "T|m 'this_b' 'thisaa'",
            "T|h 'hq' n=1",
        ],
    );
}

/// `s2[1][2] = "x"` writes an element of `string s2[2][3]`; `s2[1][2][0]`
/// writes a character of it. `sq[1][0]` is an element of a queue element.
#[test]
fn multi_dimensional_string_array_elements() {
    let src = r#"
module top;
  string s2[2][3];
  string sq[2][$];
  initial begin
    s2[1][2] = "twodim"; s2[0][0] = "zero";
    $display("T|s2 eq=%0d len=%0d eq0=%0d", s2[1][2] == "twodim", s2[1][2].len(), s2[0][0] == "zero");
    s2[1][2][0] = "T";
    $display("T|s2 char eq=%0d len=%0d", s2[1][2] == "Twodim", s2[1][2].len());
    sq[1].push_back("a"); sq[1][0] = "qq2d";
    $display("T|sq eq=%0d n=%0d", sq[1][0] == "qq2d", sq[1].size());
  end
endmodule
"#;
    assert_t_lines(
        src,
        &[
            "T|s2 eq=1 len=6 eq0=1",
            "T|s2 char eq=1 len=6",
            "T|sq eq=1 n=1",
        ],
    );
}

/// Character writes into scalar strings reached through a static property
/// (`C::ss`), a handle property inside a method (`b.s`, `this.b.m2[1][2]`),
/// a handle whose class property shares a module collection's name
/// (`c.sa`), struct members, a typedef'd string array and a local
/// two-dimensional array.
#[test]
fn scalar_string_receivers_reached_by_path() {
    let src = r#"
class B;
  string s;
  string m2[2][3];
endclass
class C;
  B b;
  static string ss;
  string sa;
  function new(); b = new(); endfunction
  function void w();
    b.s = "method"; b.s[0] = "M";
    this.b.m2[1][2] = "thism2"; this.b.m2[1][2][0] = "T";
  endfunction
endclass
typedef string str3_t[3];
typedef struct { string s; string sa[2]; } st_t;
module top;
  string sa[4];
  C c;
  st_t st;
  str3_t ts;
  function automatic string f();
    string l2[2][2];
    l2[1][0] = "loc2d"; l2[1][0][4] = "D";
    return l2[1][0];
  endfunction
  initial begin
    c = new();
    C::ss = "static"; C::ss[0] = "S";
    $display("T|C::ss '%s'", C::ss);
    c.w();
    $display("T|w '%s' m2 eq=%0d", c.b.s, c.b.m2[1][2] == "Thism2");
    c.sa = "member"; c.sa[0] = "M"; sa[1] = "modsa";
    $display("T|c.sa '%s' sa '%s'", c.sa, sa[1]);
    st.s = "struct"; st.s[0] = "S"; st.sa[1] = "stsa1x"; st.sa[1][1] = "!";
    $display("T|st '%s' '%s' n=%0d", st.s, st.sa[1], st.sa[0].len());
    ts[2] = "tdef"; ts[2][0] = "T";
    $display("T|ts eq=%0d n=%0d", ts[2] == "Tdef", ts[0].len());
    $display("T|f '%s'", f());
  end
endmodule
"#;
    assert_t_lines(
        src,
        &[
            "T|C::ss 'Static'",
            "T|w 'Method' m2 eq=1",
            "T|c.sa 'Member' sa 'modsa'",
            "T|st 'Struct' 's!sa1x' n=0",
            "T|ts eq=1 n=0",
            "T|f 'loc2D'",
        ],
    );
}
