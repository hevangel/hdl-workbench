//! §26.3 / §26.6 — value names (subroutines, parameters, `let`s) and
//! covergroups declared by more than one package, the neighbours of the
//! type-name collisions of #300.
//!
//! Package functions, tasks, parameters and `let`s share flat tables keyed
//! by the bare name, where the package registered last wins. A package that
//! imported the other one's name (`import pb::*; ... f()`) and a name
//! re-exported by a third package (`export pb::*;`, `import pa::P; export
//! pa::P;`) read the wrong declaration. Bare references are now spelled
//! `pkg::name` by the §26.3 rules; a package covergroup declared twice is
//! renamed like a type. Also: a wildcard import makes a package `let`
//! visible, and the statics of a class whose name another package also
//! declares start at their initializers.
//!
//! Expected values come from the reference simulator.

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
fn imported_and_reexported_subroutines_and_parameters() {
    let src = r#"
package pa;
  typedef logic [3:0] n_t;
  parameter int P = 3;
  function automatic int f(); return 1; endfunction
endpackage
package pb;
  typedef logic [7:0] n_t;
  parameter int P = 5;
  function automatic int f(); return 2; endfunction
endpackage
package pc;
  import pb::*;
  export pb::*;
  n_t keep;
  int u = P;
  function automatic int g(); return f() * 10; endfunction
endpackage
package pd;
  import pa::n_t; import pa::P; import pa::f;
  export pa::n_t; export pa::P; export pa::f;
endpackage
module top;
  import pc::*;
  n_t x;
  pd::n_t y;
  initial begin
    $display("T|bits %0d %0d", $bits(x), $bits(y));
    $display("T|p %0d %0d %0d", P, pd::P, pc::u);
    $display("T|f %0d %0d %0d", f(), pd::f(), pc::g());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|bits 8 4", "T|p 5 3 5", "T|f 2 1 20"],
        "T| lines differ"
    );
}

#[test]
fn package_body_reads_its_import_not_the_last_package() {
    let src = r#"
package pa;
  parameter int P = 3;
  function automatic int f(); return 1; endfunction
endpackage
package pb;
  parameter int P = 5;
  function automatic int f(); return 2; endfunction
endpackage
package pc;
  import pb::*;
  function automatic int g(); return f() * 10; endfunction
  int u = P;
endpackage
module top;
  initial $display("T|g %0d %0d", pc::g(), pc::u);
endmodule
"#;
    assert_eq!(t_lines(src), ["T|g 20 5"]);
}

#[test]
fn package_lets_by_wildcard_and_qualified() {
    let src = r#"
package pa;
  let mx(a, b) = (a > b) ? a : b;
  let sc(a) = a * 2;
endpackage
package pb;
  let mx(a, b) = (a < b) ? a : b;
  let sc(a) = a * 3;
endpackage
package pu;
  let tw(a) = a + 40;
endpackage
module top;
  import pb::*;
  import pu::*;
  initial begin
    $display("T|q %0d %0d", pa::mx(3, 5), pb::mx(3, 5));
    $display("T|s %0d %0d", pa::sc(7), pb::sc(7));
    $display("T|wild %0d %0d", sc(7), tw(2));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|q 5 3", "T|s 14 21", "T|wild 21 42"]);
}

#[test]
fn same_named_package_covergroups() {
    let src = r#"
package pa;
  class C;
    rand bit [3:0] v;
    covergroup cg; coverpoint v { bins lo = {[0:7]}; } endgroup
    function new(); cg = new; endfunction
  endclass
  covergroup pcg with function sample(int x); coverpoint x { bins a = {1}; bins b = {2}; } endgroup
endpackage
package pb;
  class C;
    rand bit [3:0] v;
    covergroup cg; coverpoint v { bins lo = {[0:3]}; bins hi = {[12:15]}; } endgroup
    function new(); cg = new; endfunction
  endclass
  covergroup pcg with function sample(int x); coverpoint x { bins a = {1}; bins b = {2}; bins c = {3}; bins d = {4}; } endgroup
endpackage
module top;
  pa::C ca = new; pb::C cb = new;
  pa::pcg ga = new; pb::pcg gb = new;
  initial begin
    ca.v = 2; ca.cg.sample(); cb.v = 2; cb.cg.sample();
    ga.sample(1); gb.sample(1);
    $display("T|cls %0.2f %0.2f", ca.cg.get_inst_coverage(), cb.cg.get_inst_coverage());
    $display("T|pkg %0.2f %0.2f", ga.get_inst_coverage(), gb.get_inst_coverage());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|cls 100.00 50.00", "T|pkg 50.00 25.00"]);
}

/// Out-of-block class methods (`function C::f`) are class scope, not
/// package values: their names and the class members their bodies read are
/// left alone.
#[test]
fn out_of_block_methods_beside_colliding_values() {
    let src = r#"package pa;
  parameter int base = 100;
  function automatic int g(); return 1; endfunction
  class C;
    int base = 5;
    extern function new();
    extern function int f();
    extern function int g();
  endclass
  function C::new(); base = 6; endfunction
  function int C::f(); return base + g(); endfunction
  function int C::g(); return 10; endfunction
endpackage
package pb;
  parameter int base = 200;
  function automatic int g(); return 2; endfunction
  class D;
    int k = 1;
    extern function new();
    extern function int f();
  endclass
  function D::new(); k = 3; endfunction
  function int D::f(); return base + g() + k; endfunction
endpackage
module top;
  import pb::*;
  initial begin
    pa::C c = new; pb::D d = new;
    $display("T|ob %0d %0d %0d %0d", c.f(), d.f(), g(), base);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|ob 16 205 2 200"]);
}

#[test]
fn statics_of_same_named_classes() {
    let src = r#"
package pa;
  class C;
    static int cnt = 1;
    static function int sf(); return cnt + 100; endfunction
    function new(); cnt++; endfunction
  endclass
  class P #(int W = 2);
    static int s = W;
    static function int get(); return s * 10; endfunction
  endclass
endpackage
package pb;
  class C;
    static int cnt = 50;
    static function int sf(); return cnt + 200; endfunction
    function new(); cnt += 5; endfunction
  endclass
  class P #(int W = 3);
    static int s = W + 1;
    static function int get(); return s * 100; endfunction
  endclass
endpackage
module top;
  initial begin
    pa::C a; pb::C b;
    $display("T|init %0d %0d", pa::C::cnt, pb::C::cnt);
    a = new; b = new; b = new;
    $display("T|cnt %0d %0d", pa::C::cnt, pb::C::cnt);
    $display("T|sf %0d %0d", pa::C::sf(), pb::C::sf());
    $display("T|ps %0d %0d %0d %0d", pa::P#()::s, pb::P#()::s, pa::P#(5)::get(), pb::P#(5)::get());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|init 1 50",
            "T|cnt 2 60",
            "T|sf 102 260",
            "T|ps 2 4 50 600"
        ]
    );
}

#[test]
fn static_through_qualified_specialization() {
    let src = r#"
package pa;
  class P #(int W = 2);
    static int s = W;
    static function int get(); return s * 10; endfunction
  endclass
endpackage
class Q #(int W = 3);
  static int s = W;
endclass
module top;
  initial begin
    $display("T|a %0d %0d %0d", pa::P#()::s, pa::P#(2)::s, pa::P#(7)::s);
    $display("T|b %0d %0d %0d", Q#()::s, Q#(3)::s, Q#(8)::s);
    $display("T|c %0d %0d", pa::P#()::get(), Q#()::s);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|a 2 2 7", "T|b 3 3 8", "T|c 20 3"]);
}

#[test]
fn qualified_parameter_inside_an_instance() {
    let src = r#"
package pa; parameter int W = 4; endpackage
package pb; parameter int W = 9; endpackage
module sub #(parameter int P = 1) (output int o, output int o2, output int o3);
  assign o = 1 + pa::W;
  assign o2 = P + pa::W;
  assign o3 = pa::W;
endmodule
module wsub #(parameter int P = 1) (output int o);
  import pa::*;
  assign o = P + W;
  initial #1 $display("T|in %0d %0d %0d", P, W, o);
endmodule
module top;
  import pb::*;
  int so, so2, so3, wo;
  sub #(.P(3)) u(.o(so), .o2(so2), .o3(so3));
  wsub #(.P(W)) w(.o(wo));
  initial #2 $display("T|sub %0d %0d %0d", so, so2, so3);
endmodule
"#;
    assert_eq!(t_lines(src), ["T|in 9 4 13", "T|sub 5 7 4"]);
}
