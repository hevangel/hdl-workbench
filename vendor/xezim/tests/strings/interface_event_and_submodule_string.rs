//! Two reported defects, both about a construct that worked at the TOP level
//! but silently misbehaved one level down the hierarchy.
//!
//! 1. §15.5.3 `@(<hier>.<event>.triggered)`. Only events declared at MODULE
//!    scope were recorded in the event set, so an event living in an
//!    INTERFACE — the ordinary `@(vif.evt.triggered)` synchronization pattern —
//!    matched nothing. The event control came out with an EMPTY sensitivity
//!    list, which is not a wait at all: the process resumed immediately at
//!    time 0 instead of at the trigger. Every event is backed by a 1-bit
//!    toggle signal, so the signal table is the reliable evidence, and the
//!    parser produces two different shapes for these paths (a flattened Ident
//!    and a MemberAccess chain) which both need the lookup.
//!
//! 2. §6.16 `string` declared in an instantiated module. A string has no
//!    declared width, so its initializer must not be fit to the fixed
//!    placeholder the elaborator hands a dynamic type. The top-level
//!    declaration path was exempt; the instance-inlining path was not, so text
//!    past 128 characters was truncated — and because string bytes pack
//!    LSB-first, it is the FRONT of the text that disappears, leaving a
//!    plausible-looking tail.

use xezim::simulate;

/// Reads a signal by its plain name, then under the usual instance prefixes.
fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    for cand in [n.to_string(), format!("top.{}", n)] {
        if let Some(v) = sim.get_signal(&cand) {
            return v
                .to_u64()
                .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", cand));
        }
    }
    panic!("signal not found: {}", n);
}

/// An event declared inside an interface: the wait must return at the trigger
/// time, not at time 0.
#[test]
fn triggered_on_interface_event_waits() {
    let src = r#"
interface my_if;
  event set_opt_reg;
  logic foo;
endinterface
module top;
  my_if u_if ();
  int ret = -1;
  initial begin
    @(u_if.set_opt_reg.triggered);
    ret = $time;
  end
  initial begin
    #50;
    -> u_if.set_opt_reg;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "ret"),
        50,
        "hierarchical .triggered must block until the event fires"
    );
}

/// The same wait reached while other processes are running, to confirm the
/// resume is the trigger and not merely the end of time.
#[test]
fn interface_event_resumes_only_on_trigger() {
    let src = r#"
interface my_if;
  event e;
endinterface
module top;
  my_if u_if ();
  int w = -1;
  int ticks = 0;
  always #5 ticks = ticks + 1;
  initial begin
    @(u_if.e.triggered);
    w = $time;
  end
  initial begin
    #30;
    -> u_if.e;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "w"), 30);
}

/// A module-scope event must keep working — the fallback must not shadow it.
#[test]
fn module_scope_event_triggered_still_waits() {
    let src = r#"
module top;
  event e;
  int m = -1;
  initial begin
    @(e.triggered);
    m = $time;
  end
  initial begin
    #17;
    -> e;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "m"), 17);
}

/// A struct member genuinely NAMED `triggered` is a value, not an event, and
/// must still read as an ordinary signal.
#[test]
fn member_named_triggered_is_not_an_event() {
    let src = r#"
module top;
  typedef struct packed { logic triggered; logic other; } st_t;
  st_t s;
  int v = -1;
  initial begin
    s.triggered = 1'b0;
    #5 s.triggered = 1'b1;
    #5 v = s.triggered;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "v"), 1);
}

const LONG: &str = "HEAD_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef_TAIL_XYZ";

/// A >128-character string initializer inside an INSTANTIATED module keeps its
/// leading characters and its full length.
#[test]
fn long_string_in_instantiated_module_is_not_truncated() {
    let src = format!(
        r#"
module child;
  string s = "{LONG}";
  int slen;
  int c0, c4;
  initial begin
    slen = s.len();
    c0 = s.getc(0);
    c4 = s.getc(4);
  end
endmodule
module top;
  child u ();
endmodule
"#
    );
    let sim = simulate(&src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "u.slen"), LONG.len() as u64, "length");
    assert_eq!(u(&sim, "u.c0"), b'H' as u64, "front of the text was lost");
    assert_eq!(u(&sim, "u.c4"), b'_' as u64);
}

/// The same string two levels down, in two instances, and written procedurally
/// rather than at the declaration.
#[test]
fn long_strings_survive_nesting_and_multiple_instances() {
    let src = format!(
        r#"
module gchild;
  string g;
  int glen, gc0;
  initial begin
    g = "{LONG}";
    glen = g.len();
    gc0 = g.getc(0);
  end
endmodule
module child #(parameter int ID = 0);
  string s = "{LONG}";
  string t;
  int slen, sc0, tlen, tc0;
  gchild gg ();
  initial begin
    t = {{"P", s}};
    slen = s.len();
    sc0 = s.getc(0);
    tlen = t.len();
    tc0 = t.getc(0);
  end
endmodule
module top;
  child #(1) u1 ();
  child #(2) u2 ();
endmodule
"#
    );
    let sim = simulate(&src, 200).expect("simulate failed");
    let n = LONG.len() as u64;
    for inst in ["u1", "u2"] {
        assert_eq!(u(&sim, &format!("{}.slen", inst)), n, "{} decl len", inst);
        assert_eq!(u(&sim, &format!("{}.sc0", inst)), b'H' as u64, "{}", inst);
        assert_eq!(u(&sim, &format!("{}.tlen", inst)), n + 1, "{} concat", inst);
        assert_eq!(u(&sim, &format!("{}.tc0", inst)), b'P' as u64, "{}", inst);
        assert_eq!(
            u(&sim, &format!("{}.gg.glen", inst)),
            n,
            "{} grandchild len",
            inst
        );
        assert_eq!(
            u(&sim, &format!("{}.gg.gc0", inst)),
            b'H' as u64,
            "{}",
            inst
        );
    }
}

/// A short string in a submodule must be unaffected — the exemption must not
/// disturb values that already fit.
#[test]
fn short_string_in_submodule_unchanged() {
    let src = r#"
module child;
  string s = "abc";
  int slen, c0, c2;
  initial begin
    slen = s.len();
    c0 = s.getc(0);
    c2 = s.getc(2);
  end
endmodule
module top;
  child u ();
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "u.slen"), 3);
    assert_eq!(u(&sim, "u.c0"), b'a' as u64);
    assert_eq!(u(&sim, "u.c2"), b'c' as u64);
}
