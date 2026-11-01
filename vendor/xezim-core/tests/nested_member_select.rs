//! §7.2.2 / §6.20.2: `array[i].field.subfield` on a localparam array of
//! packed structs is that member, not a 32-bit 0.

fn elaborate(source: &str) -> xezim_core::ElaboratedModule {
    xezim_core::parse_and_elaborate_multi(
        &[source.to_string()],
        Some("t"),
        &[],
        &["t.sv".to_string()],
        &[],
    )
    .expect("elaborate")
    .1
}

#[test]
fn nested_member_of_array_param_is_not_zero() {
    let model = elaborate(
        r#"
typedef struct packed {
  logic       accept;
  logic [1:0] gpr;
} inner_t;
typedef struct packed {
  logic [31:0] instr;
  inner_t      rsp;
} outer_t;
module t;
  localparam inner_t INNERS[1] = '{'{accept: 1'b1, gpr: 2'b01}};
  localparam outer_t OUTERS[1] = '{'{instr: 32'h0400702b, rsp: '{accept: 1'b1, gpr: 2'b01}}};
  localparam int inner_arr = INNERS[0].accept;
  localparam int inner_bits = $bits(INNERS[0].accept);
  localparam int chain = OUTERS[0].rsp.accept;
  localparam int chain_bits = $bits(OUTERS[0].rsp.accept);
  localparam int whole = OUTERS[0].rsp;
endmodule
"#,
    );
    let val = |name: &str| {
        model
            .parameters
            .get(name)
            .unwrap_or_else(|| panic!("{name} missing"))
            .to_i64()
            .unwrap_or_else(|| panic!("{name} not a number"))
    };
    assert_eq!(val("inner_arr"), 1, "one-level select");
    assert_eq!(val("inner_bits"), 1);
    assert_eq!(val("chain"), 1, "nested select");
    assert_eq!(val("chain_bits"), 1);
    assert_eq!(val("whole"), 5);
}

#[test]
fn qualified_package_array() {
    let model = elaborate(
        r#"package p;
 typedef struct packed {logic[3:0] pad; logic accept; } inner_t;
 typedef struct packed {inner_t rsp; logic[2:0] tail;} outer_t;
 localparam outer_t A[1]='{'{rsp:'{pad:4'ha,accept:1'b1},tail:3'b011}};
endpackage
module t; localparam int result=p::A[0].rsp.accept;
localparam int result_bits=$bits(p::A[0].rsp.accept); endmodule"#,
    );
    assert_eq!(model.parameters["result"].to_i64(), Some(1));
    assert_eq!(model.parameters["result_bits"].to_i64(), Some(1));
}

#[test]
fn nonzero_nested_offset() {
    let model = elaborate(
        r#"typedef struct packed {logic[3:0] pad; logic[1:0] accept; } inner_t;
 typedef struct packed {inner_t rsp; logic[7:0] tail;} outer_t;
module t; localparam outer_t A[1]='{'{rsp:'{pad:4'ha,accept:2'b10},tail:8'hff}};
localparam int result=A[0].rsp.accept; localparam int result_bits=$bits(A[0].rsp.accept); endmodule"#,
    );
    assert_eq!(model.parameters["result"].to_i64(), Some(2));
    assert_eq!(model.parameters["result_bits"].to_i64(), Some(2));
}

#[test]
fn wide_nested_member() {
    let model = elaborate(
        r#"typedef struct packed {logic[95:0] padding; logic accept;} inner_t;
typedef struct packed {inner_t rsp; logic[7:0] tail;} outer_t;
module t; localparam outer_t A[1]='{'{rsp:'{padding:'1,accept:1'b1},tail:8'hff}};
localparam int result=A[0].rsp.accept; localparam int result_bits=$bits(A[0].rsp.accept); endmodule"#,
    );
    assert_eq!(model.parameters["result"].to_i64(), Some(1));
    assert_eq!(model.parameters["result_bits"].to_i64(), Some(1));
}

#[test]
fn descending_array_root() {
    let model = elaborate(
        r#"typedef struct packed {logic accept; logic[1:0] tail;} inner_t;
typedef struct packed {logic[7:0] head;inner_t rsp;} outer_t;
module t; localparam outer_t A[5:4]='{'{head:8'hff,rsp:3'b101},'{head:8'h55,rsp:3'b010}};
localparam int result=A[5].rsp.accept; localparam int result_bits=$bits(A[5].rsp.accept); endmodule"#,
    );
    assert_eq!(model.parameters["result"].to_i64(), Some(1));
    assert_eq!(model.parameters["result_bits"].to_i64(), Some(1));
}

#[test]
fn scalar_struct_control() {
    let model = elaborate(
        r#"typedef struct packed {logic accept; logic[1:0] tail;} inner_t;
typedef struct packed {logic[7:0] head;inner_t rsp;} outer_t;
module t; localparam outer_t A='{head:8'hff,rsp:3'b101};
localparam int result=A.rsp.accept; localparam int result_bits=$bits(A.rsp.accept); endmodule"#,
    );
    assert_eq!(model.parameters["result"].to_i64(), Some(1));
    assert_eq!(model.parameters["result_bits"].to_i64(), Some(1));
}

#[test]
fn package_type_homonyms_keep_nested_layout() {
    let model = elaborate(
        r#"
package p;
 typedef struct packed { logic accept; logic[1:0] tail; } inner_t;
 typedef struct packed {logic[7:0] head; inner_t rsp;} outer_t;
 localparam outer_t A[1]='{'{head:8'hff,rsp:3'b101}};
endpackage
package q;
 typedef struct packed { logic[6:0] tail; logic accept; } inner_t;
 typedef struct packed {inner_t rsp;logic[15:0] head;} outer_t;
 localparam outer_t A[1]='{'{rsp:8'hfe,head:16'hffff}};
endpackage
module t;
 localparam int p_bit=p::A[0].rsp.accept;
 localparam int q_bit=q::A[0].rsp.accept;
 localparam int p_width=$bits(p::A[0].rsp);
 localparam int q_width=$bits(q::A[0].rsp);
endmodule
"#,
    );
    assert_eq!(model.parameters["p_bit"].to_i64(), Some(1));
    assert_eq!(model.parameters["q_bit"].to_i64(), Some(0));
    assert_eq!(model.parameters["p_width"].to_i64(), Some(3));
    assert_eq!(model.parameters["q_width"].to_i64(), Some(8));
}

#[test]
fn nested_member_width_sizes_a_packed_declaration() {
    let model = elaborate(
        r#"package p;
 typedef struct packed {logic[3:0] padding;logic accept;} inner_t;
 typedef struct packed {logic[7:0] head;inner_t rsp;} outer_t;
 localparam outer_t A[1]='{'{head:8'hff,rsp:5'b10101}};
endpackage
module t; logic[$bits(p::A[0].rsp)-1:0] probe; endmodule"#,
    );
    assert_eq!(model.signals["probe"].width, 5);
}

#[test]
fn module_array_uses_unit_struct_typedef() {
    let model = elaborate(
        r#"typedef struct packed {logic accept;logic[2:0] data;} inner_t;
 typedef struct packed {inner_t rsp;logic[7:0] tail;} outer_t;
module t; localparam outer_t A[1]='{'{rsp:4'b1010,tail:8'hff}};
localparam int result=A[0].rsp.accept;
localparam int result_bits=$bits(A[0].rsp.accept); endmodule"#,
    );
    assert_eq!(model.parameters["result"].to_i64(), Some(1));
    assert_eq!(model.parameters["result_bits"].to_i64(), Some(1));
}
