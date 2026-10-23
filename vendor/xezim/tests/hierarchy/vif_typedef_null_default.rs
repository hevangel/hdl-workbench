//! Issue #285: a variable whose type is a typedef alias of a virtual interface
//! (`typedef virtual bus_if vif_t; vif_t v;`) was not default-initialized at
//! module, `$unit`, subroutine or block scope, so `v == null` was x. IEEE
//! 1800-2023 Table 6-7 and §25.9 make the default `null`, as for the
//! spelled-out `virtual bus_if v;`. `$typename(v)` also reported `logic`
//! instead of the interface type (§20.6.1). Every expected value below comes
//! from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|") || l.starts_with("PASS") || l.starts_with("FAIL"))
        .collect()
}

/// The issue's reproducer.
#[test]
fn issue_285_reproducer() {
    let got = t_lines(
        r#"
interface bus_if(input bit clk);
    logic en;
endinterface

module top;
    typedef virtual bus_if vif_t;

    bit clk = 0;
    bus_if i0(clk);

    vif_t          by_typedef;
    virtual bus_if spelled_out;

    initial begin
        if (by_typedef == null) $display("PASS: typedef'd  is null");
        else                    $display("FAIL: typedef'd  is not null (got %0d)", by_typedef == null);

        if (spelled_out == null) $display("PASS: spelled out is null");
        else                     $display("FAIL: spelled out is not null");

        $display("T|$typename(by_typedef) = %s", $typename(by_typedef));

        by_typedef = i0;
        if (by_typedef != null) $display("PASS: non-null after assignment");
        else                    $display("FAIL: still null after assignment");
    end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "PASS: typedef'd  is null",
            "PASS: spelled out is null",
            "T|$typename(by_typedef) = virtual bus_if",
            "PASS: non-null after assignment",
        ],
    );
}

/// Every scope and spelling: a package typedef (imported and `pk::`
/// qualified), a `$unit` typedef, a typedef of a typedef, a modport, a
/// parameterized interface and the `virtual interface` form; arrays of the
/// typedef; task, static task, function, block and named-block locals; and
/// class properties.
#[test]
fn typedef_vif_defaults_to_null_in_every_scope() {
    let got = t_lines(
        r#"
interface bus_if(input bit clk);
  logic en;
  modport mp(input en);
endinterface
interface pbus_if #(parameter W = 8) (input bit clk);
  logic [W-1:0] d;
endinterface
package pk;
  typedef virtual bus_if pvif_t;
endpackage
typedef virtual bus_if uvif_t;
module top;
  import pk::*;
  typedef virtual bus_if vif_t;
  typedef vif_t vif2_t;
  typedef virtual bus_if.mp mvif_t;
  typedef virtual pbus_if #(16) pvif16_t;
  typedef virtual interface bus_if ivif_t;
  bit clk = 0;
  bus_if i0(clk);
  pbus_if #(16) p0(clk);
  vif_t a;
  pvif_t b;
  uvif_t c;
  vif2_t d;
  mvif_t e;
  pvif16_t f;
  ivif_t g;
  pk::pvif_t h;
  vif_t arr[2];
  vif_t arr2[2][3];
  vif_t arr3[1:0];
  vif_t darr[];
  virtual bus_if so;
  class C;
    vif_t cp;
    vif2_t cp2;
    mvif_t cp3;
    vif_t carr[2];
  endclass
  task automatic tk();
    vif_t t;
    mvif_t tm;
    pvif16_t tp;
    $display("T|task t=%0d tm=%0d tp=%0d", t == null, tm == null, tp == null);
  endtask
  function automatic bit fn();
    vif2_t fv;
    return fv == null;
  endfunction
  task static tks();
    vif_t ts;
    $display("T|static task ts=%0d", ts == null);
  endtask
  initial begin
    vif_t l;
    uvif_t lu;
    vif_t la[2];
    C o;
    $display("T|mod a=%0d b=%0d c=%0d d=%0d e=%0d f=%0d g=%0d h=%0d so=%0d", a == null, b == null, c == null, d == null, e == null, f == null, g == null, h == null, so == null);
    $display("T|arr0=%0d arr1=%0d arr2=%0d arr3=%0d la=%0d", arr[0] == null, arr[1] == null, arr2[1][2] == null, arr3[0] == null, la[1] == null);
    darr = new[2];
    $display("T|darr1=%0d", darr[1] == null);
    $display("T|blk l=%0d lu=%0d", l == null, lu == null);
    begin : nb
      pvif_t nbv;
      $display("T|named nbv=%0d", nbv == null);
    end
    tk();
    tks();
    $display("T|fn=%0d", fn());
    o = new;
    $display("T|cls cp=%0d cp2=%0d cp3=%0d carr=%0d", o.cp == null, o.cp2 == null, o.cp3 == null, o.carr[1] == null);
    a = i0; e = i0; f = p0;
    $display("T|after a=%0d e=%0d f=%0d", a != null, e != null, f != null);
    a = null;
    $display("T|reset a=%0d", a == null);
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|mod a=1 b=1 c=1 d=1 e=1 f=1 g=1 h=1 so=1",
            "T|arr0=1 arr1=1 arr2=1 arr3=1 la=1",
            "T|darr1=1",
            "T|blk l=1 lu=1",
            "T|named nbv=1",
            "T|task t=1 tm=1 tp=1",
            "T|static task ts=1",
            "T|fn=1",
            "T|cls cp=1 cp2=1 cp3=1 carr=1",
            "T|after a=1 e=1 f=1",
            "T|reset a=1",
        ],
    );
}

/// §20.6.1: `$typename` of a virtual-interface variable or property names
/// the interface as declared — the modport, then the parameter values
/// (positional ones evaluated, a named one kept by name).
#[test]
fn typename_of_virtual_interface() {
    let got = t_lines(
        r#"
interface bus_if(input bit clk);
  logic en;
  modport mp(input en);
endinterface
interface pbus_if #(parameter W = 8, parameter int D = 2) (input bit clk);
  logic [W-1:0] d;
  modport m(input d);
endinterface
package pk;
  typedef virtual bus_if pvif_t;
endpackage
typedef virtual bus_if uvif_t;
module top;
  import pk::*;
  localparam P = 4;
  typedef virtual bus_if vif_t;
  typedef vif_t vif2_t;
  typedef virtual bus_if.mp mvif_t;
  typedef virtual pbus_if vd_t;
  typedef virtual pbus_if #(.W(16)) vn_t;
  typedef virtual pbus_if #(P*2, 3) ve_t;
  typedef virtual pbus_if #(16).m vm_t;
  vif_t a; pvif_t b; uvif_t c; vif2_t d; mvif_t e;
  vd_t f; vn_t g; ve_t h; vm_t k;
  virtual bus_if so;
  class C;
    vif_t cp;
  endclass
  initial begin
    vif_t l;
    C o = new;
    $display("T|%s|%s|%s|%s|%s", $typename(a), $typename(b), $typename(c), $typename(d), $typename(e));
    $display("T|%s|%s|%s|%s", $typename(f), $typename(g), $typename(h), $typename(k));
    $display("T|%s|%s|%s|%s", $typename(so), $typename(l), $typename(o.cp), $typename(vif_t));
  end
endmodule
"#,
    );
    assert_eq!(
        got,
        [
            "T|virtual bus_if|virtual bus_if|virtual bus_if|virtual bus_if|virtual bus_if.mp",
            "T|virtual pbus_if|virtual pbus_if #(.W(16))|virtual pbus_if #(8, 3)|virtual pbus_if.m #(16)",
            "T|virtual bus_if|virtual bus_if|virtual bus_if|virtual bus_if",
        ],
    );
}
