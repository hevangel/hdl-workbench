//! §6.18: compiled locals retain the width, signing and state of typedefs.
//! Alias, parameter-width, initializer and result checks were verified with
//! the reference simulator (§10.8 and §13.4 also apply).
//! The loop has an iteration guard so a signing regression cannot hang.

use std::process::Command;
use xezim::simulate;

fn isolated_policies() -> bool {
    let thread = std::thread::current();
    let name = thread.name().expect("named test thread");
    if std::env::var("XEZIM_TYPEDEF_362_CHILD").as_deref() == Ok(name) {
        return true;
    }
    for (enabled, tier) in [("0", "0"), ("1", "0"), ("1", "1000")] {
        let output = Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", name, "--nocapture", "--test-threads=1"])
            .env("XEZIM_TYPEDEF_362_CHILD", name)
            .env("XEZIM_COMPILE_METHODS", enabled)
            .env("XEZIM_METHOD_TIER", tier)
            .env_remove("XEZIM_METHOD_CACHE")
            .output()
            .expect("isolated policy test");
        assert!(
            output.status.success(),
            "{name}: methods={enabled}, tier={tier}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("1 passed"),
            "isolated test filter did not run {name}"
        );
    }
    false
}

const SOURCE: &str = r#"
package kinds362;
  typedef byte signed_byte;
  typedef signed_byte chain_byte;
  typedef logic [64:0] word65;
endpackage
module typed362 #(parameter W=8) (output logic [64:0] result);
  typedef logic [W-1:0] local_word;
  typedef local_word alias_word;
  function automatic logic [64:0] widen(input logic [64:0] a);
    alias_word x; x = a; return x;
  endfunction
  always_comb result = widen(65'h1_0000_0000_0000_0005);
endmodule
module top;
  typedef int sint_t;
  typedef byte sbyte_t;
  typedef sbyte_t sbyte2_t;
  typedef logic [63:0] w_t;
  typedef logic [31:0] local_t;
  typedef bit [7:0] bit8_t;
  typedef logic [7:0] logic8_t;
  typedef integer integer_t;
  typedef byte unsigned ubyte_t;
  logic [3:0] v = 4'b1010;
  logic [7:0] u8 = 8'hf0;
  logic [31:0] u32 = 32'hfffffff8;
  int cnt, iterations;
  logic lt;
  logic [15:0] ext;
  int divv;
  logic [31:0] defaults;
  logic [15:0] pkg;
  logic [64:0] inst8, inst65;
  logic [64:0] wide65;
  logic [15:0] uns;
  typed362 #(8) u8inst(inst8);
  typed362 #(65) u65inst(inst65);
  function automatic logic f_lt(input logic [7:0] a);
    sbyte_t x; x = a; return x < 0;
  endfunction
  function automatic logic [15:0] f_ext(input logic [7:0] a);
    sbyte2_t x; x = a; return x;
  endfunction
  function automatic int f_div(input logic [31:0] a);
    sint_t x; x = a; return x / 4;
  endfunction
  function automatic logic [31:0] f_defaults();
    sint_t a; bit8_t b; logic8_t c; integer_t d;
    return {a == 0, b == 0, c === 8'hxx, d === 32'hxxxxxxxx, 28'h0};
  endfunction
  function automatic logic [15:0] f_pkg(input logic [7:0] a);
    kinds362::chain_byte x; x = a; return x;
  endfunction
  function automatic logic [64:0] f65(input logic [64:0] a);
    kinds362::word65 x; x = a; return x;
  endfunction
  function automatic logic [15:0] f_unsigned(input logic [7:0] a);
    ubyte_t x; x = a; return x;
  endfunction
  class E;
    function longint wide(); w_t x; x = 64'h1_0000_0005; return x; endfunction
    function int sext(); sbyte_t b; b = 8'hf0; return b; endfunction
    function int count(); static sint_t k; k++; return k; endfunction
  endclass
  class Own;
    typedef byte local_t;
    typedef local_t alias_t;
    function int value(); alias_t x; x = 8'hf0; return x; endfunction
  endclass
  always_comb begin
    cnt = 0; iterations = 0;
    for (sint_t i = 3; i >= 0; i--) begin
      cnt += v[i];
      iterations++;
      if (iterations > 16) break;
    end
    lt = f_lt(u8); ext = f_ext(u8); divv = f_div(u32);
    defaults = f_defaults(); pkg = f_pkg(u8); wide65 = f65(65'h1_0000_0000_0000_0005);
    uns = f_unsigned(u8);
  end
  initial begin
    E e; Own o; longint a; int c, g, own;
    e = new; o = new;
    for (int i=0; i<1200; i++) begin a=e.wide(); c=e.sext(); g=e.count(); own=o.value(); end
    #1;
    $display("T|loop %0d %0d",cnt,iterations);
    $display("T|function %b %h %0d",lt,ext,divv);
    $display("T|method %h %0d %0d",a,c,g);
    $display("T|defaults %h",defaults);
    $display("T|package %h",pkg);
    $display("T|instances %h %h",inst8,inst65);
    $display("T|wide65 %h",wide65);
    $display("T|unsigned %h",uns);
    $display("T|class-alias %0d",own);
    $finish;
  end
endmodule
"#;

#[test]
fn typedef_local_types_survive_inline_and_method_compilation() {
    if !isolated_policies() {
        return;
    }
    let sim = simulate(SOURCE, 20).expect("typedef simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "T|loop 2 4",
            "T|function 1 fff0 -2",
            "T|method 0000000100000005 -16 1200",
            "T|defaults f0000000",
            "T|package fff0",
            "T|instances 00000000000000005 10000000000000005",
            "T|wide65 10000000000000005",
            "T|unsigned 00f0",
            "T|class-alias -16",
        ]
    );
}

// §23.8/§11.5: selected formal/local writes keep their lexical object even
// when a module variable has the same name. The input fixture also checks
// task locals, class properties and process locals across suspension.
#[test]
fn selected_locals_keep_shadowing_under_eager_method_compilation() {
    if !isolated_policies() {
        return;
    }
    let sim = simulate(include_str!("../issue_cases/issue344_scope.sv"), 1000)
        .expect("selected local simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert!(!messages.iter().any(|m| m.contains("FAIL")), "{messages:?}");
    assert_eq!(
        messages.iter().filter(|m| m.starts_with("PASS")).count(),
        10,
        "{messages:?}"
    );
    assert!(messages.contains(&"ALL PASS"), "{messages:?}");
}

// §8.25: a nonliteral class-alias bound remains specialization-dependent.
#[test]
fn parameter_dependent_class_alias_retains_interpreted_width() {
    if !isolated_policies() {
        return;
    }
    let source = r#"
class Param362 #(int W=8);
  typedef logic [W-1:0] word_t;
  typedef word_t alias_t;
  function int width();
    alias_t x;
    return $bits(word_t);
  endfunction
endclass
module top;
  Param362#(4) a;
  Param362#(12) b;
  Param362#(65) c;
  int av,bv,cv;
  initial begin
    a=new; b=new; c=new;
    for(int i=0; i<1200; i++) begin av=a.width(); bv=b.width(); cv=c.width(); end
    $display("T|parameter-alias %0d %0d %0d",av,bv,cv);
    $finish;
  end
endmodule
"#;
    let sim = simulate(source, 20).expect("parameterized typedef simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(messages, ["T|parameter-alias 4 12 65"]);
}

#[test]
fn initialized_typedef_locals_and_formals_fit_their_declared_signing() {
    if !isolated_policies() {
        return;
    }
    let source = r#"
module top;
  typedef byte signed_byte;
  typedef byte unsigned unsigned_byte;
  typedef logic [7:0] four_byte;
  typedef bit [7:0] two_byte;
  logic [7:0] a=8'hf0;
  byte b=-16;
  logic [7:0] unknowns=8'bxz110011;
  logic [15:0] r0,r1,r2,r3;
  logic [7:0] r4,r5;
  function automatic logic [15:0] signed_init(input logic[7:0] a);
    signed_byte x=a; return x;
  endfunction
  function automatic logic [15:0] unsigned_init(input byte a);
    unsigned_byte x=a; return x;
  endfunction
  function automatic logic [15:0] signed_formal(input signed_byte x);
    return x;
  endfunction
  function automatic logic [15:0] unsigned_formal(input unsigned_byte x);
    return x;
  endfunction
  function automatic logic [7:0] four_init(input logic[7:0] a);
    four_byte x=a; return x;
  endfunction
  function automatic logic [7:0] two_init(input logic[7:0] a);
    two_byte x=a; return x;
  endfunction
  always_comb begin
    r0=signed_init(a); r1=unsigned_init(b);
    r2=signed_formal(a); r3=unsigned_formal(b);
    r4=four_init(unknowns); r5=two_init(unknowns);
  end
  initial #1 begin
    $display("T|signed-init %h",r0);
    $display("T|unsigned-init %h",r1);
    $display("T|signed-formal %h",r2);
    $display("T|unsigned-formal %h",r3);
    $display("T|four-init %b",r4);
    $display("T|two-init %b",r5);
    $finish;
  end
endmodule
"#;
    let sim = simulate(source, 20).expect("initialized typedef simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "T|signed-init fff0",
            "T|unsigned-init 00f0",
            "T|signed-formal fff0",
            "T|unsigned-formal 00f0",
            "T|four-init xz110011",
            "T|two-init 00110011",
        ]
    );
}

#[test]
fn typedef_result_and_static_default_preserve_signing() {
    if !isolated_policies() {
        return;
    }
    let source = r#"
module top;
  typedef byte signed_byte;
  typedef byte unsigned unsigned_byte;
  typedef logic signed [7:0] signed_logic;
  logic [7:0] a=8'hf0;
  logic [15:0] x,y,z;
  function automatic signed_byte explicit_result(input logic[7:0] a);
    return a;
  endfunction
  function automatic signed_byte implicit_result(input logic[7:0] a);
    implicit_result=a;
  endfunction
  function automatic unsigned_byte unsigned_result(input byte a);
    return a;
  endfunction
  class Defaults362;
    function logic [7:0] value();
      static signed_logic a;
      return a;
    endfunction
  endclass
  always_comb begin x=explicit_result(a); y=implicit_result(a); z=unsigned_result(a); end
  initial begin
    Defaults362 d;
    logic [7:0] v;
    d=new;
    for(int i=0; i<1200; i++) v=d.value();
    #1;
    $display("T|returns %h %h %h",x,y,z);
    $display("T|signed-default %h",v);
    $finish;
  end
endmodule
"#;
    let sim = simulate(source, 20).expect("typedef result simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        messages,
        ["T|returns fff0 fff0 00f0", "T|signed-default xx"]
    );
}

// §13.5/§10.8: formal and implicit-result seeds own their scalar type before
// the first local store. String, handle and unpacked-array paths stay distinct.
#[test]
fn formal_and_implicit_result_seeds_keep_declared_width_and_signing() {
    if !isolated_policies() {
        return;
    }
    let sim = simulate(
        include_str!("../issue_cases/p1_formal_seed_neighbors.sv"),
        20,
    )
    .expect("formal seed simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "T|output-byte -16",
            "T|inout-byte -16",
            "T|task-typedef -16",
            "T|function-typedef -16",
            "T|function-return -8",
            "T|method-return -8",
            "T|text ok!",
            "T|handle 17",
            "T|array 3 7",
        ]
    );
}

// §6.11.1/§13.5: a two-state formal is clean before it supplies another
// local, including a compiled method call that bypasses the AST binder.
#[test]
fn two_state_method_formals_are_clean_before_local_assignment() {
    if !isolated_policies() {
        return;
    }
    let sim = simulate(
        include_str!("../issue_cases/p1_method_formal_state_neighbors.sv"),
        20,
    )
    .expect("method formal state simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "T|byte-state 1",
            "T|int-state 1",
            "T|bit-state 1",
            "T|vector-state 1",
            "T|alias-state 1",
            "T|four-state xz110011",
            "T|direct-byte-state 1",
        ]
    );
}

// §6.16/§13.5: text formals preserve text storage across writes and waits;
// an unresolved type-parameter formal retains the interpreter's binding.
#[test]
fn string_formals_keep_text_storage_across_compilation_policies() {
    if !isolated_policies() {
        return;
    }
    let sim = simulate(
        include_str!("../issue_cases/p1_string_formal_neighbors.sv"),
        20,
    )
    .expect("string formal simulation");
    let messages: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "T|direct Abc!",
            "T|alias Bbc!",
            "T|chain Cbc!",
            "T|scoped Dbc!",
            "T|parameter Ebc!",
            "T|packed 12345679",
        ]
    );
}

#[test]
fn suspended_string_formals_keep_kind_under_compilation_policies() {
    if !isolated_policies() {
        return;
    }
    let sim = simulate(
        include_str!("../issue_cases/p1_blocking_string_formal_neighbors.sv"),
        1000,
    )
    .expect("blocking string formal simulation");
    let lines: Vec<_> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        lines,
        [
            "T|direct Abc!",
            "T|alias Bbc!",
            "T|parameter-string Ebc!",
            "T|parameter-int 12345679",
            "T|static-string Sbc!",
            "T|static-int 87654320",
            "T|parameter-string-again Eyz!",
        ]
    );
}
