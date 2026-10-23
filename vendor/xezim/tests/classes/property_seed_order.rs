//! Derived property defaults hide inherited parameters (§8.10/§8.25).

use xezim::simulate;

const SOURCE: &str = r#"
class parameter_base #(int N = 7, int M = N + 2);
endclass
class default_derived extends parameter_base;
  bit [7:0] N;
endclass
class signed_derived extends parameter_base;
  bit signed [7:0] N;
endclass
class intermediate_derived extends parameter_base;
  bit [7:0] N;
endclass
class inherited_derived extends intermediate_derived;
endclass
class override_derived extends parameter_base #(13, 15);
  bit [7:0] N;
endclass
class localparam_base;
  localparam int N = 17;
endclass
class localparam_derived extends localparam_base;
  bit [7:0] N;
endclass
class initialized_derived extends parameter_base;
  bit [7:0] N = 23;
endclass
class pair_derived extends parameter_base;
  bit [7:0] N;
  bit [7:0] M;
endclass
class field_base;
  bit [7:0] N;
  function void set_base(); N = 11; endfunction
endclass
class field_derived extends field_base;
  bit [7:0] N;
  function int base_value(); return super.N; endfunction
endclass
class leaf_parameter #(int N = 19) extends field_base;
endclass
class width_base #(int W = 8);
  bit [W-1:0] payload;
endclass
class width_derived extends width_base #(12);
  bit flag;
endclass
module top;
  default_derived a, again;
  signed_derived b;
  inherited_derived c;
  override_derived d;
  localparam_derived e;
  initialized_derived f;
  pair_derived g;
  field_derived h;
  leaf_parameter j;
  width_derived k;
  int plain_default, signed_default, inherited_default, override_default;
  int local_default, initialized_value, pair_first, pair_second;
  int repeated_default, base_field, leaf_field, leaf_param, payload_width, payload_value;
  int dependent_default;
  initial begin
    a=new; b=new; c=new; d=new; e=new; f=new; g=new; h=new; j=new; k=new;
    plain_default=a.N; dependent_default=a.M; signed_default=b.N; inherited_default=c.N;
    override_default=d.N; local_default=e.N; initialized_value=f.N;
    pair_first=g.N; pair_second=g.M;
    a.N=91; again=new; repeated_default=again.N;
    h.set_base(); base_field=h.base_value(); leaf_field=h.N;
    leaf_param=j.N; k.payload='1;
    payload_width=$bits(k.payload); payload_value=k.payload;
    $display("T|seeds %0d %0d %0d %0d %0d %0d %0d %0d %0d", plain_default,
      signed_default, inherited_default, override_default, local_default,
      initialized_value, pair_first, pair_second, repeated_default);
    $display("T|dependent %0d", dependent_default);
    $display("T|controls %0d %0d %0d %0d %0d", base_field, leaf_field,
      leaf_param, payload_width, payload_value);
  end
endmodule
"#;

#[test]
fn property_seeds_and_parameters_follow_each_declaring_class() {
    if !super::compiled_method_test_env::policies(&[("0", "1000"), ("1", "0"), ("1", "1000")]) {
        return;
    }
    let sim = simulate(SOURCE, 1).expect("class seed ordering regression");
    for (name, expected) in [
        ("plain_default", 0),
        ("dependent_default", 9),
        ("signed_default", 0),
        ("inherited_default", 0),
        ("override_default", 0),
        ("local_default", 0),
        ("initialized_value", 23),
        ("pair_first", 0),
        ("pair_second", 0),
        ("repeated_default", 0),
        ("base_field", 11),
        ("leaf_field", 0),
        ("leaf_param", 19),
        ("payload_width", 12),
        ("payload_value", 4095),
    ] {
        let actual = sim
            .get_signal(name)
            .or_else(|| sim.get_signal(&format!("top.{name}")))
            .unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(actual.to_u64(), Some(expected), "{name}");
    }
}
