//! Three audit findings, one test file — all reference-validated.
//!
//! 1. **§8.20 non-virtual dispatch.** Every method call dispatched from the
//!    object's RUNTIME class, so `base_h.nonvirt()` on a handle holding a
//!    Derived ran Derived's override. A non-virtual method binds to the
//!    receiver's DECLARED type. The parser had captured the `virtual`
//!    qualifier all along (`ClassMethod.qualifiers`) — it was simply never
//!    consulted. Static binding is gated hard: known declared class, a real
//!    body found in its chain, and NO definition in the chain virtual —
//!    because an override of a virtual method is virtual without the keyword.
//! 2. **§9.7 `process::RUNNING`/`WAITING`/… constants** resolved to 0 (the
//!    built-in class is never user-declared, so the static-property lookup
//!    found nothing) — `p.status() == process::WAITING` was ALWAYS false
//!    even though `status()` itself was right.
//! 3. **§21.3.4.2 `$fscanf` file position**: the implementation pulled a whole
//!    LINE and discarded whatever the format didn't match, so a partial read
//!    dropped the rest of the line and `$feof` then reported end-of-file with
//!    data still unread — breaking the `while (!$feof(fd))` idiom.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// Non-virtual binds by declared type; virtual (explicit or inherited-implicit)
/// binds by runtime type.
#[test]
fn nonvirtual_binds_declared_virtual_binds_runtime() {
    let src = r#"
module top;
  class A;
    virtual function int v(); return 1; endfunction
    function int nv(); return 10; endfunction
  endclass
  class B extends A;
    function int v(); return 2; endfunction     // no keyword: still virtual
    function int nv(); return 20; endfunction
  endclass
  A a_h; B b_h;
  int av, anv, bv, bnv;
  initial begin
    b_h = new();
    a_h = b_h;
    av = a_h.v(); anv = a_h.nv();
    bv = b_h.v(); bnv = b_h.nv();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(
        u(&sim, "av"),
        2,
        "virtual: runtime type wins through a base handle"
    );
    assert_eq!(u(&sim, "anv"), 10, "non-virtual: declared type wins");
    assert_eq!(u(&sim, "bv"), 2, "derived handle: derived override");
    assert_eq!(u(&sim, "bnv"), 20, "derived handle: derived non-virtual");
}

/// A method the derived class does not override, and `super.` calls, are
/// unaffected by static binding.
#[test]
fn unoverridden_methods_and_super_are_unchanged() {
    let src = r#"
module top;
  class A;
    function int base_only(); return 7; endfunction
    virtual function int v(); return 1; endfunction
  endclass
  class B extends A;
    virtual function int v(); return super.v() + 1; endfunction
  endclass
  A a_h;
  int bo, sv;
  initial begin
    B b;
    b = new();
    a_h = b;
    bo = a_h.base_only();
    sv = a_h.v();
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "bo"), 7, "un-overridden method");
    assert_eq!(u(&sim, "sv"), 2, "super chain through the virtual override");
}

/// §9.7: the state constants exist and status() agrees with them.
#[test]
fn process_state_constants_resolve() {
    let src = r#"
`timescale 1ns/1ns
module top;
  int waiting_ok, finished_ok, killed_ok, consts;
  initial begin
    process p1, p2;
    fork
      begin p1 = process::self(); #100; end
      begin p2 = process::self(); #1;  end
    join_none
    #2;
    waiting_ok  = (p1.status() == process::WAITING);
    finished_ok = (p2.status() == process::FINISHED);
    p1.kill();
    #1;
    killed_ok = (p1.status() == process::KILLED);
    consts = (process::FINISHED == 0) && (process::RUNNING == 1)
          && (process::WAITING == 2) && (process::SUSPENDED == 3)
          && (process::KILLED == 4);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 300).expect("simulate failed");
    assert_eq!(
        u(&sim, "consts"),
        1,
        "all five constants have their §9.7 values"
    );
    assert_eq!(u(&sim, "waiting_ok"), 1, "a #-blocked process is WAITING");
    assert_eq!(u(&sim, "finished_ok"), 1, "a completed process is FINISHED");
    assert_eq!(u(&sim, "killed_ok"), 1, "a killed process is KILLED");
}

/// §21.3.4.2: a partial-format $fscanf leaves the rest of the line readable.
#[test]
fn fscanf_advances_only_past_what_it_matched() {
    // Keep the fixture out of the repo working tree (see
    // audit_round45_finds: a CWD-relative $fopen kept re-tracking its file).
    let tmp = std::env::temp_dir().join(format!("xezim_fscanf_pos_{}.txt", std::process::id()));
    let tmp_path = tmp.to_string_lossy().replace('\\', "/");
    let src = format!(
        r#"
module top;
  int fd, r, eof_mid, a, b;
  string w1, w2;
  initial begin
    fd = $fopen("{tmp_path}", "w");
    $fdisplay(fd, "alpha beta");
    $fdisplay(fd, "42 43");
    $fclose(fd);
    fd = $fopen("{tmp_path}", "r");
    r = $fscanf(fd, "%s", w1);          // reads "alpha" only
    eof_mid = $feof(fd);                // data remains: 0
    r = $fscanf(fd, "%s", w2);          // "beta"
    r = $fscanf(fd, "%d %d", a, b);     // next line
    $fclose(fd);
    $display("W1=%s W2=%s A=%0d B=%0d", w1, w2, a, b);
  end
endmodule
"#
    );
    let sim = simulate(&src, 20).expect("simulate failed");
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(u(&sim, "eof_mid"), 0, "$feof false with the line half-read");
    let o = outs(&sim).join("\n");
    assert!(o.contains("W1=alpha W2=beta A=42 B=43"), "got: {o}");
}
