//! IEEE 1800 §10.7: property stores use the declared destination type.

use xezim::simulate;

fn check(source: &str, expected: &[&str]) {
    let result = simulate(source, 100).unwrap();
    let actual: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn original_issue_499() {
    let result = simulate(include_str!("../issue_cases/issue499.sv"), 100).unwrap();
    assert!(
        result.output.iter().any(|o| o.message == "ALL PASS"),
        "{:?}",
        result.output
    );
    assert!(
        !result.output.iter().any(|o| o.message.starts_with("FAIL")),
        "{:?}",
        result.output
    );
}

#[test]
fn method_and_external_stores_keep_declared_sign() {
    check(
        r#"class holder;int p;int unsigned q;
 function void set();this.p=32'hffff_fff6;this.q=-10;endfunction
endclass
module top;initial begin holder h=new;h.set();
 $display("T|method %0d %0d",h.p,h.q);
 h.p=32'hffff_fff6;h.q=-10;$display("T|external %0d %0d",h.p,h.q);
 h.p=-10;h.p=32'hffff_fff6;h.q=32'hffff_fff6;h.q=-10;
 $display("T|repeat %0d %0d",h.p,h.q);
end endmodule"#,
        &[
            "T|method -10 4294967286",
            "T|external -10 4294967286",
            "T|repeat -10 4294967286",
        ],
    );
}

#[test]
fn static_property_stores_keep_declared_sign() {
    check(
        r#"class holder;static int p;static int unsigned q;
 static function void set();p=32'hffff_fff6;q=-10;endfunction
endclass
module top;initial begin holder h=new;holder::set();
 $display("T|static %0d %0d",holder::p,holder::q);
 holder::p=32'hffff_fff6;holder::q=-10;
 $display("T|scoped %0d %0d",holder::p,holder::q);
 h.p=32'hffff_fff6;h.q=-10;$display("T|shared %0d %0d",holder::p,holder::q);
end endmodule"#,
        &[
            "T|static -10 4294967286",
            "T|scoped -10 4294967286",
            "T|shared -10 4294967286",
        ],
    );
}

#[test]
fn inherited_shadowed_properties_keep_each_declaration() {
    check(
        r#"class base;int p;
 function void set_base();p=32'hffff_fff6;endfunction
 function longint read_base();return p;endfunction
endclass
class derived extends base;int unsigned p;
 function void set_derived();p=-10;endfunction
endclass
module top;initial begin derived h=new;h.set_base();h.set_derived();
 $display("T|shadow %0d %0d",h.read_base(),h.p);
end endmodule"#,
        &["T|shadow -10 4294967286"],
    );
}

#[test]
fn type_parameter_signedness_is_per_instance() {
    check(
        r#"class holder #(type T=int);T p;
 function void set();p=32'hffff_fff6;endfunction
endclass
module top;initial begin holder h=new;holder#(int unsigned) u=new;
 h.set();u.set();$display("T|generic %0d %0d",h.p,u.p);
end endmodule"#,
        &["T|generic -10 4294967286"],
    );
}

#[test]
fn typedef_properties_keep_sign_and_two_state_conversion() {
    check(
        r#"typedef logic signed[7:0] signed_t;typedef logic[7:0] unsigned_t;
class holder;signed_t p;unsigned_t q;bit[7:0] r;
 function void set();p=8'hf6;q=-8'sd10;r=8'hx5;endfunction
endclass
module top;initial begin holder h=new;h.set();
 $display("T|aliases %0d %0d %0d",h.p,h.q,h.r);
end endmodule"#,
        &["T|aliases -10 246 5"],
    );
}

#[test]
fn wide_properties_keep_declared_sign() {
    check(
        r#"class holder;logic signed[127:0] p;logic[127:0] q;
 function void set();p=128'hffff_ffff_ffff_ffff_ffff_ffff_ffff_fff6;q=-128'sd10;endfunction
endclass
module top;initial begin holder h=new;h.set();
 $display("T|wide %0d %0d %0d %0d",h.p<0,h.q>0,h.p===-128'sd10,h.q===128'hffff_ffff_ffff_ffff_ffff_ffff_ffff_fff6);
end endmodule"#,
        &["T|wide 1 1 1 1"],
    );
}

#[test]
fn declaration_initializers_take_the_property_type() {
    check(
        r#"class holder;int p=32'hffff_fff6;int unsigned q=-10;endclass
module top;initial begin holder h=new;$display("T|initial %0d %0d",h.p,h.q);end endmodule"#,
        &["T|initial -10 4294967286"],
    );
}

#[test]
fn inherited_type_parameter_uses_the_base_binding() {
    check(
        r#"class base #(type T=int);T p;
 function void set_base();p=32'hffff_fff6;endfunction
 function longint read_base();return p;endfunction
endclass
class derived extends base#(shortint);int unsigned p;
 function void set_derived();p=-10;endfunction
endclass
module top;initial begin derived h=new;h.set_base();h.set_derived();
 $display("T|generic_shadow %0d %0d",h.read_base(),h.p);
end endmodule"#,
        &["T|generic_shadow -10 4294967286"],
    );
}

#[test]
fn hidden_two_state_base_property_discards_unknown_bits() {
    check(
        r#"class base;int p;
 function void set_base();p=32'hxxxx_fff6;endfunction
 function longint read_base();return p;endfunction
endclass
class derived extends base;logic[31:0] p;
 function void set_derived();p=32'hxxxx_fff6;endfunction
endclass
module top;initial begin derived h=new;h.set_base();h.set_derived();
 $display("T|two_state_shadow %0d %0d",h.read_base(),$isunknown(h.p));
end endmodule"#,
        &["T|two_state_shadow 65526 1"],
    );
}
