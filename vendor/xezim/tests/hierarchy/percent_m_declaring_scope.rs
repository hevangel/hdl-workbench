//! §21.2.1.7 `%m` names the scope that DECLARES the running code: a package
//! subroutine is `pk.f`, a class method `pk.C.show` (or `tb.u.E.show` for a
//! class declared in a module instance), a subroutine reached through a
//! hierarchical call `tb.s1.mf`. Generate blocks of the root module, the
//! label of a named `always`/`initial` block and a labelled assertion are
//! scopes too, and a severity task's "Scope:" line names the same scope.
//! Every expectation was cross-checked against the reference simulator.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn assert_has(out: &[String], want: &[&str]) {
    for w in want {
        assert!(out.iter().any(|l| l == w), "missing {:?}; got {:?}", w, out);
    }
}

#[test]
fn m_names_package_class_and_instance_declaring_scopes() {
    let o = out(r#"
package pk;
  function automatic void f(); $display("pkf=%m"); endfunction
  task automatic t(); $display("pkt=%m"); endtask
  class C;
    function void show();
      $display("show=%m");
      begin : nb
        $display("nb=%m");
      end
    endfunction
    static function void sshow(); $display("sshow=%m"); endfunction
    task t2(); #1 $display("t2=%m"); endtask
  endclass
  function automatic void pcall(C c); c.show(); endfunction
endpackage
module sub;
  class E;
    function void show(); $display("E=%m"); endfunction
  endclass
  E e;
  function void mf(); $display("mf=%m"); endfunction
  task automatic mt(); begin : tb1 $display("mt=%m"); end endtask
  initial e = new();
endmodule
module tb;
  sub s1();
  sub s2();
  initial begin
    static pk::C c = new();
    #1;
    begin : blk
      pk::f();
      pk::t();
      c.show();
      pk::C::sshow();
      pk::pcall(c);
      s1.e.show();
      s1.mf();
      s2.mt();
    end
    c.t2();
  end
endmodule
"#);
    assert_eq!(
        o,
        [
            "pkf=pk.f",
            "pkt=pk.t",
            "show=pk.C.show",
            "nb=pk.C.show.nb",
            "sshow=pk.C.sshow",
            "show=pk.C.show",
            "nb=pk.C.show.nb",
            "E=tb.s1.E.show",
            "mf=tb.s1.mf",
            "mt=tb.s2.mt.tb1",
            "t2=pk.C.t2",
        ]
    );
}

#[test]
fn m_names_root_generate_blocks_and_block_labels() {
    let o = out(r#"
module u;
  logic clk = 0;
  if (1) begin : g
    if (1) begin : h
      initial $display("u.gh=%m");
    end
  end
  always @(clk) begin : blk
    $display("u.blk=%m");
  end
  initial #1 clk = 1;
endmodule
module tb;
  u u();
  localparam P = 1;
  logic clk = 0;
  if (P) begin : g
    initial $display("g=%m");
    if (1) begin : h
      initial $display("gh=%m");
    end
  end
  if (P) initial $display("genblk=%m");
  case (P) 1: begin : c1 initial $display("case=%m"); end endcase
  for (genvar i = 0; i < 2; i++) begin : fl
    initial $display("fl=%m");
    if (i == 1) begin : in
      initial $display("flin=%m");
    end
  end
  always @(clk) begin : ab
    $display("ab=%m");
  end
  initial begin : ib
    $display("ib=%m");
    begin : inner
      $display("inner=%m");
    end
    #2 clk = 1;
  end
endmodule
"#);
    assert_has(
        &o,
        &[
            "u.gh=tb.u.g.h",
            "g=tb.g",
            "gh=tb.g.h",
            "genblk=tb.genblk2",
            "case=tb.c1",
            "fl=tb.fl[0]",
            "fl=tb.fl[1]",
            "flin=tb.fl[1].in",
            "ib=tb.ib",
            "inner=tb.ib.inner",
            "u.blk=tb.u.blk",
            "ab=tb.ab",
        ],
    );
    assert_eq!(o.len(), 12, "got {:?}", o);
}

#[test]
fn labelled_assertions_are_scopes() {
    let o = out(r#"
module tb;
  logic clk = 0, a = 0, b = 0;
  always #5 clk = ~clk;
  ap: assert property (@(posedge clk) a |-> b) else $error("ap=%m");
  cp: cover property (@(posedge clk) a) $info("cp=%m");
  initial begin
    #12 a = 1;
    #10 a = 0;
    #5 $finish;
  end
  always @(posedge clk) begin
    ai: assert (clk) $info("ai=%m");
  end
endmodule
"#);
    assert_has(
        &o,
        &[
            "** Error: ap=tb.ap",
            "** Info: cp=tb.cp",
            "** Info: ai=tb.ai",
        ],
    );
    let scopes: Vec<&str> = o.iter().filter_map(|l| l.split("Scope: ").nth(1)).collect();
    assert_eq!(
        scopes,
        ["tb.ai", "tb.ai", "tb.ap", "tb.cp", "tb.ai"],
        "got {:?}",
        o
    );
}

#[test]
fn severity_scope_matches_m() {
    let o = out(r#"
package pk;
  class C;
    function void f(); $warning("in C.f %m"); endfunction
  endclass
  function automatic void pf(); $warning("in pf %m"); endfunction
endpackage
module u;
  task automatic t(); $warning("in u.t %m"); endtask
  initial begin : ub
    #1 $warning("in ub %m");
  end
endmodule
module tb;
  u u1();
  initial begin : blk
    pk::C c;
    c = new();
    begin : inner
      $warning("inner %m");
    end
    c.f();
    pk::pf();
    #2 u1.t();
  end
endmodule
"#);
    let scopes: Vec<&str> = o.iter().filter_map(|l| l.split("Scope: ").nth(1)).collect();
    assert_eq!(
        scopes,
        ["tb.blk.inner", "pk.C.f", "pk.pf", "tb.u1.ub", "tb.u1.t"],
        "got {:?}",
        o
    );
}
