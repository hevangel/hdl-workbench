//! IEEE 1800 §25.9–25.10 and §23.11: reference-checked sibling regressions.
use xezim::simulate_multi;
fn check(source: &str, expected: &[&str]) {
    check_sources(&[source], expected);
}
fn check_sources(sources: &[&str], expected: &[&str]) {
    let sim = simulate_multi(
        &sources.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        10,
        Some("top"),
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
    .expect("simulate sibling regression");
    let mut lines: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end())
        .filter(|s| s.starts_with("T|"))
        .collect();
    let mut expected = expected.to_vec();
    lines.sort();
    expected.sort();
    assert_eq!(lines, expected);
}

#[test]
fn scope_lexical_override() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module binder#(parameter SEL=1);bind top.bank[SEL].m mon b(.s(data));endmodule
module top;for(genvar g=0;g<3;g++)begin:bank mem#(g+5) m();end binder#(2) cfg(); initial #3 $finish;endmodule
"####,
        &["T|top.bank[2].m.b=7"],
    );
}

#[test]
fn scope_localparam_shadow() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module binder;localparam SEL=2;bind top.bank[SEL].m mon b(.s(data));endmodule
module top;localparam SEL=0;for(genvar g=0;g<3;g++)begin:bank mem#(g+5) m();end binder cfg(); initial #3 $finish;endmodule
"####,
        &["T|top.bank[2].m.b=7"],
    );
}

#[test]
fn scope_relative_child() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host#(parameter SEL=1);for(genvar g=0;g<3;g++)begin:bank mem#(g+5) m();end bind bank[SEL].m mon b(.s(data));endmodule
module top;host#(2) h(); initial #3 $finish;endmodule
"####,
        &["T|top.h.bank[2].m.b=7"],
    );
}

#[test]
fn scope_sibling_parameter() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module selector;localparam SEL=2;endmodule module host;for(genvar g=0;g<3;g++)begin:bank mem#(g+5) m();end endmodule
module top;host h();selector cfg();bind top.h.bank[top.cfg.SEL].m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.h.bank[2].m.b=7"],
    );
}

#[test]
fn colon_list() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem a(),b(),decoy();bind mem:a,b mon watch(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.a.watch=7", "T|top.b.watch=7"],
    );
}

#[test]
fn colon_child() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host;mem a(),decoy();bind mem:a mon b(.s(data));endmodule
module top;host h(); initial #3 $finish;endmodule
"####,
        &["T|top.h.a.b=7"],
    );
}

#[test]
fn colon_generated() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;for(genvar g=0;g<2;g++)begin:bank mem a();end bind mem:bank[1].a mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.bank[1].a.b=7"],
    );
}

#[test]
fn colon_absolute_list() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem a(),b(),decoy();bind mem:top.a,top.b mon watch(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.a.watch=7", "T|top.b.watch=7"],
    );
}

#[test]
fn colon_array() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem a[1:0]();bind mem:a[0] mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.a[0].b=7"],
    );
}

#[test]
fn params_ordered() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host#(parameter N=1);for(genvar g=0;g<N;g++)begin:bank mem m();end endmodule
module top;host#(3) h();bind top.h.bank[2].m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.h.bank[2].m.b=7"],
    );
}

#[test]
fn params_defparam_absolute() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host#(parameter N=1);for(genvar g=0;g<N;g++)begin:bank mem m();end endmodule
module top;host h();defparam top.h.N=3;bind top.h.bank[2].m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.h.bank[2].m.b=7"],
    );
}

#[test]
fn params_defparam_nested() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host#(parameter N=1);for(genvar g=0;g<N;g++)begin:bank mem m();end endmodule module wrap;host h();endmodule
module top;wrap w();defparam w.h.N=3;bind top.w.h.bank[2].m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.w.h.bank[2].m.b=7"],
    );
}

#[test]
fn params_type_named() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host#(parameter type T=bit);for(genvar g=0;g<$bits(T);g++)begin:bank mem m();end endmodule
module top;host#(.T(logic[3:0])) h();bind top.h.bank[3].m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.h.bank[3].m.b=7"],
    );
}

#[test]
fn params_forwarded() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host#(parameter N=1);for(genvar g=0;g<N;g++)begin:bank mem m();end endmodule module wrap#(parameter N=1);host#(N) h();endmodule
module top;wrap#(3) w();bind top.w.h.bank[2].m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.w.h.bank[2].m.b=7"],
    );
}

#[test]
fn relative_two_instances() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host;mem m();bind mem:m mon watch(.s(data));endmodule
module top;host a(),b(); initial #3 $finish;endmodule
"####,
        &["T|top.a.m.watch=7", "T|top.b.m.watch=7"],
    );
}

#[test]
fn relative_generate_scope() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;if(1)begin:active mem m();bind top.active.m mon b(.s(data));end initial #3 $finish;endmodule
"####,
        &["T|top.active.m.b=7"],
    );
}

#[test]
fn relative_ancestor() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module binder;bind top.m mon b(.s(data));endmodule
module top;mem m();binder cfg(); initial #3 $finish;endmodule
"####,
        &["T|top.m.b=7"],
    );
}

#[test]
fn array_ascending() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem m[0:2]();bind top.m[2] mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.m[2].b=7"],
    );
}

#[test]
fn array_negative() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem m[-1:-2]();bind top.m[-2] mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.m[-2].b=7"],
    );
}

#[test]
fn array_nested() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host;mem m[0:1]();endmodule
module top;host h[1:0]();bind top.h[0].m[1] mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.h[0].m[1].b=7"],
    );
}

#[test]
fn array_generated() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;for(genvar g=0;g<2;g++)begin:bank mem m[1:0]();end bind top.bank[1].m[0] mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.bank[1].m[0].b=7"],
    );
}

#[test]
fn root_direct() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem m();bind $root.top.m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.m.b=7"],
    );
}

#[test]
fn root_colon() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem m();bind mem:$root.top.m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.m.b=7"],
    );
}

#[test]
fn root_selector() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;localparam SEL=2;for(genvar g=0;g<3;g++)begin:bank mem#(g+5) m();end bind $root.top.bank[SEL].m mon b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.bank[2].m.b=7"],
    );
}

#[test]
fn root_from_child() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module binder;localparam SEL=1;bind $root.top.bank[SEL].m mon b(.s(data));endmodule
module top;for(genvar g=0;g<3;g++)begin:bank mem#(g+5) m();end binder cfg(); initial #3 $finish;endmodule
"####,
        &["T|top.bank[1].m.b=6"],
    );
}

#[test]
fn interface_nested() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
interface node;logic[3:0]data=7;endinterface interface watch(input logic[3:0]s);initial #1 $display("T|%m=%0d",s);endinterface interface host;node n();endinterface
module top;host h();bind top.h.n watch b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.h.n.b=7"],
    );
}

#[test]
fn interface_array() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
interface node;logic[3:0]data=7;endinterface interface watch(input logic[3:0]s);initial #1 $display("T|%m=%0d",s);endinterface
module top;node n[1:0]();bind top.n[0] watch b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.n[0].b=7"],
    );
}

#[test]
fn interface_relative() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
interface node;logic[3:0]data=7;endinterface interface watch(input logic[3:0]s);initial #1 $display("T|%m=%0d",s);endinterface interface host;node n();bind node:n watch b(.s(data));endinterface
module top;host h(); initial #3 $finish;endmodule
"####,
        &["T|top.h.n.b=7"],
    );
}

#[test]
fn interface_repeat() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
interface node;logic[3:0]data=7;endinterface interface watch(input logic[3:0]s);initial #1 $display("T|%m=%0d",s);endinterface
module top;node n();bind top.n watch a(.s(data));bind top.n watch b(.s(data)); initial #3 $finish;endmodule
"####,
        &["T|top.n.b=7", "T|top.n.a=7"],
    );
}

#[test]
fn generate_inactive() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem m();if(0)begin:inactive bind mem mon b(.s(data));end initial #3 $finish;endmodule
"####,
        &[],
    );
}

#[test]
fn generate_else() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem m();if(0)begin:inactive bind mem mon a(.s(data));end else begin:active bind mem mon b(.s(data));end initial #3 $finish;endmodule
"####,
        &["T|top.m.b=7"],
    );
}

#[test]
fn generate_case() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;mem m();case(2)1:begin bind mem mon a(.s(data));end 2:begin bind mem mon b(.s(data));end endcase initial #3 $finish;endmodule
"####,
        &["T|top.m.b=7"],
    );
}

#[test]
fn generate_loop() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;for(genvar g=0;g<2;g++)begin:bank mem m();bind top.bank[g].m mon b(.s(data));end initial #3 $finish;endmodule
"####,
        &["T|top.bank[0].m.b=7", "T|top.bank[1].m.b=7"],
    );
}

#[test]
fn generate_relative() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule

module top;if(1)begin:bank mem m();bind mem:m mon b(.s(data));end initial #3 $finish;endmodule
"####,
        &["T|top.bank.m.b=7"],
    );
}

#[test]
fn repeat_three() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module up;initial #1 $display("T|%m=%0d",mem.data);endmodule
module top;mem m();bind top.m up a();bind top.m up b();bind top.m up c(); initial #3 $finish;endmodule
"####,
        &["T|top.m.c=7", "T|top.m.b=7", "T|top.m.a=7"],
    );
}

#[test]
fn repeat_type_and_instance() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module up;initial #1 $display("T|%m=%0d",mem.data);endmodule
module top;mem m(),decoy();bind mem up a();bind top.m up b(); initial #3 $finish;endmodule
"####,
        &["T|top.m.a=7", "T|top.m.b=7", "T|top.decoy.a=7"],
    );
}

#[test]
fn repeat_generated() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module up;initial #1 $display("T|%m=%0d",mem.data);endmodule
module top;for(genvar g=0;g<2;g++)begin:bank mem m();end bind top.bank[1].m up a();bind top.bank[1].m up b(); initial #3 $finish;endmodule
"####,
        &["T|top.bank[1].m.b=7", "T|top.bank[1].m.a=7"],
    );
}

#[test]
fn repeat_nested() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module up;initial #1 $display("T|%m=%0d",mem.data);endmodule module host;mem m();endmodule
module top;host h();bind top.h.m up a();bind top.h.m up b(); initial #3 $finish;endmodule
"####,
        &["T|top.h.m.b=7", "T|top.h.m.a=7"],
    );
}

#[test]
fn repeat_declared_suffix() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module mem__bind0;logic[3:0]data=9;endmodule module up;initial #1 $display("T|%m=%0d",mem__bind0.data);endmodule
module top;mem__bind0 m();bind top.m up a();bind top.m up b(); initial #3 $finish;endmodule
"####,
        &["T|top.m.b=9", "T|top.m.a=9"],
    );
}

#[test]
fn relative_nested() {
    check(
        r####"module mem #(parameter V=7);logic[3:0] data=V;endmodule
module mon(input logic[3:0] s);initial #1 $display("T|%m=%0d",s);endmodule
module host;if(1)begin:bank mem m();end bind bank.m mon watch(.s(data));endmodule module wrap;host h();endmodule
module top;wrap w(); initial #3 $finish;endmodule
"####,
        &["T|top.w.h.bank.m.watch=7"],
    );
}

#[test]
fn scope_unit_shadow() {
    check(
        r####"module mem;logic[3:0]data=7;endmodule module mon(input logic[3:0]s);initial #1 $display("T|%m=%0d",s);endmodule localparam SEL=1;module top;localparam SEL=0;for(genvar g=0;g<2;g++)begin:bank mem m();end initial #2 $finish;endmodule bind top.bank[SEL].m mon b(.s(data));"####,
        &["T|top.bank[1].m.b=7"],
    );
}

#[test]
fn scope_generated_shadow() {
    check(
        r####"module mem;logic[3:0]data=7;endmodule module mon(input logic[3:0]s);initial #1 $display("T|%m=%0d",s);endmodule module top;localparam SEL=0;for(genvar g=0;g<2;g++)begin:bank mem m();end if(1)begin:cfg localparam SEL=1;bind top.bank[SEL].m mon b(.s(data));end initial #2 $finish;endmodule"####,
        &["T|top.bank[1].m.b=7"],
    );
}

#[test]
fn scope_binder_instances() {
    check(
        r####"module mem;logic[3:0]data=7;endmodule module mon(input logic[3:0]s);initial #1 $display("T|%m=%0d",s);endmodule module binder#(parameter SEL=0);bind top.bank[SEL].m mon b(.s(data));endmodule module top;for(genvar g=0;g<2;g++)begin:bank mem m();end binder#(0) a();binder#(1) b();initial #2 $finish;endmodule"####,
        &["T|top.bank[0].m.b=7", "T|top.bank[1].m.b=7"],
    );
}

/// §23.11: compilation units may have equal byte offsets for distinct binds.
#[test]
fn repeated_bind_equal_offsets_across_files() {
    check_sources(
        &[
            "module mem;logic[3:0]data=7;endmodule module up;initial #1 $display(\"T|%m=%0d\",mem.data);endmodule module top;mem m();initial #2 $finish;endmodule",
            "bind top.m up a();\n",
            "bind top.m up b();\n",
        ],
        &["T|top.m.a=7", "T|top.m.b=7"],
    );
}
