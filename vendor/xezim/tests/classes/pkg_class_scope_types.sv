`timescale 1ns/1ps

// ----- self-check macros (tests/classes SVTEST_* style, inlined) -----
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH

`define SVTEST_INIT \
int failures = 0;

`define SVTEST_CHECK(expr, msg) \
if (!(expr)) begin \
  failures++; \
  $display("FAIL @%0t : %s", $time, msg); \
end

`define SVTEST_PASSFAIL \
if (failures == 0) begin \
  $display("TEST_PASS"); \
end else begin \
  $display("TEST_FAIL count=%0d", failures); \
  $fatal(1); \
end

`endif

// ============================================================================
// IEEE 1800-2017 chained class-scope types: a class scope used as the
// prefix of a TYPE (§8.23 `class_type :: { class_type :: } identifier`,
// §8.25.1 parameterized class as scope prefix `C#(N)::t`, §6.20.3
// `typedef C::T c_t;`, §13.5 ports). Every check below is a TYPE
// position; the expression-position shapes (statics, enum labels,
// methods) are guarded at the end. All expected values are
// cross-checked against the reference simulator.
// ============================================================================

// [MODEL] package carrying class-scope types at three depths.
package layout_pkg;

  virtual class LayoutRoot;

    parameter int BUS_W = 8;

    typedef struct packed {
      bit [1:0] mode_code;
      bit [1:0] size_code;
      bit [3:0] kind_code;
    } cfg_t;

    typedef enum logic [1:0] { OP_A, OP_B } op_e;

    static logic [7:0] s_shadow = 8'hA5;

    static function logic [7:0] get_shadow();
      return s_shadow;
    endfunction

    class Sub;
      typedef logic [3:0] sub_t;
    endclass

  endclass

  class VectorShape #(parameter int W = 4);
    typedef logic [W-1:0] vec_t;
  endclass

endpackage : layout_pkg

// [ORIG] consumer package: a class-method ANSI port typed by a chained
// class scope plus a task-body declaration of the same shape.
package inspection_pkg;
  import layout_pkg::*;

  class LayoutInspector;

    function int get_size_idx(
      logic [39:0] addr,
      layout_pkg::LayoutRoot::cfg_t layout_word
    );
      return 6 + layout_word.size_code;
    endfunction

    task show_cfg;
      layout_pkg::LayoutRoot::cfg_t layout_word;
      layout_word.kind_code = 1;
      $display("ORIG site2 %b", layout_word.kind_code);
    endtask

  endclass
endpackage

// [C02] submodule with a chained class-scope ANSI port type.
module c02_sub(input layout_pkg::LayoutRoot::cfg_t p, output int o);
  assign o = p.size_code;
endmodule

module top;
  import layout_pkg::*;
  import inspection_pkg::*;

  `SVTEST_INIT

  // [C01] module-level variable
  layout_pkg::LayoutRoot::cfg_t v01;
  initial begin
    `SVTEST_CHECK($bits(v01) == 8, "C01 module var $bits")
  end

  // [C02] module ANSI port type (via the submodule above)
  int o02;
  c02_sub u_c02('0, o02);
  initial begin
    `SVTEST_CHECK(o02 == 0, "C02 ANSI port type")
  end

  // [C03] function ANSI port type
  function int count_ones(layout_pkg::LayoutRoot::cfg_t c);
    return $countones(c);
  endfunction
  initial begin
    `SVTEST_CHECK(count_ones('0) == 0, "C03 function port type")
  end

  // [C04] task-body data declaration
  task t04;
    layout_pkg::LayoutRoot::cfg_t c04;
    begin
      c04 = '0;
      `SVTEST_CHECK($bits(c04) == 8, "C04 task-body decl $bits")
    end
  endtask
  initial t04;

  // [C05] function return type
  function layout_pkg::LayoutRoot::cfg_t mk_cfg();
    return '0;
  endfunction
  initial begin
    `SVTEST_CHECK($bits(mk_cfg()) == 8, "C05 function return type")
  end

  // [C06] class property type
  class Holder;
    layout_pkg::LayoutRoot::cfg_t slot;
  endclass
  Holder h06;
  initial begin
    h06 = new();
    `SVTEST_CHECK($bits(h06.slot) == 8, "C06 class property $bits")
  end

  // [C07] typedef aliasing a class-scoped type (§6.20.3)
  typedef layout_pkg::LayoutRoot::cfg_t alias_t;
  alias_t v07;
  initial begin
    `SVTEST_CHECK($bits(v07) == 8, "C07 typedef alias $bits")
  end

  // [C08] parameter declared with a class-scoped type
  parameter layout_pkg::LayoutRoot::cfg_t P08 = 0;
  initial begin
    `SVTEST_CHECK($bits(P08) == 8, "C08 parameter $bits")
  end

  // [C09] queue element type
  layout_pkg::LayoutRoot::cfg_t q09[$];
  initial begin
    q09.push_back(0);
    `SVTEST_CHECK(q09.size() == 1, "C09 queue element type")
  end

  // [C10] cast target type
  logic [7:0] raw10 = 8'h5A;
  layout_pkg::LayoutRoot::cfg_t m10;
  initial begin
    m10 = layout_pkg::LayoutRoot::cfg_t'(raw10);
    `SVTEST_CHECK(m10.mode_code == 2'b01, "C10 cast target field")
  end

  // [C11] struct member type
  typedef struct packed {
    layout_pkg::LayoutRoot::cfg_t m11;
    bit [3:0] pad11;
  } wrap11_t;
  wrap11_t w11;
  initial begin
    `SVTEST_CHECK($bits(w11) == 12, "C11 struct member $bits")
  end

  // [C12] parameterized class as scope prefix (§8.25.1)
  layout_pkg::VectorShape#(6)::vec_t v12a;
  VectorShape#(6)::vec_t v12b;
  initial begin
    `SVTEST_CHECK($bits(v12a) == 6, "C12 pkg-qualified specialization $bits")
    `SVTEST_CHECK($bits(v12b) == 6, "C12 bare specialization $bits")
  end

  // [C13] 3-level chain pkg::Outer::Inner::typedef (§8.23)
  layout_pkg::LayoutRoot::Sub::sub_t v13;
  initial begin
    `SVTEST_CHECK($bits(v13) == 4, "C13 3-level chain $bits")
  end

  // [C14] 2-level chain without the package prefix (imported class)
  LayoutRoot::Sub::sub_t v14;
  initial begin
    `SVTEST_CHECK($bits(v14) == 4, "C14 2-level chain $bits")
  end

  // [C15] pkg::class::typedef as a type-parameter DEFAULT
  class Wrap15 #(type T = layout_pkg::LayoutRoot::cfg_t);
    T v15;
  endclass
  Wrap15 w15;
  initial begin
    w15 = new();
    `SVTEST_CHECK($bits(w15.v15) == 8, "C15 type-param default $bits")
  end

  // [C16] enum-typed variable via class scope
  layout_pkg::LayoutRoot::op_e op16;
  initial begin
    op16 = layout_pkg::LayoutRoot::OP_B;
    `SVTEST_CHECK(op16 == 1, "C16 enum var value")
  end

  // [G01] read static class property (expression position)
  logic [7:0] xg1;
  initial begin
    xg1 = layout_pkg::LayoutRoot::s_shadow;
    `SVTEST_CHECK(xg1 == 8'hA5, "G01 static property read")
  end

  // [G03] static method call (before G02 rewrites the static)
  logic [7:0] xg3;
  initial begin
    xg3 = layout_pkg::LayoutRoot::get_shadow();
    `SVTEST_CHECK(xg3 == 8'hA5, "G03 static method call")
  end

  // [G02] write static class property
  initial begin
    layout_pkg::LayoutRoot::s_shadow = 8'h3C;
    `SVTEST_CHECK(layout_pkg::LayoutRoot::s_shadow == 8'h3C, "G02 static property write")
  end

  // [G05] enum label in pure expression position
  int ig5;
  initial begin
    ig5 = layout_pkg::LayoutRoot::OP_B;
    `SVTEST_CHECK(ig5 == 1, "G05 enum label value")
  end

  // [G09] out-of-block method with class-scoped return type (§8.24)
  class C09;
    typedef int T09;
    extern static function T09 f09();
  endclass
  function C09::T09 C09::f09();
    return 1;
  endfunction
  initial begin
    `SVTEST_CHECK(C09::f09() == 1, "G09 out-of-block method")
  end

  // [ORIG] the consumer-package methods that motivated the fix
  LayoutInspector arb;
  initial begin
    arb = new();
    `SVTEST_CHECK(arb.get_size_idx(40'h0, '0) == 6, "ORIG port-site method")
    arb.show_cfg();
  end

  final begin
    `SVTEST_PASSFAIL
  end
endmodule
