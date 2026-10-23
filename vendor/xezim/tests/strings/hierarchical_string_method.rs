//! IEEE 1800-2023 §23.6 (hierarchical names) + §6.16 (string methods):
//! a string field reached through a hierarchical reference
//! (`u_if.data.len()`) must dispatch the §6.16 string builtins on the
//! FULLY-QUALIFIED receiver (`u_if.data`), not on just the first path
//! segment (`u_if`).
//!
//! The parser flattens `a.b.m()` into `Ident([a, b, m])`. The generic
//! hierarchical-call tail derived the receiver as `path[0]` (`a`) instead of
//! the full `path[0..len-1]` (`a.b`), so every §6.16 method on a 3+-segment
//! hierarchical string returned 0/empty: `u_if.data.len()` → 0,
//! `u_if.data.getc(1)` → 0, `u_if.data.substr(1,3)` → "".
//!
//! The fix resolves the receiver's fully-qualified name and dispatches the
//! builtin by it, while guarding against a USER class method of the same name
//! (a class defining its own `len()` must still call the real method, not the
//! string builtin). All values below match the reference simulator exactly.

#[test]
fn hierarchical_string_len() {
    let src = r#"
interface my_if;
    string data = "hello_world_this_string_is_valid";
endinterface
module consumer;
    my_if u_if();
    initial begin
        if (u_if.data.len() != 32)
            $display("TAG_FAIL len=%0d", u_if.data.len());
        else
            $display("TAG_PASS");
    end
endmodule
module top;
    consumer u_consumer();
endmodule
"#;
    let sim = xezim::simulate(src, 50).expect("simulate");
    assert!(
        sim.output.iter().any(|l| l.message == "TAG_PASS"),
        "hierarchical u_if.data.len() must return the string length.\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn hierarchical_string_getc_substr() {
    let src = r#"
interface my_if;
    string data = "Hello";
endinterface
module consumer;
    my_if u_if();
    initial begin
        // getc(1) == 'e' (101); substr(1,3) == "ell"
        if (u_if.data.getc(1) != 101)
            $display("TAG_FAIL getc=%0d", u_if.data.getc(1));
        else if (u_if.data.substr(1, 3) != "ell")
            $display("TAG_FAIL substr=%0s", u_if.data.substr(1, 3));
        else
            $display("TAG_PASS");
    end
endmodule
module top;
    consumer u_consumer();
endmodule
"#;
    let sim = xezim::simulate(src, 50).expect("simulate");
    assert!(
        sim.output.iter().any(|l| l.message == "TAG_PASS"),
        "hierarchical getc/substr must read the string field.\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn hierarchical_len_does_not_shadow_user_method() {
    // A class defining its OWN `len()` must dispatch to it, not the string
    // builtin. This guards the user-method precedence in the fix.
    let src = r#"
module top;
    class container;
        int cnt[$];
        function void add(); cnt.push_back(0); endfunction
        function int len(); return cnt.size(); endfunction
    endclass
    initial begin
        container c = new();
        c.add(); c.add();
        if (c.len() != 2)
            $display("TAG_FAIL c.len=%0d", c.len());
        else
            $display("TAG_PASS");
    end
endmodule
"#;
    let sim = xezim::simulate(src, 50).expect("simulate");
    assert!(
        sim.output.iter().any(|l| l.message == "TAG_PASS"),
        "a user class method named len() must win over the string builtin.\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
