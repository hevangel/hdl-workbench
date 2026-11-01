//! Reading or writing a class member through a HIERARCHICAL path
//! (`tb.u_leaf.obj.field`) must walk the object, not the flat signal table.
//!
//! The member-access branch of `eval_hierarchical_ident` tried
//! `get_signal_value_by_name(<full path>)` before resolving the handle, so the
//! path was never validated against an object: a same-named flat signal in the
//! observing module won (`tb.u_leaf.obj.field` returned `tb.field`), and with a
//! member name that collides with nothing the read returned `""`.  Writes were
//! mis-routed the same way, so `tb.u_leaf.obj.n = 7` updated the shadow signal
//! and left the object at 42.  `hier_parent_handle` now resolves the parent
//! (`tb.u_leaf.obj`) to a live heap object — but only when the name statically
//! denotes a class object, which is what keeps a struct, vector, or net whose
//! value happens to be a small integer from being read back as a heap index.
//! Verified against the reference simulator.

fn lines(src: &str, top: &str) -> Vec<String> {
    xezim::simulate_multi(
        &[src.to_string()],
        1000,
        Some(top),
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &[],
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
    )
    .expect("sim")
    .output
    .iter()
    .map(|o| o.message.clone())
    .collect()
}

const SRC: &str = r#"
class n4_c;
  string field     = "LIT";
  string only_here = "UNIQ";
  int    n         = 42;
  function new();
  endfunction
  function string f();
    return "FN";
  endfunction
endclass

typedef struct { int field; int n; } n4_p;

module n4_leaf;
  n4_c obj = new();
endmodule

module n4_leaf2;
  n4_p   p;
  int    plain = 5;
  logic [7:0] vec;
  initial begin
    p.field = 11;
    p.n  = 3;
    vec  = 8'h3c;
  end
endmodule

module tb;
  n4_leaf  u_leaf();
  n4_leaf2 u_leaf2();
  n4_c     tmp;
  // Deliberately collides with the class member name: a tool that collapses
  // `tb.u_leaf.obj.field` to the bare `field` returns THIS value.
  string   field = "SHADOW";
  string   only_here = "SHADOW2";
  initial begin
    #1;
    tmp = tb.u_leaf.obj;
    $display("VIA_TMP=%s",   tmp.field);
    $display("METHOD=%s",    tb.u_leaf.obj.f());
    $display("MEMBER=%s",    tb.u_leaf.obj.field);
    $display("UNIQ=%s",      tb.u_leaf.obj.only_here);
    $display("MEMBER_N=%0d", tb.u_leaf.obj.n);
    tb.u_leaf.obj.n = 7;
    $display("WR_PATH=%0d",  tb.u_leaf.obj.n);
    $display("WR_OBJ=%0d",   tmp.n);
    $display("WR_FIELD=%s",  tb.u_leaf.obj.field);
    tb.u_leaf.obj.field = "SET";
    $display("WR_FIELD2=%s", tmp.field);
    // non-class parents must keep the ordinary signal path
    $display("STRUCT_FIELD=%0d", tb.u_leaf2.p.field);
    $display("STRUCT_N=%0d",     tb.u_leaf2.p.n);
    $display("PLAIN=%0d",    tb.u_leaf2.plain);
    $display("VEC=%0d",      tb.u_leaf2.vec);
    $finish;
  end
endmodule
"#;

/// A member read through a hierarchical path comes from the object even when a
/// same-named signal exists in the observing module.
#[test]
fn hier_read_of_class_member_walks_the_object() {
    let out = lines(SRC, "tb").join("\n");
    assert!(out.contains("VIA_TMP=LIT"), "{out}");
    assert!(out.contains("METHOD=FN"), "{out}");
    assert!(out.contains("MEMBER=LIT"), "leaf-name collapse: {out}");
    assert!(out.contains("UNIQ=UNIQ"), "leaf-name collapse: {out}");
    assert!(out.contains("MEMBER_N=42"), "{out}");
}

/// A write through the path lands in the object — the handle that already holds
/// it sees the new value too.
#[test]
fn hier_write_of_class_member_lands_in_the_object() {
    let out = lines(SRC, "tb").join("\n");
    assert!(out.contains("WR_PATH=7"), "{out}");
    assert!(
        out.contains("WR_OBJ=7"),
        "write hit the shadow signal: {out}"
    );
    assert!(out.contains("WR_FIELD=LIT"), "{out}");
    assert!(out.contains("WR_FIELD2=SET"), "{out}");
}

/// The handle guard is a static class-object check, so struct members, plain
/// variables, and vectors under an instance still resolve through the signal
/// table instead of being read back as heap indices — including `p.field`,
/// whose name collides with the class member above.
#[test]
fn non_class_parents_stay_on_the_signal_path() {
    let out = lines(SRC, "tb").join("\n");
    assert!(out.contains("STRUCT_FIELD=11"), "{out}");
    assert!(out.contains("STRUCT_N=3"), "{out}");
    assert!(out.contains("PLAIN=5"), "{out}");
    assert!(out.contains("VEC=60"), "{out}");
}

#[test]
fn basic_relative() {
    let got = lines(r#"class C; int n=42; endclass module leaf; C obj=new; endmodule module top;leaf u();int n=99;initial begin #1;$display("T|%0d",u.obj.n);u.obj.n=7;$display("T|%0d",u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7"]);
}

#[test]
fn nested_object() {
    let got = lines(r#"class B;int n=42;endclass class C;B child=new;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;$display("T|%0d",top.u.obj.child.n);top.u.obj.child.n=7;$display("T|%0d",top.u.obj.child.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7"]);
}

#[test]
fn nonclass_collision() {
    let got = lines(r#"class C;int n=42;endclass typedef struct packed {int n;} P;module leaf;P p=1;endmodule module top;leaf u();C p=new;initial begin #1;$display("T|%0d",top.u.p.n);top.u.p.n=7;$display("T|%0d %0d",top.u.p.n,p.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|1", "T|7 42"]);
}

#[test]
fn nonclass_struct_property() {
    let got = lines(r#"class C;int n=42;endclass typedef struct packed {int n;} P;class D;P bits=1;endclass module leaf;D obj=new;endmodule module top;leaf u();initial begin #1;$display("T|%0d",top.u.obj.bits.n);top.u.obj.bits.n=7;$display("T|%0d",top.u.obj.bits.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|1", "T|7"]);
}

#[test]
fn generated_handle() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;for(genvar i=0;i<2;i++)begin:g leaf u();end initial begin #1;$display("T|%0d",top.g[0].u.obj.n);top.g[0].u.obj.n=7;$display("T|%0d",top.g[0].u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7"]);
}

#[test]
fn array_handle() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj[2];initial foreach(obj[i])obj[i]=new;endmodule module top;leaf u();initial begin #1;$display("T|%0d",top.u.obj[0].n);top.u.obj[0].n=7;$display("T|%0d",top.u.obj[0].n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7"]);
}

#[test]
fn member_slice() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;$display("T|%0d",top.u.obj.n[3:0]);top.u.obj.n[3:0]=7;$display("T|%0d",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|10", "T|39"]);
}

#[test]
fn object_shadow() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();C obj=new;initial begin #1;obj.n=99;$display("T|%0d",top.u.obj.n);top.u.obj.n=7;$display("T|%0d %0d",top.u.obj.n,obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7 99"]);
}

#[test]
fn local_object_shadow() {
    let got = lines(r#"class C;int n=42;endclass module top;C obj=new;initial begin C obj;obj=new;obj.n=7;$display("T|%0d %0d",obj.n,top.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|7 42"]);
}

#[test]
fn formal_object_shadow() {
    let got = lines(r#"class C;int n=42;endclass module top;C obj=new;C other=new;function automatic int f(C obj);obj.n=7;return obj.n;endfunction initial begin $display("T|%0d",f(other));$display("T|%0d %0d",obj.n,other.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|7", "T|42 7"]);
}

#[test]
fn class_property_shadow() {
    let got = lines(r#"class C;int n=42;endclass class D;C obj=new;function int f();obj.n=7;return obj.n;endfunction endclass module top;C obj=new;D d=new;initial begin $display("T|%0d",d.f());$display("T|%0d %0d",obj.n,d.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|7", "T|42 7"]);
}

#[test]
fn nested_alias() {
    let got = lines(r#"class B;int n=42;endclass typedef B A;class C;A child=new;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;$display("T|%0d",top.u.obj.child.n);top.u.obj.child.n=7;$display("T|%0d",top.u.obj.child.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7"]);
}

#[test]
fn nested_inherited() {
    let got = lines(r#"class B;int n=42;endclass class P;B child=new;endclass class C extends P;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;$display("T|%0d",top.u.obj.child.n);top.u.obj.child.n=7;$display("T|%0d",top.u.obj.child.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7"]);
}

#[test]
fn nested_type_parameter() {
    let got = lines(r#"class B;int n=42;endclass class C#(type T=B);T child=new;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;$display("T|%0d",top.u.obj.child.n);top.u.obj.child.n=7;$display("T|%0d",top.u.obj.child.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42", "T|7"]);
}

#[test]
fn member_bit() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;top.u.obj.n[0]=1;$display("T|%0d",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|43"]);
}

#[test]
fn member_indexed_up() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;top.u.obj.n[0+:4]=7;$display("T|%0d",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|39"]);
}

#[test]
fn member_indexed_down() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;top.u.obj.n[3-:4]=7;$display("T|%0d",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|39"]);
}

#[test]
fn member_unknown_select() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();logic [3:0] idx='x;initial begin #1;top.u.obj.n[idx+:4]=7;$display("T|%0d",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|42"]);
}

#[test]
fn member_bound_once() {
    let got = lines(r#"class C;int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();int calls=0;function int base();calls++;return 0;endfunction initial begin #1;top.u.obj.n[base()+:4]=7;$display("T|%0d %0d",top.u.obj.n,calls);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|39 1"]);
}

#[test]
fn member_ascending() {
    let got = lines(r#"class C;logic[0:7] n=8'ha5;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;top.u.obj.n[0:3]=4'h3;$display("T|%h",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|35"]);
}

#[test]
fn member_offset() {
    let got = lines(r#"class C;logic[7:4] n=4'ha;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;top.u.obj.n[5:4]=3;$display("T|%h",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|b"]);
}

#[test]
fn member_partial_oob() {
    let got = lines(r#"class C;logic[3:0] n=4'ha;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;top.u.obj.n[4:2]=7;$display("T|%h",top.u.obj.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|e"]);
}

#[test]
fn hier_static_slice() {
    let got = lines(r#"class C;static int n=42;endclass module leaf;C obj=new;endmodule module top;leaf u();initial begin #1;top.u.obj.n[3:0]=7;$display("T|%0d %0d",top.u.obj.n,C::n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|39 39"]);
}

#[test]
fn nested_static_handle() {
    let got = lines(r#"class B;int n=42;endclass class C;static B child=new;endclass module leaf;C obj=new;endmodule module top;leaf u();B other=new;initial begin #1;other.n=9;C::child=other;$display("T|%0d",top.u.obj.child.n);top.u.obj.child.n=7;$display("T|%0d %0d",top.u.obj.child.n,other.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|9", "T|7 7"]);
}

#[test]
fn whole_struct() {
    let got=lines(r#"typedef struct {int n;} P;class C;P data;function new();data.n=42;endfunction endclass module leaf;C obj=new;endmodule module top;leaf u();P t;initial begin #1;top.u.obj.data.n=7;t=top.u.obj.data;$display("T|%0d %0d",t.n,top.u.obj.data.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|7 7"]);
}

#[test]
fn whole_static_struct() {
    let got=lines(r#"typedef struct {int n;} P;class C;static P data;endclass module leaf;C obj=new;endmodule module top;leaf u();P t;initial begin #1;top.u.obj.data.n=7;t=top.u.obj.data;$display("T|%0d %0d",t.n,top.u.obj.data.n);end endmodule"#, "top").into_iter().filter(|line| line.starts_with("T|")).collect::<Vec<_>>();
    assert_eq!(got, ["T|7 7"]);
}
