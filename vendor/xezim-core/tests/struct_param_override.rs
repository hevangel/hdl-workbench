//! A packed-struct parameter passed as another module's parameter override
//! keeps its fields (§6.20, §23.10).
//!
//! `child #(.Features(FPU_FEATURES))` bound the override value but never
//! registered the struct's field layout before the child's body localparams
//! were folded, so `localparam WIDTH = Features.Width` read a 32-bit 0 and a
//! port `[WIDTH-1:0]` elaborated to 2 bits. The parent's own select of the
//! same value was fine.
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

const STRUCT_PARAM: &str = r#"
package p;
  typedef struct packed {
    int unsigned Width;
    logic        Flag;
  } features_t;
  parameter features_t F = '{Width: 32, Flag: 1'b1};
endpackage

module child #(
  parameter p::features_t Features = p::F,
  localparam int unsigned WIDTH = Features.Width
) (
  output logic [WIDTH-1:0] result_o
);
endmodule

module parent #(
  parameter p::features_t FPU_FEATURES = p::F
);
  logic [31:0] result;
  child #(.Features(FPU_FEATURES)) u (.result_o(result));
endmodule

module t;
  parent u ();
endmodule
"#;

#[test]
fn overridden_struct_param_keeps_its_fields() {
    let design = elaborate(STRUCT_PARAM);
    // {Width: 32, Flag: 1} packs to 33 bits, Flag at bit 0.
    let parent = &design.parameters["u.FPU_FEATURES"];
    assert_eq!(parent.width, 33);
    assert_eq!(parent.to_u64(), Some(65));
    assert_eq!(design.parameters["u.u.WIDTH"].to_i64(), Some(32));
    assert_eq!(design.signals["u.u.result_o"].width, 32);
}

#[test]
fn overridden_struct_fields_follow_instance_type_and_binding() {
    let design = elaborate(
        r#"
package p;
 typedef struct packed {int unsigned Width;logic Flag;} a_t;
 typedef struct packed {logic[7:0] head;int unsigned Width;logic[2:0] Flag;} b_t;
 parameter a_t A='{Width:32,Flag:1'b1};
 parameter b_t B='{head:8'hff,Width:17,Flag:3'b101};
endpackage
module child #(parameter p::a_t Features=p::A,
 localparam int WIDTH=Features.Width)(output logic[WIDTH-1:0] result_o);
endmodule
module typed_child #(parameter type T=p::a_t,parameter T Features='0,
 localparam int WIDTH=Features.Width)(output logic[WIDTH-1:0] result_o);
endmodule
module parent;
 localparam p::a_t F=p::A;
 child#(.Features(F)) u();
endmodule
module t;
 child#(.Features({32'd11,1'b1})) a();
 child#(.Features({32'd29,1'b0})) b();
 child c(); defparam c.Features={32'd19,1'b1};
 typed_child#(.T(p::b_t),.Features(p::B)) d();
 parent e();
endmodule
"#,
    );
    for (instance, width) in [("a", 11), ("b", 29), ("c", 19), ("d", 17), ("e.u", 32)] {
        assert_eq!(
            design.parameters[&format!("{instance}.WIDTH")].to_i64(),
            Some(width)
        );
        assert_eq!(
            design.signals[&format!("{instance}.result_o")].width,
            width as u32
        );
    }
}
