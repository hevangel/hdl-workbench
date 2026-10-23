//! Test for whole struct copy and multi-segment property assignments when a struct
//! contains a class handle.
//!
//! Checks:
//!   - Struct member assignment writing a class handle (`stor.handle = new()`).
//!   - Multi-segment property write through a struct member (`stor.handle.val = 777`).
//!   - Whole struct copy (`cp = stor`) preserving class handles and properties across copy.

use xezim::simulate;

const SRC: &str = r#"
class some_class;
   int val;
endclass

typedef struct {
   some_class handle;
   string     name;
} my_pair_t;

module top;
   initial begin
      automatic my_pair_t stor;
      automatic my_pair_t cp;
      stor.handle = new();
      stor.handle.val = 777;
      stor.name = "hello";
      cp = stor;
      if (cp.handle != null && cp.handle.val == 777 && cp.name == "hello")
        $display("TAG_PASS");
      else
        $display("TAG_FAIL: cp.handle=%0d val=%0d name=%s",
                 cp.handle != null,
                 cp.handle != null ? cp.handle.val : 0,
                 cp.name);
   end
endmodule
"#;

fn line(sim: &xezim::compiler::Simulator, tag: &str) -> String {
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .find(|m| m.starts_with(tag))
        .unwrap_or_else(|| panic!("no output line starting with {}", tag))
}

#[test]
fn struct_copy_with_class_handle() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(line(&sim, "TAG_"), "TAG_PASS");
}
