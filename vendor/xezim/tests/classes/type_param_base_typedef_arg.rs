//! §6.18 / §8.25: a typedef names the type it is declared as, so a class
//! extending `first_layer #(holder_alias_t)`, with `typedef typed_holder #(options_leaf) holder_alias_t;` and
//! `class first_layer #(type BASE) extends BASE;`, has the ancestor `typed_holder
//! #(options_leaf)` exactly as one extending `first_layer #(typed_holder #(options_leaf))` does. The
//! entry linked for the derived class read `typed_holder`'s arguments back from
//! the binding of `BASE`, which the bare typedef name did not spell, so
//! `typed_holder` ran with its defaults: a `CFG` object built in its constructor
//! was an `options_root` rather than the selected specialization.

use xezim::simulate;

const SRC: &str = r#"
class options_root; virtual function string name(); return "options_root"; endfunction endclass
class options_leaf extends options_root; virtual function string name(); return "options_leaf"; endfunction endclass
class state_root; virtual function string name(); return "state_root"; endfunction endclass
class state_leaf extends state_root; virtual function string name(); return "state_leaf"; endfunction endclass
class proxy_root; endclass
class proxy_base #(type CFG = options_root) extends proxy_root; endclass
class proxy #(type CFG = options_root) extends proxy_base #(CFG); endclass
class typed_holder #(type CFG = options_root, type ST = state_root, type PCFG = CFG);
  options_root c;
  state_root s;
  proxy_root px;
  function new();
    CFG t = new();
    ST u = new();
    proxy #(PCFG) p = new();
    c = t;
    s = u;
    px = p;
  endfunction
endclass
typedef typed_holder #(options_leaf) holder_alias_t;
typedef typed_holder #(.ST(state_leaf), .CFG(options_leaf)) named_holder_t;
class first_layer #(type BASE = int) extends BASE; endclass
class second_layer #(type BASE = int) extends first_layer #(BASE); endclass
class alias_path extends first_layer #(holder_alias_t); endclass
class explicit_path    extends first_layer #(typed_holder #(options_leaf)); endclass
class named_path   extends first_layer #(.BASE(named_holder_t)); endclass
class double_path     extends second_layer #(holder_alias_t); endclass
class derived_path  extends alias_path; endclass
module top;
  initial begin
    alias_path  a = new();
    explicit_path     b = new();
    named_path    n = new();
    double_path      w = new();
    derived_path   d = new();
    first_layer #(holder_alias_t) c = new();
    proxy_base #(options_leaf) pb;
    $display("typedef=%s spec=%s named=%s,%s two=%s deeper=%s itself=%s proxy_cast=%0d",
             a.c.name(), b.c.name(), n.c.name(), n.s.name(), w.c.name(), d.c.name(), c.c.name(),
             $cast(pb, a.px));
  end
endmodule
"#;

#[test]
fn typedef_specialization_through_type_param_base_keeps_its_arguments() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        [
            "typedef=options_leaf spec=options_leaf named=options_leaf,state_leaf two=options_leaf deeper=options_leaf itself=options_leaf proxy_cast=1"
        ],
        "{out:?}"
    );
}
