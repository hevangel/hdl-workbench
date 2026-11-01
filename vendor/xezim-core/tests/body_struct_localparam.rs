//! A localparam typed by a body struct typedef is packed in every instance (§6.20.2 / §7.2.1).
fn elaborate(source: &str) -> xezim_core::ElaboratedModule {
    xezim_core::parse_and_elaborate_multi(
        &[source.to_string()],
        Some("top"),
        &[],
        &["top.sv".to_string()],
        &[],
    )
    .expect("elaborate")
    .1
}

#[test]
fn body_struct_localparam_is_packed_in_the_first_instance() {
    let design = elaborate(
        r#"
module m;
  typedef logic [1:0] field_t;
  typedef struct packed { logic a; logic b; field_t c; } s_t;
  localparam s_t R = '{a: 1'b0, b: 1'b1, c: 2'b10};
endmodule
module top; m u0(); m u1(); endmodule
"#,
    );
    let param = |name: &str| design.parameters.get(name).and_then(|v| v.to_u64());
    assert_eq!(param("u0.R"), Some(0b0110));
    assert_eq!(param("u1.R"), Some(0b0110));
}
