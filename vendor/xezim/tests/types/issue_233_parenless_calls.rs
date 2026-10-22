//! Issue #233: zero-argument calls keep their call semantics when the empty
//! argument list is omitted. Enum calls have focused coverage elsewhere; this
//! module covers the other receiver and scope forms from the report.

use xezim::simulate;

#[test]
fn zero_argument_calls_work_without_parentheses() {
    let source = r#"
package call_pkg;
  function automatic int package_value(); return 7; endfunction
endpackage

function automatic int root_value(); return 42; endfunction

interface link_if;
  function int value(); return 3; endfunction
endinterface

class holder_c;
  int entries[$];
  function void fill(); entries.push_back(3); entries.push_back(4); endfunction
  function int from_root(); return root_value; endfunction
  function int take_front(); return entries.pop_front; endfunction
  function int take_back(); return entries.pop_back; endfunction
endclass

module top;
  import call_pkg::*;
  link_if link();
  holder_c holder = new;
  string word = "hello";
  int result;
  string changed;

  function automatic int wrapped_root(); return root_value; endfunction
  function automatic int wrapped_package(); return package_value; endfunction

  initial begin
    result = root_value;              $display("T root %0d", result);
    result = wrapped_root();          $display("T wrapped %0d", result);
    result = package_value;           $display("T imported %0d", result);
    result = call_pkg::package_value; $display("T scoped %0d", result);
    result = wrapped_package();       $display("T package_body %0d", result);
    result = word.len;                $display("T strlen %0d", result);
    changed = word.toupper;           $display("T upper %0s", changed);
    result = holder.from_root();      $display("T class_body %0d", result);
    holder.fill();
    result = holder.take_front();     $display("T front %0d %0d", result, holder.entries.size());
    result = holder.take_back();      $display("T back %0d %0d", result, holder.entries.size());
    result = link.value;              $display("T interface %0d", result);
  end
endmodule
"#;
    let sim = simulate(source, 1000).expect("simulate issue #233 matrix");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .filter(|line| line.starts_with("T "))
        .collect();
    assert_eq!(
        lines,
        [
            "T root 42",
            "T wrapped 42",
            "T imported 7",
            "T scoped 7",
            "T package_body 7",
            "T strlen 5",
            "T upper HELLO",
            "T class_body 42",
            "T front 3 1",
            "T back 4 0",
            "T interface 3",
        ]
    );
}

/// A package-QUALIFIED parenless call inside a subroutine body parses as a
/// member access on the package name (`vp::pkf` -> MemberAccess{vp, pkf});
/// read as a member it returned 0. Every subroutine kind, and the package's
/// own functions, must call it.
#[test]
fn qualified_package_call_without_parentheses_in_subroutine_bodies() {
    let source = r#"
package vp;
  function automatic int pkf(); return 7; endfunction
  function automatic int twice(); return 2 * vp::pkf; endfunction
endpackage
function automatic int wrapk(); return vp::pkf; endfunction
function automatic int wrapq(); int r; r = vp::pkf; return r; endfunction
task automatic tk(output int o); o = vp::pkf + 1; endtask
class K; function int m(); return vp::pkf + 2; endfunction endclass
module top;
  initial begin
    int r; K k = new;
    r = wrapk(); $display("T function %0d", r);
    r = wrapq(); $display("T assign %0d", r);
    tk(r);       $display("T task %0d", r);
    $display("T method %0d", k.m());
    $display("T in_package %0d", vp::twice());
  end
endmodule
"#;
    let sim = simulate(source, 1000).expect("simulate");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .filter(|line| line.starts_with("T "))
        .collect();
    assert_eq!(
        lines,
        [
            "T function 7",
            "T assign 7",
            "T task 8",
            "T method 9",
            "T in_package 14"
        ]
    );
}

/// §6.19.6: the index variable of `foreach` over an associative array keyed
/// by an enum has that enum type, so its methods resolve with or without
/// parentheses — also when the array is a procedural local (elaboration
/// records the key type for module-scope arrays only).
#[test]
fn foreach_key_of_local_enum_keyed_array_has_the_enum_type() {
    let source = r#"
typedef enum logic [5:0] { C0 = 6'd0, C1 = 6'd1, CC = 6'd63 } e_t;
function automatic void in_function();
  int aa[e_t];
  int plain[int];
  aa[C1] = 1;
  plain[5] = 1;
  foreach (aa[k]) $display("T fn first=%0d last=%0d num=%0d name=%s next=%0d",
                           k.first, k.last, k.num, k.name, k.next);
  // an int-keyed array in the same frame keeps an untyped key
  foreach (plain[k]) $display("T fn plain k=%0d", k);
endfunction
module top;
  initial begin
    int aa[e_t];
    aa[C0] = 10; aa[CC] = 5;
    foreach (aa[k])
      $display("T init k=%0d first=%0d last=%0d num=%0d name=%s next=%0d last()=%0d",
               k, k.first, k.last, k.num, k.name, k.next, k.last());
    in_function();
  end
endmodule
"#;
    let sim = simulate(source, 1000).expect("simulate");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .filter(|line| line.starts_with("T "))
        .collect();
    assert_eq!(
        lines,
        [
            "T init k=0 first=0 last=63 num=3 name=C0 next=1 last()=63",
            "T init k=63 first=0 last=63 num=3 name=CC next=0 last()=63",
            "T fn first=0 last=63 num=3 name=C1 next=63",
            "T fn plain k=5",
        ]
    );
}
