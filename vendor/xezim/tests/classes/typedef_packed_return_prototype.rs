//! Issue #284: IEEE 1800-2023 A.2.2.1 `data_type ::= ... | [ class_scope |
//! package_scope ] type_identifier { packed_dimension }`. A subroutine
//! prototype rejected a packed dimension on a typedef-named return type
//! (`extern virtual function M [1:0] low_two(...)`), though the same type
//! worked on a function with a body. Prototypes now share the declaration's
//! return-type parser. Every expected value below comes from the reference
//! simulator.

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
fn issue_284_reproducer() {
    assert_eq!(
        t_lines(
            r#"
// typedef_return.sv
module top;
    typedef bit [31:0] M;

    class c;
        extern virtual function M [1:0] low_two(bit [511:0] w);
    endclass

    function M [1:0] c::low_two(bit [511:0] w);
        return 64'hdead_beef_cafe_f00d;
    endfunction

    c h;
    initial begin
        M [1:0] v;
        h = new();
        v = h.low_two(512'h0);
        if (v === 64'hdead_beef_cafe_f00d) $display("PASS: M [1:0] returned %h", v);
        else                               $display("FAIL: M [1:0] returned %h", v);
    end
endmodule
"#
        ),
        ["PASS: M [1:0] returned deadbeefcafef00d"],
    );
}

/// `extern` virtual, local and static functions, an implicit `[7:0]`
/// return, an `extern` task, pure virtual and interface-class prototypes,
/// each matched by its out-of-block or overriding definition.
#[test]
fn prototype_forms_with_packed_typedef_returns() {
    assert_eq!(
        t_lines(
            r#"
typedef bit [31:0] M;
typedef logic [7:0] B;
interface class IC;
  pure virtual function M [1:0] icf(int k);
endclass
virtual class VB;
  pure virtual function B [3:0] pv(input int k);
endclass
class impl extends VB implements IC;
  virtual function M [1:0] icf(int k);
    return {32'(k), 32'(k + 1)};
  endfunction
  virtual function B [3:0] pv(input int k);
    return {B'(k), B'(k + 1), B'(k + 2), B'(k + 3)};
  endfunction
endclass
class c;
  extern virtual function M [1:0] low_two(bit [511:0] w);
  extern local function B [1:0] loc_f();
  extern static function M [0:0] st_f();
  extern function [7:0] imp_f();
  extern task tk(output M [1:0] o);
  function B [1:0] call_loc(); return loc_f(); endfunction
endclass
function M [1:0] c::low_two(bit [511:0] w);
  return 64'hdead_beef_cafe_f00d ^ w[63:0];
endfunction
function B [1:0] c::loc_f();
  return 16'hab_cd;
endfunction
function M [0:0] c::st_f();
  return 32'h1234_5678;
endfunction
function [7:0] c::imp_f();
  return 8'h5a;
endfunction
task c::tk(output M [1:0] o);
  o = {32'h1, 32'h2};
endtask
module top;
  c h;
  impl im;
  IC ic;
  VB vb;
  M [1:0] v;
  initial begin
    h = new();
    v = h.low_two(512'h0);
    $display("T|low_two=%h bits=%0d", v, $bits(v));
    $display("T|loc=%h st=%h imp=%h", h.call_loc(), c::st_f(), h.imp_f());
    h.tk(v);
    $display("T|tk=%h", v);
    im = new(); ic = im; vb = im;
    $display("T|icf=%h pv=%h", ic.icf(5), vb.pv(1));
  end
endmodule
"#
        ),
        [
            "T|low_two=deadbeefcafef00d bits=64",
            "T|loc=abcd st=12345678 imp=5a",
            "T|tk=0000000100000002",
            "T|icf=0000000500000006 pv=01020304",
        ],
    );
}
