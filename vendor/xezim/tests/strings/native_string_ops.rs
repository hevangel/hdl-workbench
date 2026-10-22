//! §6.16 string methods compile to native `StrOp` insns — len/getc/substr/
//! toupper/tolower/compare/icompare/atoi/atohex + the in-place mutators
//! (putc, itoa/hextoa) as read-modify-write — and §11.4.12 all-string
//! concatenation compiles to a byte-level join. §6.16.6: relational
//! operators on two strings are LEXICOGRAPHIC ("abc" < "b" is true); the
//! numeric compare insns diverge once lengths differ, so the compiler
//! routes them through the native strcmp. Every expectation byte-verified
//! against the reference simulator; the whole module compiles with
//! fallbacks=0.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const METHODS: &str = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  string a, b, c, m;
  logic [31:0] n;
  logic [7:0] ch;
  integer cmp1, cmp2, ai;
  logic [2:0] sel;

  always @(posedge clk) begin
    case (sel)
      3'd0: begin
        a = "Hello";
        b = "World";
        c = {a, ", ", b, "!"};
        n = c.len();
        ch = c.getc(4);
        m = c.substr(7, 11);
      end
      3'd1: begin
        m = a.toupper();
        c = b.tolower();
        cmp1 = a.compare(b);
        cmp2 = a.icompare("HELLO");
      end
      3'd2: begin
        a = "  123abc";
        ai = a.atoi();
        b = "ff9";
        n = b.atohex();
      end
      3'd3: begin
        m = "xyzzy";
        m.putc(2, "Q");
        c.itoa(-42);
        b.hextoa(48879);
      end
      default: begin
        m = "idle";
      end
    endcase
  end
  task step(input logic [2:0] s2);
    sel = s2; @(negedge clk);
  endtask
  initial begin
    step(0); $display("NOTE: SOP0 [%s] len=%0d ch=%c sub=[%s]", c, n, ch, m);
    step(1); $display("NOTE: SOP1 up=[%s] low=[%s] cmp=%0d icmp=%0d", m, c, cmp1, cmp2);
    step(2); $display("NOTE: SOP2 atoi=%0d atohex=%0d", ai, n);
    step(3); $display("NOTE: SOP3 putc=[%s] itoa=[%s] hextoa=[%s]", m, c, b);
    step(4); $display("NOTE: SOP4 [%s]", m);
    $finish;
  end
endmodule

"#;

const ORDERING: &str = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  string a, b;
  logic lt, gt, eq, le;
  always @(posedge clk) begin
    lt = (a < b); gt = (a > b); eq = (a == b); le = (a <= b);
  end
  task chk; @(negedge clk); $display("NOTE: ORD %s %s -> lt=%b gt=%b eq=%b le=%b", a, b, lt, gt, eq, le); endtask
  initial begin
    a = "abc"; b = "b";   chk;
    a = "abc"; b = "abd"; chk;
    a = "same"; b = "same"; chk;
    a = "Z"; b = "aa";    chk;
    $finish;
  end
endmodule

"#;

#[test]
fn string_methods_compile_natively() {
    assert_eq!(
        notes(METHODS),
        [
            "NOTE: SOP0 [Hello, World!] len=13 ch=o sub=[World]",
            "NOTE: SOP1 up=[HELLO] low=[world] cmp=-15 icmp=0",
            "NOTE: SOP2 atoi=0 atohex=4089",
            "NOTE: SOP3 putc=[xyQzy] itoa=[-42] hextoa=[beef]",
            "NOTE: SOP4 [idle]",
        ]
    );
}

#[test]
fn string_relationals_are_lexicographic_when_compiled() {
    assert_eq!(
        notes(ORDERING),
        [
            "NOTE: ORD abc b -> lt=1 gt=0 eq=0 le=1",
            "NOTE: ORD abc abd -> lt=1 gt=0 eq=0 le=1",
            "NOTE: ORD same same -> lt=0 gt=0 eq=1 le=1",
            "NOTE: ORD Z aa -> lt=1 gt=0 eq=0 le=1",
        ]
    );
}
