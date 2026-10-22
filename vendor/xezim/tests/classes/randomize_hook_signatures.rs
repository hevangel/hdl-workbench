//! §18.11: pre_randomize and post_randomize are void functions with no
//! arguments. Reject invalid profiles before either callback can execute.

use xezim::simulate;

#[test]
fn invalid_randomization_callback_profiles_are_rejected() {
    for hook in ["pre_randomize", "post_randomize"] {
        for declaration in [
            format!("task {hook}(); $display(\"EXECUTED\"); endtask"),
            format!("function int {hook}(); return 1; endfunction"),
            format!("function {hook}(); return 1; endfunction"),
            format!("function void {hook}(int n); endfunction"),
            format!("function void {hook}(int n = 0); endfunction"),
            format!("function void {hook}; input int n; endfunction"),
            format!("extern function int {hook}();"),
            format!("extern task {hook}();"),
        ] {
            let source = format!(
                "class packet; rand bit [7:0] data; {declaration} endclass\n\
                 module top; initial begin packet p = new; void'(p.randomize()); end endmodule"
            );
            let error = match simulate(&source, 100) {
                Err(error) => error,
                Ok(_) => panic!("accepted invalid callback: {declaration}"),
            };
            assert!(error.contains(hook) && error.contains("§18.11"), "{error}");
        }
    }
}

#[test]
fn inherited_and_overridden_void_callbacks_execute() {
    let source = r#"
class base_packet;
  rand bit [7:0] data;
  int before_count, after_count;
  function void pre_randomize(); before_count++; endfunction
  function void post_randomize(); after_count++; endfunction
endclass
class inherited_packet extends base_packet; endclass
class overridden_packet extends base_packet;
  function void post_randomize(); super.post_randomize(); after_count += 10; endfunction
endclass
module top;
  initial begin
    inherited_packet a = new;
    overridden_packet b = new;
    void'(a.randomize()); void'(b.randomize());
    $display("T|hooks %0d %0d %0d %0d", a.before_count, a.after_count,
             b.before_count, b.after_count);
  end
endmodule
"#;
    let result = simulate(source, 100).expect("legal callbacks");
    assert!(
        result
            .output
            .iter()
            .any(|o| o.message == "T|hooks 1 1 1 11")
    );
}
