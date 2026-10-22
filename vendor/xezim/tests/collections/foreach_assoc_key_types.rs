//! §7.8.2 / §12.7.3 — a `foreach` index variable over an associative array has
//! the array's KEY type. Reference-validated.
//!
//! An associative array declared in an instantiated child module (or a
//! generate scope) had no key type recorded: elaboration noted the key's width
//! and typedef name only for top-module declarations, so `foreach (mem[k])` in
//! the child bound a 32-bit unsigned `k` — `$bits(k)` was 32 for a
//! `logic [KW-1:0]` key, a negative `byte` key printed as 4294967293, and a
//! packed-struct key had no members. A typedef'd signed key (`shortint`
//! behind a typedef) was unsigned at the top level too, and a packed-struct
//! key's members read 0 there.

use xezim::simulate;

fn t_lines(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

/// The DDR exploration's shape: a parameter-sized key in a module task.
#[test]
fn parameter_sized_key_in_child_task() {
    const SRC: &str = r#"
module m #(parameter int KW = 26) ();
  logic [7:0] mem [logic [KW-1:0]];
  task report();
    foreach (mem[k]) $display("T| key=%h bits=%0d", k, $bits(k));
  endtask
  initial begin
    mem[26'h11] = 8'hAA;
    #1 report();
  end
endmodule
module tb;
  m u ();
  initial #5 $finish;
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(t_lines(&sim), ["T| key=0000011 bits=26"]);
}

/// Every key type, in two differently parameterized child instances, at the
/// top level, and in generate scopes.
#[test]
fn key_types_in_child_instances_and_generate_scopes() {
    const SRC: &str = r#"
typedef logic [11:0] k12_t;
typedef struct packed { logic [3:0] a; logic [5:0] b; } ks_t;
typedef enum logic [2:0] { E0, E1, E5 = 5 } ke_t;
typedef shortint sk_t;
module m #(parameter int KW = 26) ();
  typedef logic [KW+1:0] lk_t;
  logic [7:0] mem  [logic [KW-1:0]];
  logic [7:0] memt [k12_t];
  logic [7:0] mems [ks_t];
  logic [7:0] meme [ke_t];
  logic [7:0] memst[string];
  logic [7:0] memi [int];
  logic [7:0] memb [byte];
  logic [7:0] mems2[sk_t];
  logic [7:0] meml [lk_t];
  logic [7:0] memsg[logic signed [KW-1:0]];
  task report();
    foreach (mem[k])   $display("T|p key=%h bits=%0d", k, $bits(k));
    foreach (memt[k])  $display("T|t key=%h bits=%0d", k, $bits(k));
    foreach (mems[k])  $display("T|s key=%h bits=%0d a=%0d", k, $bits(k), k.a);
    foreach (meme[k])  $display("T|e key=%0d bits=%0d", k, $bits(k));
    foreach (memst[k]) $display("T|str key=%s len=%0d", k, k.len());
    foreach (memi[k])  $display("T|i key=%0d bits=%0d", k, $bits(k));
    foreach (memb[k])  $display("T|b key=%0d bits=%0d", k, $bits(k));
    foreach (mems2[k]) $display("T|sk key=%0d bits=%0d", k, $bits(k));
    foreach (meml[k])  $display("T|l key=%h bits=%0d", k, $bits(k));
    foreach (memsg[k]) $display("T|sg key=%0d bits=%0d", k, $bits(k));
  endtask
  initial begin
    mem[26'h11] = 8'hAA;
    memt[12'h123] = 1;
    mems['{a:4'h3, b:6'h5}] = 2;
    meme[E5] = 3;
    memst["hello"] = 4;
    memi[-5] = 5;
    memb[-3] = 6;
    mems2[-7] = 7;
    meml[28'h1] = 8;
    memsg[-2] = 9;
    #1 report();
    foreach (mem[k]) $display("T|pi key=%h bits=%0d", k, $bits(k));
  end
endmodule
module tb;
  m u ();
  m #(.KW(10)) v ();
  logic [7:0] tmem [logic [19:0]];
  initial begin
    tmem[20'h5] = 1;
    #2 foreach (tmem[k]) $display("T|top key=%h bits=%0d", k, $bits(k));
  end
  genvar gi;
  for (gi = 0; gi < 2; gi++) begin : g
    logic [7:0] gmem [logic [gi*4+7:0]];
    initial begin
      gmem[3] = 1;
      #3 foreach (gmem[k]) $display("T|gen%0d key=%h bits=%0d", gi, k, $bits(k));
    end
  end
  initial #5 $finish;
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|p key=0000011 bits=26",
            "T|t key=123 bits=12",
            "T|s key=0c5 bits=10 a=3",
            "T|e key=5 bits=3",
            "T|str key=hello len=5",
            "T|i key=-5 bits=32",
            "T|b key=-3 bits=8",
            "T|sk key=-7 bits=16",
            "T|l key=0000001 bits=28",
            "T|sg key=-2 bits=26",
            "T|pi key=0000011 bits=26",
            "T|p key=011 bits=10",
            "T|t key=123 bits=12",
            "T|s key=0c5 bits=10 a=3",
            "T|e key=5 bits=3",
            "T|str key=hello len=5",
            "T|i key=-5 bits=32",
            "T|b key=-3 bits=8",
            "T|sk key=-7 bits=16",
            "T|l key=001 bits=12",
            "T|sg key=-2 bits=10",
            "T|pi key=011 bits=10",
            "T|top key=00005 bits=20",
            "T|gen0 key=03 bits=8",
            "T|gen1 key=003 bits=12",
        ],
    );
}

/// The same declarations in the top module.
#[test]
fn key_types_at_top_level() {
    const SRC: &str = r#"
typedef logic [11:0] k12_t;
typedef struct packed { logic [3:0] a; logic [5:0] b; } ks_t;
typedef enum logic [2:0] { E0, E1, E5 = 5 } ke_t;
typedef shortint sk_t;
module tb;
  localparam int KW = 26;

  typedef logic [KW+1:0] lk_t;
  logic [7:0] mem  [logic [KW-1:0]];
  logic [7:0] memt [k12_t];
  logic [7:0] mems [ks_t];
  logic [7:0] meme [ke_t];
  logic [7:0] memst[string];
  logic [7:0] memi [int];
  logic [7:0] memb [byte];
  logic [7:0] mems2[sk_t];
  logic [7:0] meml [lk_t];
  logic [7:0] memsg[logic signed [KW-1:0]];
  task report();
    foreach (mem[k])   $display("T|p key=%h bits=%0d", k, $bits(k));
    foreach (memt[k])  $display("T|t key=%h bits=%0d", k, $bits(k));
    foreach (mems[k])  $display("T|s key=%h bits=%0d a=%0d", k, $bits(k), k.a);
    foreach (meme[k])  $display("T|e key=%0d bits=%0d", k, $bits(k));
    foreach (memst[k]) $display("T|str key=%s len=%0d", k, k.len());
    foreach (memi[k])  $display("T|i key=%0d bits=%0d", k, $bits(k));
    foreach (memb[k])  $display("T|b key=%0d bits=%0d", k, $bits(k));
    foreach (mems2[k]) $display("T|sk key=%0d bits=%0d", k, $bits(k));
    foreach (meml[k])  $display("T|l key=%h bits=%0d", k, $bits(k));
    foreach (memsg[k]) $display("T|sg key=%0d bits=%0d", k, $bits(k));
  endtask
  initial begin
    mem[26'h11] = 8'hAA;
    memt[12'h123] = 1;
    mems['{a:4'h3, b:6'h5}] = 2;
    meme[E5] = 3;
    memst["hello"] = 4;
    memi[-5] = 5;
    memb[-3] = 6;
    mems2[-7] = 7;
    meml[28'h1] = 8;
    memsg[-2] = 9;
    #1 report();
    foreach (mem[k]) $display("T|pi key=%h bits=%0d", k, $bits(k));
  end
  initial #5 $finish;
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|p key=0000011 bits=26",
            "T|t key=123 bits=12",
            "T|s key=0c5 bits=10 a=3",
            "T|e key=5 bits=3",
            "T|str key=hello len=5",
            "T|i key=-5 bits=32",
            "T|b key=-3 bits=8",
            "T|sk key=-7 bits=16",
            "T|l key=0000001 bits=28",
            "T|sg key=-2 bits=26",
            "T|pi key=0000011 bits=26",
        ],
    );
}

/// A packed-struct key's members through the index variable, in a task and
/// in an initial block.
#[test]
fn packed_struct_key_members() {
    const SRC: &str = r#"
typedef struct packed { logic [3:0] a; logic [5:0] b; } ks_t;
module tb;
  logic [7:0] mems [ks_t];
  task report();
    ks_t lv;
    lv = 10'h0c5;
    $display("T|lv a=%0d b=%0d", lv.a, lv.b);
    foreach (mems[k]) $display("T|s key=%h a=%0d b=%0d", k, k.a, k.b);
  endtask
  initial begin
    mems['{a:4'h3, b:6'h5}] = 2;
    foreach (mems[k]) $display("T|i key=%h a=%0d b=%0d", k, k.a, k.b);
    report();
  end
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(
        t_lines(&sim),
        ["T|i key=0c5 a=3 b=5", "T|lv a=3 b=5", "T|s key=0c5 a=3 b=5",],
    );
}

/// Class properties: a typedef'd key resolved through the installed typedef
/// table (it fell back to 32 bits). A typedef'd SIGNED key on a class
/// property still iterates unsigned (no typedef signedness during class
/// elaboration), so it is left out here.
#[test]
fn key_types_of_class_properties() {
    const SRC: &str = r#"
typedef logic [11:0] k12_t;
typedef struct packed { logic [3:0] a; logic [5:0] b; } ks_t;
typedef enum logic [2:0] { E0, E1, E5 = 5 } ke_t;
typedef shortint sk_t;
class C #(int KW = 26);
  logic [7:0] mem  [logic [KW-1:0]];
  logic [7:0] memt [k12_t];
  logic [7:0] mems [ks_t];
  logic [7:0] meme [ke_t];
  logic [7:0] memb [byte];
  function void fill();
    mem[26'h11] = 1; memt[12'h123] = 1; mems['{a:4'h3, b:6'h5}] = 2; meme[E5] = 3; memb[-3] = 6;
  endfunction
  function void report();
    foreach (mem[k])   $display("T|p key=%h bits=%0d", k, $bits(k));
    foreach (memt[k])  $display("T|t key=%h bits=%0d", k, $bits(k));
    foreach (mems[k])  $display("T|s key=%h bits=%0d a=%0d", k, $bits(k), k.a);
    foreach (meme[k])  $display("T|e key=%0d bits=%0d", k, $bits(k));
    foreach (memb[k])  $display("T|b key=%0d bits=%0d", k, $bits(k));
  endfunction
endclass
module tb;
  initial begin
    C #(26) c;
    c = new;
    c.fill();
    c.report();
  end
endmodule"#;
    let sim = simulate(SRC, 100).expect("sim");
    assert_eq!(
        t_lines(&sim),
        [
            "T|p key=0000011 bits=26",
            "T|t key=123 bits=12",
            "T|s key=0c5 bits=10 a=3",
            "T|e key=5 bits=3",
            "T|b key=-3 bits=8",
        ],
    );
}
