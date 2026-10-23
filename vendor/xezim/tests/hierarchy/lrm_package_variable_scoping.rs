//! IEEE 1800-2023 §26.3: same-named variables of different packages, or of a package and an importing module, are distinct variables; references resolve through explicit qualification, the package's own subroutines and classes, and each design unit's imports.
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

const PKG_ALIAS: &str = r#"
// top: rpv
package pa; int cnt = 1; function int inc(); cnt++; return cnt; endfunction endpackage
package pc; int cnt = 50; endpackage
module rpv;
  initial begin
    #1;
    $display("T|r1|pa=%0d pc=%0d", pa::cnt, pc::cnt);
    void'(pa::inc());
    $display("T|r2|pa=%0d pc=%0d", pa::cnt, pc::cnt);
    pc::cnt = 7;
    $display("T|r3|pa=%0d pc=%0d", pa::cnt, pc::cnt);
  end
endmodule
"#;

const PKG_SHADOW: &str = r#"
// top: rpl
package base_p; int shared_v = 1; endpackage
module rpl;
  import base_p::*;
  int shared_v = 99;
  initial #1 $display("T|r1|local=%0d pkg=%0d", shared_v, base_p::shared_v);
endmodule
"#;

const PKG_FAMILY: &str = r#"
package pa; int cnt = 1; parameter int P = 10; function int inc(); cnt++; return cnt; endfunction
  function int who(); return 1; endfunction task automatic tk(output int o); o = 100; endtask
  int only_a = 3;
endpackage
package pc; int cnt = 50; parameter int P = 20; function int who(); return 2; endfunction
  task automatic tk(output int o); o = 200; endtask
  function int getv(); return cnt; endfunction
endpackage
module m1; import pa::*; initial #2 $display("T|m1|cnt=%0d P=%0d who=%0d only=%0d", cnt, P, who(), only_a); endmodule
module m2; import pc::cnt; import pc::who; initial #2 $display("T|m2|cnt=%0d who=%0d", cnt, who()); endmodule
module m3; import pa::*; int cnt = 77; initial #2 begin $display("T|m3|cnt=%0d pa=%0d", cnt, pa::cnt); cnt = 78; $display("T|m3b|cnt=%0d pa=%0d", cnt, pa::cnt); end endmodule
module t;
  int o1, o2;
  m1 u1(); m2 u2(); m3 u3();
  initial begin
    #1;
    $display("T|r1|pa=%0d pc=%0d P=%0d/%0d who=%0d/%0d", pa::cnt, pc::cnt, pa::P, pc::P, pa::who(), pc::who());
    void'(pa::inc());
    pa::tk(o1); pc::tk(o2);
    $display("T|r2|pa=%0d pc=%0d getv=%0d tk=%0d/%0d", pa::cnt, pc::cnt, pc::getv(), o1, o2);
    pc::cnt = 7;
    $display("T|r3|pa=%0d pc=%0d getv=%0d", pa::cnt, pc::cnt, pc::getv());
    #5 $display("T|r4|pa=%0d pc=%0d", pa::cnt, pc::cnt);
  end
endmodule
"#;

const PKG_SHADOW2: &str = r#"
package base_p; int shared_v = 1; endpackage
module t;
  import base_p::*;
  int shared_v = 99;
  initial begin #1 $display("T|r1|local=%0d pkg=%0d", shared_v, base_p::shared_v);
    base_p::shared_v = 5; $display("T|r2|local=%0d pkg=%0d", shared_v, base_p::shared_v);
    shared_v = 6; $display("T|r3|local=%0d pkg=%0d", shared_v, base_p::shared_v); end
endmodule
"#;

const PKG_CLASS: &str = r#"
package pa; int cnt = 1; string nm = "A"; int calc = f0();
  function int f0(); return 5; endfunction
  class C; function int get(); return cnt; endfunction function void bump(); cnt += 10; endfunction endclass
endpackage
package pc; int cnt = 50; string nm = "C"; int calc = 9;
  task automatic upd(); cnt = cnt + 1; nm = "CC"; endtask
endpackage
module m4; import pc::*; initial #3 begin upd(); $display("T|m4|cnt=%0d nm=%s calc=%0d", cnt, nm, calc); end endmodule
module t;
  pa::C c; m4 u4();
  initial begin
    #1 c = new; c.bump();
    $display("T|a|get=%0d pa=%0d pc=%0d nm=%s/%s calc=%0d/%0d", c.get(), pa::cnt, pc::cnt, pa::nm, pc::nm, pa::calc, pc::calc);
    #5 $display("T|b|pa=%0d pc=%0d nm=%s/%s", pa::cnt, pc::cnt, pa::nm, pc::nm);
  end
endmodule
"#;

#[test]
fn audit_repro_two_packages() {
    let want = ["T|r1|pa=1 pc=50", "T|r2|pa=2 pc=50", "T|r3|pa=2 pc=7"];
    assert_eq!(t_lines(PKG_ALIAS), want);
}

#[test]
fn audit_repro_local_shadow() {
    let want = ["T|r1|local=99 pkg=1"];
    assert_eq!(t_lines(PKG_SHADOW), want);
}

#[test]
fn imports_subroutines_and_params() {
    let want = [
        "T|r1|pa=1 pc=50 P=10/20 who=1/2",
        "T|r2|pa=2 pc=50 getv=50 tk=100/200",
        "T|r3|pa=2 pc=7 getv=7",
        "T|m1|cnt=2 P=10 who=1 only=3",
        "T|m2|cnt=7 who=2",
        "T|m3|cnt=77 pa=2",
        "T|m3b|cnt=78 pa=2",
        "T|r4|pa=2 pc=7",
    ];
    assert_eq!(t_lines(PKG_FAMILY), want);
}

#[test]
fn local_shadow_writes() {
    let want = [
        "T|r1|local=99 pkg=1",
        "T|r2|local=99 pkg=5",
        "T|r3|local=6 pkg=5",
    ];
    assert_eq!(t_lines(PKG_SHADOW2), want);
}

#[test]
fn package_class_and_task_scopes() {
    let want = [
        "T|a|get=11 pa=11 pc=50 nm=A/C calc=5/9",
        "T|m4|cnt=51 nm=CC calc=9",
        "T|b|pa=11 pc=51 nm=A/CC",
    ];
    assert_eq!(t_lines(PKG_CLASS), want);
}
