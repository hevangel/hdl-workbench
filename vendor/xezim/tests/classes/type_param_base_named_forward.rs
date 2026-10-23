//! Named inheritance arguments bind to parameter slots before substitution.
//! Constructor state and casts must survive both direct and nested forwarding.

use xezim::simulate;

const SRC: &str = r#"
class token_record; endclass
class origin_record;
  int marker = 5;
  token_record token;
  function new(); token = new(); endfunction
endclass
class settings_record; endclass
class generic_layer #(type PARENT = int, type SETTINGS = settings_record, type COPY_SETTINGS = SETTINGS) extends PARENT;
  function new(); super.new(); endfunction
endclass
class named_layer #(type PARENT = int) extends generic_layer #(.PARENT(PARENT));
  function new(); super.new(); endfunction
endclass
class ordered_layer #(type PARENT = int) extends generic_layer #(PARENT);
endclass
class renamed_layer #(type OTHER_PARENT = int) extends generic_layer #(.PARENT(OTHER_PARENT));
endclass
// two named levels, other parameters named alongside, one left to its default
class middle_layer #(type PARENT = int, type SETTINGS = settings_record) extends generic_layer #(.SETTINGS(SETTINGS), .PARENT(PARENT));
  function new(); super.new(); endfunction
endclass
class upper_layer #(type PARENT = int) extends middle_layer #(.PARENT(PARENT));
  function new(); super.new(); endfunction
endclass
module top;
  initial begin
    origin_record base_view;
    named_layer #(origin_record) named_object = new();
    ordered_layer   #(origin_record) ordered_object = new();
    renamed_layer #(origin_record) renamed_object = new();
    upper_layer   #(origin_record) nested_object = new();
    $display("named=%0d,%0d pos=%0d other=%0d two=%0d,%0d cast=%0d,%0d",
             named_object.marker, named_object.token != null, ordered_object.marker, renamed_object.marker, nested_object.marker, nested_object.token != null,
             $cast(base_view, named_object), $cast(base_view, nested_object));
  end
endmodule
"#;

#[test]
fn type_param_forwarded_by_name_resolves_base() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|renamed_object| renamed_object.message.clone())
        .collect();
    assert_eq!(out, ["named=5,1 pos=5 other=5 two=5,1 cast=1,1"], "{out:?}");
}
