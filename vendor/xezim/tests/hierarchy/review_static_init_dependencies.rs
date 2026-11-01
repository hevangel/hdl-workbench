//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    let result = xezim::simulate(src, 1_000_000).expect("simulation failed");
    assert_eq!(result.error_count, 0, "unexpected simulation error");
    result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn replicated_function_initializer_executes_once() {
    let got = output(
        r###"module child;
 int calls=0;
 function automatic logic [3:0] f(); calls++; return 4'h3; endfunction
 int r=$urandom_range(1,9);
 logic [71:0] b={2{r,f()}};
 initial begin #1; if (calls==1 && b=={2{r,4'h3}}) $display("T|PASS"); else $display("FAIL calls=%0d b=%h",calls,b); $finish; end
endmodule
module top; child u(); endmodule
"###,
    );
    assert!(got.contains("T|PASS"), "{got}");
}

#[test]
fn range_selected_reader_is_initialized() {
    let got = output(
        r###"module child;
 int r=$urandom_range(16,31);
 int a=r[3:0];
 int b=a+1;
 string p=$sformatf("%m");
 string q={p,".q"};
 initial begin #1; if (a==r[3:0] && b==a+1 && q=={p,".q"}) $display("T|PASS"); else $display("FAIL r=%0d a=%0d b=%0d",r,a,b); $finish; end
endmodule
module top; child u(); endmodule
"###,
    );
    assert!(got.contains("T|PASS"), "{got}");
}

#[test]
fn cast_reader_and_transitive_dependency() {
    let got = output(
        r###"module child;
 string a=$sformatf("%m");
 string b=string'(a);
 string c={b,".chain"};
 string direct={a,".chain"};
 initial begin $display("T|a %s b %s c %s direct %s",a,b,c,direct); $finish; end
endmodule

module top; child u(); endmodule
"###,
    );
    assert!(
        got.contains("T|a top.u b top.u c top.u.chain direct top.u.chain"),
        "{got}"
    );
}

#[test]
fn call_reader_and_transitive_dependency() {
    let got = output(
        r###"module child;
 string a=$sformatf("%m");
 string b=$sformatf("%s.chain",a);
 string c={b,".end"};
 initial begin $display("T|a %s b %s c %s",a,b,c); $finish; end
endmodule

module top; child u(); endmodule
"###,
    );
    assert!(
        got.contains("T|a top.u b top.u.chain c top.u.chain.end"),
        "{got}"
    );
}

#[test]
fn replicated_selected_reader_calls_once() {
    let got = output(
        r###"module child;
 int calls=0;
 function automatic logic [3:0] f(); calls++; return 4'h3; endfunction
 int r=$urandom_range(1,9);
 logic [15:0] b={2{r[3:0],f()}};
 initial begin #1; if (calls==1 && b=={2{r[3:0],4'h3}}) $display("T|PASS"); else $display("FAIL calls=%0d b=%h",calls,b); $finish; end
endmodule
module top; child u(); endmodule
"###,
    );
    assert!(got.contains("T|PASS"), "{got}");
}

#[test]
fn package_initializer_readers_keep_scope_and_run_once() {
    let got = output(
        r#"
package p;
 string a=$sformatf("%m");
 string b=string'(a);
 string c={b,".end"};
endpackage
package q;
 string a=$sformatf("%m");
 string b={a,".other"};
endpackage
module top;
 string a="module";
 initial begin #1; $display("T|p %s %s %s q %s module %s",p::a,p::b,p::c,q::b,a); $finish; end
endmodule
"#,
    );
    assert!(
        got.contains("T|p p p p.end q q.other module module"),
        "{got}"
    );
}

#[test]
fn replicated_runtime_system_function_is_not_folded_away() {
    let got = output(
        r#"module top;
 int v={2{$urandom_range(1,1)}};
 initial begin $display("T|replicated %0d",v); $finish; end
endmodule"#,
    );
    assert!(got.contains("T|replicated 1"), "{got}");
}

#[test]
fn package_object_initializers_use_the_exported_cell() {
    let source = r#"package p;
 int calls=0;
 class C; int x;
  function new(); calls++; x=5; endfunction
 endclass
 string label=$sformatf("%m");
 C obj=new;
endpackage
module top; import p::*;
 initial begin
  $display("T|object %0d qualified %0d calls %0d label %s",obj.x,p::obj.x,calls,label);
  $finish;
 end
endmodule"#;
    for source in [
        source.to_string(),
        source.replace(
            "string label=$sformatf(\"%m\");\n C obj=new;",
            "C obj=new;\n string label=$sformatf(\"%m\");",
        ),
    ] {
        let got = output(&source);
        assert!(
            got.contains("T|object 5 qualified 5 calls 1 label p"),
            "{got}"
        );
    }
}

#[test]
fn package_function_initializers_run_once_in_their_own_scope() {
    let got = output(
        r#"package p;
 int calls=0;
 string label=$sformatf("%m");
 function automatic string build(); calls++; return {label,".result"}; endfunction
 string result=build();
endpackage
package q;
 int calls=0;
 string label=$sformatf("%m");
 function automatic string build(); calls++; return {label,".result"}; endfunction
 string result=build();
endpackage
module top; initial begin
 $display("T|functions %s %s calls %0d %0d",p::result,q::result,p::calls,q::calls);
 $finish; end endmodule"#,
    );
    assert!(
        got.contains("T|functions p.result q.result calls 1 1"),
        "{got}"
    );
}

#[test]
fn package_constructor_reads_a_deferred_value() {
    let got = output(
        r#"package p;
 int seed_value=$urandom_range(3,3);
 class C; int x;
  function new(int value); x=value; endfunction
 endclass
 C obj=new(seed_value);
endpackage
module top; import p::*;
 initial begin $display("T|constructor %0d seed %0d",obj.x,seed_value); $finish; end
endmodule"#,
    );
    assert!(got.contains("T|constructor 3 seed 3"), "{got}");
}

#[test]
fn out_of_class_constructor_does_not_capture_new_initializers() {
    let got = output(
        r#"package p;
 int calls=0;
 typedef class C;
 C obj=new();
 string label=$sformatf("%m");
 class C;
  int x;
  extern function new();
 endclass
 function C::new(); calls++; x=5; endfunction
endpackage
module top; import p::*;
 initial begin
  $display("T|external constructor %0d calls %0d label %s",obj.x,calls,label);
  $finish;
 end
endmodule"#,
    );
    assert!(
        got.contains("T|external constructor 5 calls 1 label p"),
        "{got}"
    );
}

#[test]
fn deferred_initializers_call_each_instances_own_function() {
    let got = output(
        r#"interface bus #(parameter ID=0);
 int calls=0;
 string label=$sformatf("%m");
 int value=next_value();
 function automatic int next_value(); calls++; return ID; endfunction
endinterface
module child #(parameter ID=0);
 int calls=0;
 string label=$sformatf("%m");
 int value=next_value();
 function automatic int next_value(); calls++; return ID; endfunction
endmodule
module top;
 bus#(7) a(); bus#(9) b(); child#(11) c(); child#(13) d();
 initial begin
  $display("T|instance calls %0d %0d %0d %0d values %0d %0d %0d %0d",a.calls,b.calls,c.calls,d.calls,a.value,b.value,c.value,d.value);
  $finish;
 end
endmodule"#,
    );
    assert!(
        got.contains("T|instance calls 1 1 1 1 values 7 9 11 13"),
        "{got}"
    );
}
