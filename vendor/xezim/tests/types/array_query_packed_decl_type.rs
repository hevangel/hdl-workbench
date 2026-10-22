//! The array query functions on a multi-dimensional packed variable report
//! per-dimension bounds (§20.7, §7.4.5) also where its storage registers no
//! packed shape: a class property (`m`, `this.m`, `o.m`) and a member of a
//! subroutine's struct formal (`a.wdata`). Both are stored as one flattened
//! vector, so `$size(a.wdata)` on `bit [256:0][511:0]` gave 131584 instead
//! of 257 (the axi4 AVIP's slave converter looped 131584 times), and
//! `$dimensions` counted one dimension. They are now answered from the
//! declared type, unpacked dimensions first.
//!
//! Expected values are the reference simulator's.

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

const SRC_SZ: &str = r#"
typedef struct { bit [3:0] id; bit [256:0][511:0] wdata; bit [3:0][7:0] w2; } s_t;
typedef struct packed { bit [3:0][7:0] w2; } p_t;
class c; bit [3:0][7:0] m; s_t s;
  static function int f(input s_t a); return $size(a.wdata); endfunction
  static function int g(input s_t a); return $size(a.w2); endfunction
endclass
module top;
  s_t s; p_t p; bit [3:0][7:0] v;
  initial begin
    c o = new();
    $display("T|a %0d %0d %0d %0d", $size(s.wdata), $size(s.w2), $size(v), $size(p.w2));
    $display("T|b %0d %0d %0d %0d", c::f(s), c::g(s), $size(o.m), $size(o.s.w2));
    $display("T|c %0d %0d %0d", $size(s.wdata, 2), $bits(s.wdata), $high(s.w2));
  end
endmodule
"#;

/// Struct members of a class-method formal, a class property and a member of a
/// struct property.
#[test]
fn query_packed_member_of_method_formal() {
    assert_eq!(
        t_lines(SRC_SZ),
        ["T|a 257 4 4 4", "T|b 257 4 4 4", "T|c 512 131584 3"]
    );
}

const SRC_SZ2: &str = r#"
typedef struct { bit [3:0] id; bit [3:0][7:0] w2; bit [7:0] u [3]; } s_t;
function automatic int mf(input s_t a); return $size(a.w2); endfunction
function automatic int mu(input s_t a); return $size(a.u); endfunction
class c; bit [3:0][7:0] m;
  static function int g(input s_t a); return $size(a.w2); endfunction
  function int h(); return $size(m); endfunction
endclass
module top;
  s_t s;
  initial begin
    c o = new(); s_t l;
    $display("T|%0d %0d %0d %0d %0d %0d", mf(s), mu(s), c::g(s), o.h(), $size(o.m), $size(l.w2));
  end
endmodule
"#;

/// A module function formal, a class-method formal, a property inside and
/// outside its class.
#[test]
fn query_packed_member_of_function_formal() {
    assert_eq!(t_lines(SRC_SZ2), ["T|4 3 4 4 4 4"]);
}

const SRC_SZ3: &str = r#"
typedef bit [3:0][7:0] w_t;
typedef struct { bit [1:0][2:0] q; } in_t;
typedef struct { bit [3:0] id; bit [256:0][511:0] wdata; w_t w; in_t inner; bit [0:3][7:0] asc; } s_t;
class c;
  bit [3:0][7:0] m; bit [8:1] m1; bit [0:2][3:0] ma; s_t s;
  function void show(input s_t a, ref s_t r);
    bit [5:0][1:0] lv;
    $display("T|m %0d %0d %0d %0d %0d", $size(m), $size(m, 2), $left(m1), $right(m1), $size(this.m));
    $display("T|ma %0d %0d %0d %0d", $left(ma), $right(ma), $increment(ma), $high(ma, 2));
    $display("T|a %0d %0d %0d %0d %0d", $size(a.wdata), $size(a.wdata, 2), $size(a.w), $size(a.inner.q), $left(a.asc));
    $display("T|r %0d %0d %0d", $size(r.wdata), $size(r.w, 2), $increment(r.asc));
    $display("T|lv %0d %0d", $size(lv), $size(lv, 2));
    $display("T|dim %0d %0d", $dimensions(a.wdata), $dimensions(m));
    for (int i = 0; i < $size(a.wdata); i += 64) $display("T|loop %0d", i);
  endfunction
  task automatic tk(input s_t a); $display("T|tk %0d %0d", $size(a.wdata), $size(a.w)); endtask
endclass
module top;
  s_t s;
  initial begin
    c o = new();
    o.show(s, s); o.tk(s);
    $display("T|o %0d %0d %0d %0d", $size(o.m), $left(o.m1), $size(o.s.wdata), $size(o.ma, 2));
  end
endmodule
"#;

/// Neighbouring shapes: a `ref` formal, a task formal, an ascending range,
/// a typedef'd and a nested member, a loop bound and `$dimensions`.
#[test]
fn query_packed_neighbour_shapes() {
    assert_eq!(
        t_lines(SRC_SZ3),
        [
            "T|m 4 8 8 1 4",
            "T|ma 0 2 -1 3",
            "T|a 257 512 4 2 0",
            "T|r 257 8 -1",
            "T|lv 6 2",
            "T|dim 2 2",
            "T|loop 0",
            "T|loop 64",
            "T|loop 128",
            "T|loop 192",
            "T|loop 256",
            "T|tk 257 4",
            "T|o 4 8 257 4",
        ]
    );
}

const SRC_SZ4: &str = r#"
typedef struct { bit [256:0][511:0] wdata; bit [7:0] u [3]; bit [3:0] v; } s_t;
class c; bit [3:0][7:0] m; bit [7:0] n; bit [1:0][3:0] um [2]; s_t s;
  function void show(input s_t a);
    $display("T|d %0d %0d %0d %0d %0d %0d", $dimensions(a.wdata), $dimensions(m), $dimensions(this.m), $dimensions(n), $dimensions(a.v), $dimensions(um));
    $display("T|u %0d %0d %0d %0d", $unpacked_dimensions(a.wdata), $unpacked_dimensions(m), $unpacked_dimensions(um), $unpacked_dimensions(a.u));
    $display("T|s %0d %0d %0d %0d", $size(um), $size(um, 2), $size(um, 3), $size(a.u));
  endfunction
endclass
module top; s_t s; initial begin c o = new(); o.show(s);
  $display("T|o %0d %0d %0d", $dimensions(o.m), $dimensions(o.s.wdata), $unpacked_dimensions(o.m)); end endmodule
"#;

/// `$dimensions` / `$unpacked_dimensions`, and a property that is an unpacked
/// array of packed arrays.
#[test]
fn query_dimension_counts_from_declared_type() {
    assert_eq!(
        t_lines(SRC_SZ4),
        ["T|d 2 2 2 1 1 3", "T|u 0 0 1 1", "T|s 2 2 4 3", "T|o 2 2 0"]
    );
}
