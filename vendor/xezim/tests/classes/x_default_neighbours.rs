//! §6.8 / §8.4 / §6.24.2 / §7.8.6: initial values and defaults next to the
//! aggregate class-property x defaults.
//!
//! - An unpacked array of class handles, and a class-handle member of an
//!   unpacked struct, start null; they started x, so `h == null` was x.
//! - A static class property of a 4-state enum or packed struct type with
//!   no initializer starts x, like an instance property; it started 0.
//! - A read of a missing key of a class associative array of packed
//!   structs yields x at the element's width (it gave a 32-bit 0).
//! - `$cast` to an enum fails for a source with x or z bits.
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
fn class_handle_arrays_and_struct_members_start_null() {
    let src = r#"
class obj; int v; endclass
typedef struct {obj h; int n;} hs_t;
typedef struct {logic [3:0] arr[3]; bit [3:0] barr[2]; obj h; integer n;} box_t;
class withint; int ia[3]; logic [3:0] la[3]; endclass
module top;
  obj oa[2]; obj o1; hs_t hs; box_t mbox;
  withint wa[2];
  initial begin
    $display("T|null %0d %0d %0d %0d", oa[1] == null, o1 == null, hs.h == null, oa[0] === null);
    $display("T|cond %0d", (oa[1] == null) ? 1 : 0);
    $display("T|box %b %b %0d", mbox.arr[1], mbox.barr[0], mbox.h == null);
    wa[0] = new;
    $display("T|wa %0d %b %0d", wa[0].ia[2], wa[0].la[1], wa[1] == null);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|null 1 1 1 1",
            "T|cond 1",
            "T|box xxxx 0000 1",
            "T|wa 0 xxxx 1"
        ]
    );
}

#[test]
fn four_state_statics_start_unknown() {
    let src = r#"
typedef enum logic [1:0] {E0, E1, E2} le_t;
typedef enum {I0, I1} ie_t;
typedef struct packed {bit [3:0] b; logic [3:0] l;} mix_t;
typedef struct packed {logic [3:0] d; logic [3:0] e;} pk_t;
class sc;
  static logic [3:0] s; static int si; static le_t se; static ie_t ie;
  static mix_t sm; static pk_t sp; static le_t sinit = E2;
endclass
module top;
  initial begin
    $display("T|static %b %0d %b %0d", sc::s, sc::si, sc::se, sc::ie);
    $display("T|packed %b %b %b", sc::sm, sc::sp, sc::sinit);
    sc::se = E1;
    $display("T|write %b", sc::se);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|static xxxx 0 xx 0",
            "T|packed xxxxxxxx xxxxxxxx 10",
            "T|write 01"
        ]
    );
}

#[test]
fn missing_packed_struct_assoc_element() {
    let src = r#"
typedef struct packed {logic [3:0] d; bit [3:0] f;} prec_t;
typedef struct packed {bit [3:0] d; bit [3:0] f;} brec_t;
class holder; prec_t pa[int]; brec_t ba[int]; endclass
module top;
  holder h;
  initial begin
    h = new;
    $display("T|miss %b %b %0d", h.pa[3], h.ba[3], h.pa.num());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|miss xxxxxxxx 00000000 0"]);
}

#[test]
fn enum_cast_of_unknown_value_fails() {
    let src = r#"
typedef enum logic [1:0] {E0, E1, E2} le_t;
module top;
  le_t ce; int ok;
  initial begin
    ok = $cast(ce, 3);
    $display("T|cast %0d %b", ok, ce);
    ok = $cast(ce, 4'bx01);
    $display("T|castx %0d %b", ok, ce);
    ok = $cast(ce, 2'bz1);
    $display("T|castz %0d %b", ok, ce);
    ok = $cast(ce, 1);
    $display("T|ok %0d %b", ok, ce);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|cast 0 xx", "T|castx 0 xx", "T|castz 0 xx", "T|ok 1 01"]
    );
}
