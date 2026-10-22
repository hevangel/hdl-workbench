//! §25.9/§8.25/§6.22.1: a parameterized class specialized with a virtual
//! interface type keys its statics by that TYPE, however it is spelled:
//! with the enclosing class's parameters (`db#(virtual my_if#(T))` inside
//! `mon#(int)`), through a class-local or module typedef, with the
//! `interface` keyword, named or defaulted interface parameters, or reached
//! through `extends db#(virtual my_if#(T))`. The argument text used to be
//! keyed verbatim, so each spelling had its own empty table (issue #320,
//! reported by Ganesh-TNL). Expected values come from the reference
//! simulator.

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

#[test]
fn vif_spec_keys_through_class_params() {
    let expected = [
        "T|m_size_vifT|1",
        "T|m_size_vif_t|1",
        "T|m_size_vif_int|1",
        "T|m_size_T|1",
        "T|m_cnt_vifT|1",
        "T|m_size_vifTN|1",
        "T|m_size_typedef|1",
        "T|mb_size_vifT|1",
        "T|mb_size_vif_t|1",
        "T|mb_size_vif_int|2",
        "T|mb_size_T|0",
        "T|mb_cnt_vifT|1",
        "T|mb_size_vifTN|1",
        "T|mb_size_typedef|2",
        "T|m8_size_vifT|2",
        "T|m8_size_vif_t|2",
        "T|m8_size_vif_int|2",
        "T|m8_size_T|1",
        "T|m8_cnt_vifT|2",
        "T|m8_size_vifTN|1",
        "T|m8_size_typedef|2",
        "T|child_size_vifT|3",
        "T|child_size_vif_t|3",
        "T|child_size_vif_int|3",
        "T|child_size_T|1",
        "T|child_cnt_vifT|3",
        "T|child_size_vifTN|3",
        "T|child_size_typedef|3",
        "T|wrap|4",
        "T|wrapm|1",
        "T|mod_after|4",
        "T|mod_after_typedef|4",
    ];
    assert_eq!(
        t_lines(include_str!("vif_spec_keys_class_params.sv")),
        expected
    );
}

#[test]
fn vif_spec_keys_across_spellings() {
    let expected = [
        "T|noargs|1",
        "T|kw|1",
        "T|named|1",
        "T|named_both|1",
        "T|full|1",
        "T|other_n|0",
        "T|other_t|0",
        "T|kw_typedef|1",
        "T|modport|0",
        "T|modport_typedef|0",
        "T|after_inst|2",
        "T|inst_view|2",
        "T|after_ext|3",
        "T|after_ext2|4",
        "T|ext2_view|4",
    ];
    assert_eq!(
        t_lines(include_str!("vif_spec_keys_spellings.sv")),
        expected
    );
}

/// Objects declared `db#(virtual my_if#(T))` (a property, a typedef'd
/// local) inside `mon#(int)` share `db#(virtual my_if#(int))`'s statics.
#[test]
fn vif_spec_keys_of_declared_objects() {
    let expected = [
        "T|int_after_m|3",
        "T|byte_after_m|0",
        "T|int_after_mb|3",
        "T|byte_after_mb|3",
    ];
    assert_eq!(t_lines(include_str!("vif_spec_keys_declared.sv")), expected);
}

/// §8.25: value parameters match by VALUE — `P#(2+2)` is `P#(4)`.
#[test]
fn spec_value_args_match_by_value() {
    const SRC: &str = r#"
class P #(int N = 4); static int c; endclass
interface vi_if #(int W = 4); logic [W-1:0] d; endinterface
class D #(type T = int); static int c; endclass
module tb;
  initial begin
    P#(4)::c++;
    $display("T|cls_expr|%0d", P#(2+2)::c);
    D#(virtual vi_if#(4))::c++;
    $display("T|vif_expr|%0d", D#(virtual vi_if#(2+2))::c);
    $display("T|vif_hex|%0d", D#(virtual vi_if#('h4))::c);
    $finish;
  end
endmodule
"#;
    let expected = ["T|cls_expr|1", "T|vif_expr|1", "T|vif_hex|1"];
    assert_eq!(t_lines(SRC), expected);
}

/// §6.22.1: two typedefs of the same queue type (`int[$]`, one through a
/// class type parameter) are the same specialization argument.
#[test]
fn spec_typedef_queue_args_match() {
    const SRC: &str = r#"
class db #(type T = int);
  static int cnt;
endclass
typedef int intq_t[$];
class mon #(type T = int);
  typedef T tq_t[$];
  function int n(); return db#(tq_t)::cnt; endfunction
endclass
module tb;
  initial begin
    mon#(int) m = new;
    db#(intq_t)::cnt++;
    $display("T|typedef_queue|%0d", m.n());
    $finish;
  end
endmodule
"#;
    let expected = ["T|typedef_queue|1"];
    assert_eq!(t_lines(SRC), expected);
}

/// §25.9/§8.9: a virtual interface held in a STATIC class property (plain,
/// queue element, function return, `static T` of `G#(virtual pif)`).
#[test]
fn static_class_vif_properties() {
    const SRC: &str = r#"
interface pif; int data; endinterface
class S;
  static virtual pif spv;
  static virtual pif pq[$];
  static function virtual pif getp(); return spv; endfunction
endclass
class G #(type T = int);
  static T one;
  static function void put(T v); one = v; endfunction
endclass
module tb;
  pif pi();
  initial begin
    virtual pif p;
    pi.data = 9;
    S::spv = pi; p = S::spv; $display("T|static_vif|%0d", p.data);
    p = S::getp(); $display("T|static_fn_ret|%0d", p.data);
    S::pq.push_back(pi); p = S::pq[0]; $display("T|static_q|%0d", p.data);
    G#(virtual pif)::put(pi);
    p = G#(virtual pif)::one; $display("T|gen_one|%0d", p.data);
    $finish;
  end
endmodule
"#;
    let expected = [
        "T|static_vif|9",
        "T|static_fn_ret|9",
        "T|static_q|9",
        "T|gen_one|9",
    ];
    assert_eq!(t_lines(SRC), expected);
}

/// §8.25 siblings: products, shifts, unsized based and sized literals for a
/// typed `int` parameter, named arguments, design parameters and partial
/// argument lists match by value; an UNTYPED parameter takes a sized
/// literal's width into its type, so `Q#(4'd4)` is not `Q#(4)`.
#[test]
fn spec_value_args_by_value_siblings() {
    let expected = [
        "T|mul|1",
        "T|shift|1",
        "T|paren|1",
        "T|dec|1",
        "T|sized|1",
        "T|named|1",
        "T|lp|1",
        "T|lpx|1",
        "T|dflt|1",
        "T|q_sized|0",
        "T|q_expr|1",
        "T|r_expr|1",
        "T|r_part|1",
        "T|vif_mul|1",
        "T|vif_dflt|1",
    ];
    assert_eq!(
        t_lines(include_str!("spec_value_args_by_value.sv")),
        expected
    );
}

/// §6.22.1 siblings: typedefs of the same unpacked array type — a queue,
/// a dynamic array, a `[string]` associative array, `[3]` and `[0:2]`,
/// a typedef of a typedef, a package typedef, `int signed` — are one type
/// argument; `[2:0]`, `bit [31:0]` and `byte` elements are other types.
#[test]
fn spec_typedef_type_args_siblings() {
    let expected = [
        "T|q2|1",
        "T|q3|1",
        "T|pkg|1",
        "T|isq|1",
        "T|bq|0",
        "T|byq|0",
        "T|cls|1",
        "T|cls_b|0",
        "T|dyn|5 5",
        "T|fix|7 0",
        "T|aa|9",
    ];
    assert_eq!(t_lines(include_str!("spec_typedef_type_args.sv")), expected);
}

/// §25.9/§8.9 siblings: a static virtual interface written and read from a
/// static method, an instance method, through a handle, a derived class, a
/// direct `S::spv = ...`, a `static T` of `G#(virtual pif)` beside
/// `G#(int)`, and cleared with `null`.
#[test]
fn static_vif_property_shapes() {
    let expected = [
        "T|static_set|4",
        "T|inst_get_static|4",
        "T|inst_set|9",
        "T|via_handle|9",
        "T|derived|9",
        "T|derived_fn|9",
        "T|direct_write|4",
        "T|gen_get|9",
        "T|gen_one|9",
        "T|gen_int|5",
        "T|null|1",
    ];
    assert_eq!(
        t_lines(include_str!("static_vif_property_shapes.sv")),
        expected
    );
}

/// §25.9/§25.10/§8.9: an ELEMENT of a vif queue, dynamic or fixed array —
/// an instance property, a static array or a static queue — assigned to a
/// `virtual` variable or returned from a static method binds the variable to
/// that element's interface; `G#(virtual pif)::one.data` reads through the
/// specialization's static vif. Each assignment used to leave the variable
/// on its previous interface.
#[test]
fn vif_collection_element_shapes() {
    let expected = [
        "T|q0|1",
        "T|q1|2",
        "T|qf|3 3",
        "T|qd|1 2",
        "T|d1|2",
        "T|f0|2 2",
        "T|s1|1",
        "T|s0|2 1 2",
        "T|sh|1",
        "T|sg|2",
        "T|sq|2 2",
        "T|g1|2",
        "T|g2|3",
    ];
    assert_eq!(
        t_lines(include_str!("vif_collection_element_shapes.sv")),
        expected
    );
}
