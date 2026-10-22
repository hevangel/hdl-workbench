//! IEEE 1800-2023 §8.5 / §23.9: inside a class method a bare property name is that property even when an unpacked-struct variable elsewhere has a member of the same name; `%s` of a string reached through two handle levels prints the text.
//!
//! Expected lines come from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const STRUCTVAR: &str = r#"
// top: t8_5c   (any unpacked-struct variable anywhere in the design - even in another module - breaks reads of g.h.<member>: strings become spaces, ints x. Packed struct or struct inside a class: fine)
typedef struct { int k; int h; } rec_t;
class Item; string tag = "abc"; int n = 7; function string show(); return "show"; endfunction endclass
class Hold; Item h; function new(); h = new; endfunction function Item get(); return h; endfunction endclass
module t8_5c;
  Hold g; Item i; rec_t r;   // removing `rec_t r;` makes the strings correct
  string s;
  initial begin
    g = new; i = new;
    $display("T|a|[%s] [%s] [%s] [%s] [%s]", g.get().show(), g.h.tag, g.h.show(), i.tag, i.show());
    s = g.h.tag; $display("T|b|assigned [%s] len=%0d eq=%0d n=%0d", s, s.len(), g.h.tag == "abc", g.h.n);
  end
endmodule
"#;

const STRUCTVAR_FAMILY: &str = r#"
typedef struct { int k; int h; } rec_t;
class Item; string tag = "abc"; int n = 7; function string show(); return "show"; endfunction endclass
class Hold; Item h; function new(); h = new; endfunction function Item get(); return h; endfunction endclass
class W; Hold c; W other; function new(); c = new; endfunction endclass
module t;
  Hold g; Item i; rec_t r;
  W w2;
  string s;
  initial begin
    g = new; i = new; w2 = new;
    $display("T|a|[%s] [%s] [%s] [%s] [%s]", g.get().show(), g.h.tag, g.h.show(), i.tag, i.show());
    s = g.h.tag; $display("T|b|assigned [%s] len=%0d eq=%0d n=%0d", s, s.len(), g.h.tag == "abc", g.h.n);
    g.h.n = 9; g.h.tag = "xy";
    $display("T|c|n=%0d tag=%s w=%0d wtag=%s null=%0d", g.h.n, g.h.tag, w2.c.h.n, w2.c.h.tag, w2.other == null);
    r.k = 1;
  end
endmodule
"#;

const CTOR_WRITE: &str = r#"
typedef struct { int k; int h; } rec_t;
class Item; int n = 7; endclass
class Hold; Item h; function new(); h = new; endfunction function Item get(); return h; endfunction endclass
class Hold2; Item h; function new(); this.h = new; endfunction endclass
module t; Hold g; Hold2 g2; rec_t r; Item i;
  initial begin g = new; g2 = new; i = g.get(); $display("T|a|null=%0d null2=%0d %0d", g.h == null, g2.h == null, i == null); end
endmodule
"#;

const TWO_LEVEL: &str = r#"
// top: t17
class Leaf; string tag = "s"; int k = 4; endclass
class Hold; Leaf h; function new(); h = new; endfunction endclass
module t;
  Hold n; string x;
  initial begin
    n = new;
    $display("T|a|two-level [%s] k=%0d", n.h.tag, n.h.k);
    x = {n.h.tag, "!"}; $display("T|b|concat [%s] eq=%0d", x, n.h.tag == "s");
  end
endmodule
"#;

#[test]
fn audit_repro() {
    let want = [
        "T|a|[show] [abc] [show] [abc] [show]",
        "T|b|assigned [abc] len=3 eq=1 n=7",
    ];
    assert_eq!(t_lines(STRUCTVAR), want);
}

#[test]
fn writes_and_nested_reads() {
    let want = [
        "T|a|[show] [abc] [show] [abc] [show]",
        "T|b|assigned [abc] len=3 eq=1 n=7",
        "T|c|n=9 tag=xy w=7 wtag=abc null=1",
    ];
    assert_eq!(t_lines(STRUCTVAR_FAMILY), want);
}

#[test]
fn constructor_bare_write() {
    let want = ["T|a|null=0 null2=0 0"];
    assert_eq!(t_lines(CTOR_WRITE), want);
}

#[test]
fn two_level_string_display() {
    let want = ["T|a|two-level [s] k=4", "T|b|concat [s!] eq=1"];
    assert_eq!(t_lines(TWO_LEVEL), want);
}
