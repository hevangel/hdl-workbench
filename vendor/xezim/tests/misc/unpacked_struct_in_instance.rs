//! §7.2 / §10.9.2 — unpacked structs declared in an INSTANTIATED module.
//! Three defects found by sweeping the same construct through every
//! assignment path and both scopes; all reference-validated.
//!
//! 1. **Two distinct unpacked structs in a child module SHARED storage.** The
//!    inlining path registered one packed vector per struct instead of the
//!    per-member signals the top-level path creates, so a member write landed
//!    in an ad-hoc runtime entry and a member read of an unwritten struct fell
//!    through the name-resolution suffix scan into *another* struct's member:
//!    writing `one.a` made `two.a` read back that value instead of `x`. This
//!    is the dangerous one — it corrupts silently and in the reader, not the
//!    writer.
//! 2. **A pattern assigned to an instance-scoped struct wrote array elements.**
//!    The "struct-field sub-path is an unregistered queue" heuristic keyed off
//!    `base.contains('.')`, which is equally true of an INSTANCE-qualified
//!    name (`u.one`), so it registered the struct as a dynamic array and wrote
//!    `u.one[0]`/`u.one[1]`, leaving every member `x`.
//! 3. **A nonblocking pattern assign never spread member-wise at all.** Only
//!    the blocking path called the aggregate spread, so `s <= '{a:1, b:2};`
//!    left the members `x` while the identical blocking assignment worked.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Distinct structs in a child module must not share storage. The x-ness is
/// checked INSIDE the design (`$isunknown`) so the assertion sees exactly what
/// the SV name resolution sees.
#[test]
fn child_unpacked_structs_do_not_alias() {
    let src = r#"
module leaf;
  typedef struct { logic [7:0] a; logic [7:0] b; } s_t;
  s_t one, two;
  initial begin
    one.a = 8'h11;
    one.b = 8'h22;
    // `two` is deliberately never written
  end
endmodule
module tb;
  leaf u();
  int one_a, one_b, two_a_unknown, two_b_unknown;
  initial begin
    #1;
    one_a = u.one.a;
    one_b = u.one.b;
    two_a_unknown = $isunknown(u.two.a);
    two_b_unknown = $isunknown(u.two.b);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "one_a"), 0x11);
    assert_eq!(u(&sim, "one_b"), 0x22);
    assert_eq!(
        u(&sim, "two_a_unknown"),
        1,
        "an unwritten struct must stay x, not alias"
    );
    assert_eq!(
        u(&sim, "two_b_unknown"),
        1,
        "an unwritten struct must stay x, not alias"
    );
}

/// Patterns, member writes and whole copies all work inside an instance,
/// through a typedef and an inline struct alike.
#[test]
fn child_unpacked_struct_pattern_assignment() {
    let src = r#"
module leaf;
  typedef struct { logic [7:0] a; logic [7:0] b; } s_t;
  struct { logic [7:0] a; logic [7:0] b; } inl;
  s_t named, ordered, copied;
  initial begin
    named   = '{a:8'h11, b:8'h22};
    ordered = '{8'h33, 8'h44};
    inl     = '{a:8'h55, b:8'h66};
    copied  = named;
  end
endmodule
module tb;
  leaf u();
  int na, nb, oa, ob, ia, ib, ca, cb;
  initial begin
    #1;
    na = u.named.a;   nb = u.named.b;
    oa = u.ordered.a; ob = u.ordered.b;
    ia = u.inl.a;     ib = u.inl.b;
    ca = u.copied.a;  cb = u.copied.b;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "na"), u(&sim, "nb")),
        (0x11, 0x22),
        "named pattern"
    );
    assert_eq!(
        (u(&sim, "oa"), u(&sim, "ob")),
        (0x33, 0x44),
        "ordered pattern"
    );
    assert_eq!(
        (u(&sim, "ia"), u(&sim, "ib")),
        (0x55, 0x66),
        "inline struct type"
    );
    assert_eq!((u(&sim, "ca"), u(&sim, "cb")), (0x11, 0x22), "whole copy");
}

/// A nonblocking pattern assign must spread member-wise, like the blocking one.
#[test]
fn nonblocking_pattern_spreads_into_an_unpacked_struct() {
    let src = r#"
module tb;
  typedef struct { logic [7:0] a; logic [7:0] b; } s_t;
  logic clk = 0;
  always #5 clk = ~clk;
  s_t named, ordered;
  int na, nb, oa, ob;
  always_ff @(posedge clk) named   <= '{a:8'h77, b:8'h88};
  always_ff @(posedge clk) ordered <= '{8'h99, 8'hAA};
  initial begin
    @(posedge clk);
    #1;
    na = named.a;   nb = named.b;
    oa = ordered.a; ob = ordered.b;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(
        (u(&sim, "na"), u(&sim, "nb")),
        (0x77, 0x88),
        "named via NBA"
    );
    assert_eq!(
        (u(&sim, "oa"), u(&sim, "ob")),
        (0x99, 0xAA),
        "ordered via NBA"
    );
}
