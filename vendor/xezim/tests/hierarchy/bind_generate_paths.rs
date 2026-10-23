//! §23.11 / §27: bind through selected generate scopes.
//! Expected values were checked with the reference simulator.
use xezim::simulate;

fn check(source: &str, expected: &[&str]) {
    let sim = simulate(source, 10).expect("simulate bind through generate scope");
    let mut lines: Vec<_> = sim
        .output
        .iter()
        .map(|o| o.message.trim_end())
        .filter(|line| line.starts_with("T|"))
        .collect();
    lines.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(lines, expected);
}

#[test]
fn bind_original() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
for(genvar g=0;g<2;g++) begin: bank wrap w(); end
bind top.bank[0].w.m mif mi(.s(MEMORY[1]));
initial begin #1; $display("T|bound=%0d",top.bank[0].w.m.mi.s); $finish; end
endmodule

"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_other_index() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
for(genvar g=0;g<2;g++) begin: bank wrap w(); end
bind top.bank[1].w.m mif mi(.s(MEMORY[1]));
initial begin #1; $display("T|bound=%0d",top.bank[1].w.m.mi.s); $finish; end
endmodule

"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_constant_expression() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
for(genvar g=0;g<2;g++) begin: bank wrap w(); end
bind top.bank[1-1].w.m mif mi(.s(MEMORY[1]));
initial begin #1; $display("T|bound=%0d",top.bank[1-1].w.m.mi.s); $finish; end
endmodule

"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_nested() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
for(genvar g=0;g<2;g++) begin: bank for(genvar j=0;j<2;j++) begin: lane wrap w(); end end
bind top.bank[1].lane[0].w.m mif mi(.s(MEMORY[1]));
initial begin #1; $display("T|bound=%0d",top.bank[1].lane[0].w.m.mi.s); $finish; end
endmodule

"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_unit_scope() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
for(genvar g=0;g<2;g++) begin: bank wrap w(); end

initial begin #1; $display("T|bound=%0d",top.bank[0].w.m.mi.s); $finish; end
endmodule
bind top.bank[0].w.m mif mi(.s(MEMORY[1]));
"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_colon_list() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
for(genvar g=0;g<2;g++) begin: bank wrap w(); end
bind mem : top.bank[0].w.m mif mi(.s(MEMORY[1]));
initial begin #1; $display("T|bound=%0d",top.bank[0].w.m.mi.s); $finish; end
endmodule

"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_ordinary() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
wrap w();
bind top.w.m mif mi(.s(MEMORY[1]));
initial begin #1; $display("T|bound=%0d",top.w.m.mi.s); $finish; end
endmodule

"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_named_if() {
    check(
        r####"interface mif(input logic [3:0] s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top;
if(1) begin: active wrap w(); end
bind top.active.w.m mif mi(.s(MEMORY[1]));
initial begin #1; $display("T|bound=%0d",top.active.w.m.mi.s); $finish; end
endmodule

"####,
        &["T|bound=7"],
    );
}

#[test]
fn bind_siblings() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top; for(genvar g=0;g<2;g++) begin: bank wrap w(); end bind top.bank[0].w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &["T|top.bank[0].w.m.mi=7"],
    );
}

#[test]
fn bind_two_targets() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top; for(genvar g=0;g<3;g++) begin: bank wrap w(); end bind mem : top.bank[0].w.m, top.bank[2].w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &["T|top.bank[0].w.m.mi=7", "T|top.bank[2].w.m.mi=7"],
    );
}

#[test]
fn bind_root_parameter() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top; localparam int SEL=1; for(genvar g=0;g<3;g++) begin: bank wrap w(); end bind top.bank[SEL].w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &["T|top.bank[1].w.m.mi=7"],
    );
}

#[test]
fn bind_child_parameter() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module host #(parameter int N=1); for(genvar g=0;g<N;g++) begin: bank wrap w(); end endmodule module top; host #(3) h(); host decoy(); bind top.h.bank[2].w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &["T|top.h.bank[2].w.m.mi=7"],
    );
}

#[test]
fn bind_negative_index() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top; for(genvar g=-2;g<1;g++) begin: bank wrap w(); end bind top.bank[-1].w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &["T|top.bank[-1].w.m.mi=7"],
    );
}

#[test]
fn bind_case_scope() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top; case(1) 1: begin: chosen wrap w(); end default: begin: other wrap w(); end endcase bind top.chosen.w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &["T|top.chosen.w.m.mi=7"],
    );
}

#[test]
fn bind_branch_decoy() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module host #(parameter bit ON=1); if(ON) begin: active wrap w(); end else begin: active wrap w(); end endmodule module top; host h(); host #(0) decoy(); bind top.h.active.w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &["T|top.h.active.w.m.mi=7"],
    );
}

#[test]
fn bind_module_and_path() {
    check(
        r####"interface mif(input logic [3:0] s); initial #1 $display("T|%m=%0d",s); endinterface
module mem; logic [3:0] MEMORY[0:3]; initial MEMORY[1]=4'd7; endmodule
module wrap; mem m(); endmodule
module top; for(genvar g=0;g<2;g++) begin: bank wrap w(); end bind mem mif all_mi(.s(MEMORY[1])); bind top.bank[0].w.m mif mi(.s(MEMORY[1])); endmodule
"####,
        &[
            "T|top.bank[0].w.m.mi=7",
            "T|top.bank[0].w.m.all_mi=7",
            "T|top.bank[1].w.m.all_mi=7",
        ],
    );
}
