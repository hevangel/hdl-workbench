//! Regression tests for parameterized-class static dispatch + constructor
//! resolution through a class-scoped typedef alias — the three generic
//! (LRM-only) fixes that resolved a re-entrant factory-construction loop:
//!
//!   1. §8.25 — `T obj = new(...)` in a parameterized class method resolves
//!      the type parameter `T` through the active specialization, so the
//!      constructor builds the bound class instead of a broken literal-"T"
//!      instance.
//!   2. §6.18/§8.23 — `Class::typedef_alias::static_method(args)` parses as a
//!      flat 3-segment hierarchical Ident; the alias (ANY name, not a
//!      library-specific spelling) must be resolved to its target
//!      class/specialization before dispatching the static method.
//!   3. §23.7 — a method-local shadows a same-named module-scope signal, so
//!      `T obj; obj = new(name)` inside a parameterized method constructs the
//!      declared type, not the unrelated module-level object.
//!
//! Each case uses custom identifiers (`proxy` / `build`) so the test proves
//! the mechanism is generic and not tied to a particular library's names.

use std::process::Command;

fn run(src: &str, tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_ptc_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{tag}.sv"));
    std::fs::write(&path, src).unwrap();
    let bin = env!("CARGO_BIN_EXE_xezim");
    let out = Command::new(bin)
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(path.to_str().unwrap())
        .output()
        .expect("failed to run xezim");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// A parameterized registry whose static `build` constructs `T` through a
// class-scoped typedef alias — object (1-arg) and component-style (2-arg)
// constructors, with and without members.
const SRC: &str = r#"
class registry #(type T=int, string N="");
    static function T build(string name);
        T obj = new(name);          // combined form — §8.25 type-param
        return obj;
    endfunction
endclass

class base_c;
    string full_name;
    function new(string n, base_c parent=null);
        full_name = n;
    endfunction
endclass
class base_o;
    string full_name;
    function new(string n="");
        full_name = n;
    endfunction
endclass

class comp_m extends base_c;
    int payload;
    typedef registry#(comp_m, "comp_m") proxy;
    function new(string n, base_c parent=null);
        super.new(n, parent);
        payload = 11;
    endfunction
endclass

class comp_n extends base_c;
    typedef registry#(comp_n, "comp_n") proxy;
    function new(string n, base_c parent=null);
        super.new(n, parent);
    endfunction
endclass

class obj_m extends base_o;
    int payload;
    typedef registry#(obj_m, "obj_m") proxy;
    function new(string n="");
        super.new(n);
        payload = 22;
    endfunction
endclass

class obj_n extends base_o;
    typedef registry#(obj_n, "obj_n") proxy;
    function new(string n="");
        super.new(n);
    endfunction
endclass

module top;
    integer pass = 0;
    integer fail = 0;
    task chk(input string l, input bit ok);
        if (ok) begin
            $display("PASS: %s", l);
            pass = pass + 1;
        end else begin
            $display("FAIL: %s", l);
            fail = fail + 1;
        end
    endtask
    initial begin
        comp_m c1; comp_n c2; obj_m o1; obj_n o2;
        c1 = comp_m::proxy::build("t1");
        chk("comp with member", c1 != null && c1.payload == 11 && c1.full_name == "t1");
        c2 = comp_n::proxy::build("t2");
        chk("comp no member", c2 != null && c2.full_name == "t2");
        o1 = obj_m::proxy::build("t3");
        chk("obj with member", o1 != null && o1.payload == 22 && o1.full_name == "t3");
        o2 = obj_n::proxy::build("t4");
        chk("obj no member", o2 != null && o2.full_name == "t4");
        $display("RESULT: %0d pass %0d fail", pass, fail);
    end
endmodule
"#;

#[test]
fn param_typedef_alias_static_dispatch() {
    let out = run(SRC, "ptc_main");
    assert!(
        out.contains("PASS: comp with member")
            && out.contains("PASS: comp no member")
            && out.contains("PASS: obj with member")
            && out.contains("PASS: obj no member"),
        "expected all four checks to pass; got:\n{out}"
    );
    assert!(
        out.contains("RESULT: 4 pass 0 fail"),
        "expected 4 pass / 0 fail; got:\n{out}"
    );
}

// §23.7: a module-scope `C obj;` must NOT shadow a method-local `T obj;`
// inside a parameterized class method. Before the fix, `obj = new(name)`
// constructed the module-level `C` (re-entering construction) instead of `T`.
const SHADOW_SRC: &str = r#"
class maker #(type T=int);
    static function T make(string name);
        T obj;                        // separate decl + assign form
        obj = new(name);
        return obj;
    endfunction
endclass

class baseo;
    string nm;
    function new(string n="");
        nm = n;
    endfunction
endclass

class target extends baseo;
    int mark;
    function new(string n="");
        super.new(n);
        mark = 7;
    endfunction
endclass

module top;
    target obj;                       // module-scope `obj` of a DIFFERENT class
    integer pass = 0; integer fail = 0;
    initial begin
        target t;
        obj = new("module_obj");
        t = maker#(target)::make("local_obj");
        if (t != null && t.mark == 7 && t.nm == "local_obj") begin
            $display("PASS: local shadows module");
            pass = pass + 1;
        end else begin
            $display("FAIL: local shadows module (mark=%0d nm=%0s)", t.mark, t.nm);
            fail = fail + 1;
        end
        if (obj.nm == "module_obj") begin
            $display("PASS: module obj intact");
            pass = pass + 1;
        end else begin
            $display("FAIL: module obj intact (nm=%0s)", obj.nm);
            fail = fail + 1;
        end
        $display("RESULT: %0d pass %0d fail", pass, fail);
    end
endmodule
"#;

#[test]
fn method_local_shadows_module_signal() {
    let out = run(SHADOW_SRC, "ptc_shadow");
    assert!(
        out.contains("PASS: local shadows module") && out.contains("PASS: module obj intact"),
        "expected both shadow checks to pass; got:\n{out}"
    );
    assert!(
        out.contains("RESULT: 2 pass 0 fail"),
        "expected 2 pass / 0 fail; got:\n{out}"
    );
}
