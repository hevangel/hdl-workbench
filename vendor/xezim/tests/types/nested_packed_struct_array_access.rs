//! §7.4.1/§7.4.2 — selects into a packed struct array nested INSIDE another
//! packed struct, and into an unpacked array of packed struct arrays.
//! Reference-validated.
//!
//! `arr[0].wdata[0]` worked and `n.wdata[0]` worked, but `n.wdata[0].wdata[0]`
//! read 0: the resolver matched a fixed "index chain, then field path" shape,
//! while a nested path ALTERNATES (field, index, field, index). It resolved the
//! first group and gave up. `n.wdata[0].amask` — a nested member with no
//! trailing index — fell all the way through the member arm to the class-handle
//! fallback and returned `zero(32)`.
//!
//! Both the AST interpreter and the bytecode VM were wrong in the same way:
//! the VM bails on `MemberAccess` reads, so those blocks run interpreted.
//!
//! Elaboration already registered the indexed keys (`wdata[0].wdata` at offset
//! 18) — what was missing was the nested member's element STRIDE, since member
//! dims were registered only one level deep.
//!
//! The payload is asymmetric in every field so a misaligned or truncated slice
//! cannot alias a correct one.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

const SRC: &str = r#"
package P;
  typedef struct packed {
    logic [1:0][63:0] wdata;
    logic [1:0][7:0]  mask;
    logic [1:0]       amask;
  } s_t;                                   // 146
  typedef struct packed { s_t [1:0] wdata; } n_t;   // 292
endpackage

module tb;
  P::s_t [1:0] arr;      // packed array of structs      (was already correct)
  P::n_t       n;        // struct CONTAINING a packed array of structs
  P::n_t [0:0] na;       // array of those
  P::s_t [1:0] ua [0:1]; // UNPACKED array of packed struct arrays

  localparam logic [145:0] E0 =
    {64'hAAAA_AAAA_AAAA_AAA1, 64'hBBBB_BBBB_BBBB_BBB0, 8'hC1, 8'hC0, 1'b1, 1'b0};
  localparam logic [145:0] E1 =
    {64'hDDDD_DDDD_DDDD_DDD1, 64'hEEEE_EEEE_EEEE_EEE0, 8'hF1, 8'hF0, 1'b0, 1'b1};

  // Bytecode VM path.
  logic [63:0] b_arr, b_n, b_na, b_ua, b_n_hi;
  logic [7:0]  b_mask;
  logic [1:0]  b_amask, b_amask2;
  assign b_arr   = arr[0].wdata[0];
  assign b_n     = n.wdata[0].wdata[0];
  assign b_n_hi  = n.wdata[1].wdata[1];
  assign b_na    = na[0].wdata[0].wdata[0];
  assign b_ua    = ua[1][0].wdata[0];
  assign b_mask  = n.wdata[1].mask[1];
  // Nested member with NO trailing index, under a leading array index — the
  // shape that fell through to the class-handle fallback. Without the leading
  // index (`n.wdata[0].amask`) an existing path already resolved it, so that
  // form is kept below only as a did-not-regress check.
  assign b_amask  = na[0].wdata[0].amask;
  assign b_amask2 = n.wdata[0].amask;

  // AST interpreter path.
  logic [63:0] a_n, a_n_hi, a_na, a_ua;
  logic [1:0]  a_amask, a_amask2;
  // Writes THROUGH the nested accessor.
  P::n_t       wn;
  P::s_t [1:0] wua [0:1];
  logic [63:0] w_lo, w_hi, w_ua;
  logic [7:0]  w_mask;

  initial begin
    arr = {E1, E0};  n = {E1, E0};  na = {E1, E0};  ua[1] = {E1, E0};
    #1;
    a_n     = n.wdata[0].wdata[0];
    a_n_hi  = n.wdata[1].wdata[1];
    a_na    = na[0].wdata[0].wdata[0];
    a_ua    = ua[1][0].wdata[0];
    a_amask  = na[0].wdata[0].amask;
    a_amask2 = n.wdata[0].amask;

    wn = '0;  wua[1] = '0;
    wn.wdata[0].wdata[0] = 64'h1111_1111_1111_1110;
    wn.wdata[1].wdata[1] = 64'h2222_2222_2222_2221;
    wn.wdata[1].mask[1]  = 8'h33;
    wua[1][0].wdata[1]   = 64'h5555_5555_5555_5551;
    #1;
    w_lo   = wn.wdata[0].wdata[0];
    w_hi   = wn.wdata[1].wdata[1];
    w_mask = wn.wdata[1].mask[1];
    w_ua   = wua[1][0].wdata[1];
  end
endmodule
"#;

#[test]
fn single_level_packed_struct_array_still_resolves() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "b_arr"), 0xBBBB_BBBB_BBBB_BBB0);
}

#[test]
fn nested_struct_array_element_reads_on_the_bytecode_path() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "b_n"), 0xBBBB_BBBB_BBBB_BBB0, "n.wdata[0].wdata[0]");
    assert_eq!(
        u(&sim, "b_n_hi"),
        0xDDDD_DDDD_DDDD_DDD1,
        "n.wdata[1].wdata[1]"
    );
    assert_eq!(
        u(&sim, "b_na"),
        0xBBBB_BBBB_BBBB_BBB0,
        "na[0].wdata[0].wdata[0]"
    );
    assert_eq!(
        u(&sim, "b_ua"),
        0xBBBB_BBBB_BBBB_BBB0,
        "unpacked ua[1][0].wdata[0]"
    );
    assert_eq!(u(&sim, "b_mask"), 0xF1, "narrow nested member element");
}

#[test]
fn nested_struct_array_element_reads_on_the_ast_path() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "a_n"), 0xBBBB_BBBB_BBBB_BBB0);
    assert_eq!(u(&sim, "a_n_hi"), 0xDDDD_DDDD_DDDD_DDD1);
    assert_eq!(u(&sim, "a_na"), 0xBBBB_BBBB_BBBB_BBB0);
    assert_eq!(u(&sim, "a_ua"), 0xBBBB_BBBB_BBBB_BBB0);
}

#[test]
fn nested_member_without_a_trailing_index_is_not_zero32() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // amask of element [0] is 2'b10 — this returned a 32-bit zero.
    assert_eq!(u(&sim, "b_amask"), 0b10, "na[0].wdata[0].amask, bytecode");
    assert_eq!(u(&sim, "a_amask"), 0b10, "na[0].wdata[0].amask, AST");
    assert_eq!(u(&sim, "b_amask2"), 0b10, "n.wdata[0].amask still resolves");
    assert_eq!(u(&sim, "a_amask2"), 0b10, "n.wdata[0].amask still resolves");
}

#[test]
fn writes_through_the_nested_accessor_land_in_the_right_slice() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "w_lo"), 0x1111_1111_1111_1110);
    assert_eq!(u(&sim, "w_hi"), 0x2222_2222_2222_2221);
    assert_eq!(u(&sim, "w_mask"), 0x33);
    assert_eq!(u(&sim, "w_ua"), 0x5555_5555_5555_5551, "unpacked write");
}
