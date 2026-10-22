//! §10.9.2: a NAMED assignment pattern in a CONTINUOUS whole-struct assign.
//!
//! The PROCEDURAL form already worked and is covered by
//! `struct_named_patterns.rs` (`s = '{x: 5, y: 8'h22};` inside an
//! initial block). This is the continuous form, which takes a different
//! path -- `expand_whole_struct_continuous_assigns` ->
//! `emit_struct_member_assigns` -- and did not handle named items at all.
//!
//! An unpacked struct is stored one signal per member, so
//! `assign s = <pattern>` is expanded member-wise. That expansion took the
//! pattern apart only when every item was ORDERED:
//!
//!     assign s = '{3.0e-3, 1.8086};        // positional -- worked
//!     assign s = '{i: 3.0e-3, v: 1.8086};  // named      -- wrote ZEROS
//!
//! The map returned None for a named item and `collect::<Option<Vec<_>>>`
//! short-circuits the whole pattern on the first one, so the fallback ran:
//! member-selecting the literal, `'{i: ..., v: ...}.i`, which the expansion's
//! own comment already identifies as having no evaluation path and yielding
//! 0. The guard was written for that hazard and then reached from the other
//! side.
//!
//! Silent, and the failure is a struct of zeros -- not an error, not an X.
//! §10.9.2 gives both forms equal standing, and named is what anyone writes
//! for a struct whose member order is not self-evident.
//!
//! These assert VALUES. A test that only checked "the assign happened" would
//! have passed against the bug: it did happen, onto members that stayed zero.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

/// The two forms must agree. Ordered was correct before and must stay so.
#[test]
fn named_and_ordered_patterns_agree() {
    let src = r#"
module top;
  typedef struct { real i; real v; } nd_t;
  nd_t ord;
  nd_t nam;
  assign ord = '{3.0e-3, 1.8086};
  assign nam = '{i: 3.0e-3, v: 1.8086};
  initial begin
    #1;
    $display("NOTE: ord i=%0.4f v=%0.4f", ord.i * 1e3, ord.v);
    $display("NOTE: nam i=%0.4f v=%0.4f", nam.i * 1e3, nam.v);
    $finish;
  end
endmodule
"#;
    assert_eq!(
        notes(src),
        vec!["NOTE: ord i=3.0000 v=1.8086", "NOTE: nam i=3.0000 v=1.8086"]
    );
}

/// Named items bind by NAME, so writing them out of declaration order must
/// still land on the right members. Position-mapping a named pattern would
/// pass the test above and fail this one.
#[test]
fn named_items_bind_by_name_not_position() {
    let src = r#"
module top;
  typedef struct { real i; real v; } nd_t;
  nd_t s;
  assign s = '{v: 1.8086, i: 3.0e-3};   // deliberately reversed
  initial begin
    #1;
    $display("NOTE: i=%0.4f v=%0.4f", s.i * 1e3, s.v);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: i=3.0000 v=1.8086"]);
}

/// Mixed member types, and a nested struct, since the expansion recurses.
#[test]
fn named_pattern_with_mixed_and_nested_members() {
    let src = r#"
module top;
  typedef struct { real v; bit ok; } inner_t;
  typedef struct { real i; inner_t sub; } outer_t;
  outer_t s;
  assign s = '{i: 2.5e-3, sub: '{v: 1.8086, ok: 1'b1}};
  initial begin
    #1;
    $display("NOTE: i=%0.4f v=%0.4f ok=%0b", s.i * 1e3, s.sub.v, s.sub.ok);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: i=2.5000 v=1.8086 ok=1"]);
}

/// A named pattern feeding a module port -- the shape that made this look
/// like a struct-PORT defect. The port was always fine; the value reaching
/// it was already zeros.
#[test]
fn named_pattern_survives_a_module_port() {
    let src = r#"
package p;
  typedef struct { real i; real v; } nd_t;
endpackage
module child import p::*; (input nd_t d);
  initial begin
    #2;
    $display("NOTE: child i=%0.4f v=%0.4f", d.i * 1e3, d.v);
  end
endmodule
module top;
  import p::*;
  nd_t s;
  assign s = '{i: 3.0e-3, v: 1.8086};
  child c (.d(s));
  initial begin
    #1;
    $display("NOTE: parent i=%0.4f v=%0.4f", s.i * 1e3, s.v);
    #3;
    $finish;
  end
endmodule
"#;
    assert_eq!(
        notes(src),
        vec![
            "NOTE: parent i=3.0000 v=1.8086",
            "NOTE: child i=3.0000 v=1.8086"
        ]
    );
}
