//! Bytecode-compiled `for` loops in edge blocks (the For_init_vardecl /
//! For_step_other fallbacks were 83% of a customer run's wall time).
//! Register-backed loop vars, signal-backed `i++` steps, size-casts of the
//! loop var, and the still-AST self-reading counter shape.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

#[test]
fn register_var_loop_with_cast_matches_model() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [63:0] lanes [16];
  logic [63:0] src = 64'hdeadbeef01234567;
  logic [63:0] acc = 0;
  int cyc = 0;
  always #1 clk = ~clk;
  initial for (int k = 0; k < 16; k++) lanes[k] = 0;
  always @(posedge clk) begin
    for (int i = 0; i < 16; i++) begin
      lanes[i] <= src ^ (64'(i) << 8) ^ acc;
    end
    acc <= acc + lanes[cyc & 15];
    cyc <= cyc + 1;
  end
  initial #62 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    // Rust mirror of the NBA semantics.
    let mut lanes = [0u64; 16];
    let mut acc: u64 = 0;
    let srcv: u64 = 0xdead_beef_0123_4567;
    let n = u(&sim, "cyc");
    for c in 0..n {
        let old = lanes;
        let old_acc = acc;
        for i in 0..16u64 {
            lanes[i as usize] = srcv ^ (i << 8) ^ old_acc;
        }
        acc = old_acc.wrapping_add(old[(c & 15) as usize]);
    }
    assert_eq!(u(&sim, "acc"), acc, "after {} cycles", n);
}

#[test]
fn signal_var_incr_step_loop() {
    let src = r#"
module tb;
  logic clk = 0;
  int i;
  logic [31:0] sum = 0;
  int cyc = 0;
  always #1 clk = ~clk;
  always @(posedge clk) begin
    for (i = 0; i < 8; i++) sum <= sum + i; // last NBA wins: sum += 7
    cyc <= cyc + 1;
  end
  initial #22 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let n = u(&sim, "cyc");
    assert_eq!(u(&sim, "sum"), 7 * n, "one +7 per cycle (last NBA wins)");
}

#[test]
fn self_reading_counter_loop_still_correct() {
    // Excluded from register compilation (self-read gate) — must stay right.
    let src = r#"
module tb;
  logic clk = 0;
  logic [9:0] ptr [4];
  int cyc = 0;
  always #1 clk = ~clk;
  initial for (int k = 0; k < 4; k++) ptr[k] = 0;
  always @(posedge clk) begin
    for (int i = 0; i < 4; i++) ptr[i] <= ptr[i] + 1;
    cyc <= cyc + 1;
  end
  initial #22 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let n = u(&sim, "cyc");
    assert_eq!(u(&sim, "ptr[2]"), n, "each element counts every posedge");
}

#[test]
fn negative_bound_descending_loop_signed_compare() {
    // `i > -3` with a register-backed var: an unsigned step constant used
    // to strip the var's sign on the first i--, turning the compare
    // unsigned and exiting after one iteration.
    let src = r#"
module tb;
  logic clk = 0;
  logic signed [31:0] acc = 0;
  int cyc = 0;
  always #1 clk = ~clk;
  always @(posedge clk) begin
    for (int i = 2; i > -3; i--) acc <= acc + i; // last NBA: acc + (-2)
    cyc <= cyc + 1;
  end
  initial #12 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let n = u(&sim, "cyc") as i64;
    let acc = u(&sim, "acc") as u32 as i32 as i64;
    assert_eq!(
        acc,
        -2 * n,
        "descending loop crosses zero with signed compare"
    );
}

#[test]
fn loop_var_shadows_module_signal() {
    let src = r#"
module tb;
  logic clk = 0;
  int i = 777;
  logic [31:0] acc = 0;
  int cyc = 0;
  always #1 clk = ~clk;
  always @(posedge clk) begin
    for (int i = 0; i < 5; i++) acc <= acc + i;
    cyc <= cyc + 1;
  end
  initial #12 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "i"), 777, "outer signal untouched by the loop var");
    let n = u(&sim, "cyc");
    assert_eq!(u(&sim, "acc"), 4 * n, "last NBA wins: acc + 4 per cycle");
}

#[test]
fn nested_register_var_loops_and_stride() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [31:0] a = 0, b = 0;
  int cyc = 0;
  always #1 clk = ~clk;
  always @(posedge clk) begin
    for (int i = 0; i < 4; i++)
      for (int j = 0; j < 4; j++)
        a <= a + i * 4 + j;          // last NBA: a + 15
    for (byte k = 0; k < 10; k += 2) b <= b + k; // last NBA: b + 8
    cyc <= cyc + 1;
  end
  initial #12 $finish;
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let n = u(&sim, "cyc");
    assert_eq!(u(&sim, "a"), 15 * n);
    assert_eq!(u(&sim, "b"), 8 * n);
}

#[test]
fn full_range_packed_nba_copy_keeps_residual_body_and_range_order() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [3:0][7:0] src = 32'h1020_3040;
  logic [3:0][7:0] copied = 0;
  logic [3:0][7:0] mapped = 0;
  logic [3:0][7:0] identity_wins = 0;
  logic [3:0][7:0] compute_wins = 0;
  logic [0:3][7:0] ascending = 0;
  logic [3:0][7:0] reversed = 0;

  function automatic [7:0] mix(input [7:0] v);
    mix = v ^ 8'h5a;
  endfunction

  always #1 clk = ~clk;
  always @(posedge clk) begin
    for (int i = 0; i < 4; i++) begin
      copied[i] <= src[i];
      mapped[i] <= mix(src[i]);
      reversed[i] <= ascending[i];
    end
  end
  always @(posedge clk) begin
    for (int j = 0; j < 4; j++) begin
      identity_wins[j] <= mix(src[j]);
      identity_wins[j] <= src[j];
      compute_wins[j] <= src[j];
      compute_wins[j] <= mix(src[j]);
    end
  end
  initial begin
    ascending[0] = 8'h11;
    ascending[1] = 8'h22;
    ascending[2] = 8'h33;
    ascending[3] = 8'h44;
    #4 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "copied"), 0x1020_3040);
    assert_eq!(u(&sim, "mapped"), 0x4a7a_6a1a);
    assert_eq!(u(&sim, "identity_wins"), 0x1020_3040);
    assert_eq!(u(&sim, "compute_wins"), 0x4a7a_6a1a);
    assert_eq!(
        u(&sim, "reversed"),
        0x4433_2211,
        "opposite packed orientations must use element-wise lowering"
    );
}

#[test]
fn full_range_packed_blocking_fill_preserves_x_and_guarded_fallbacks() {
    let src = r#"
module tb;
  logic [2:0][3:0][1:0] filled;
  logic [3:0][7:0] partial;
  logic [3:0][7:0] carried;
  logic [2:0] checks = 0;
  initial begin
    filled = '0;
    for (int outer = 0; outer < 3; outer++) begin
      for (int lane = 0; lane < 4; lane++) begin
        filled[outer][lane] = 2'bx0;
      end
    end

    partial = '0;
    for (int i = 1; i < 3; i++) partial[i] = 8'ha0 + i;

    carried = '0;
    carried[0] = 1;
    for (int i = 0; i < 4; i++) carried[i] = carried[0] + 1;

    checks[0] = (filled === {12{2'bx0}});
    checks[1] = (partial === 32'h00a2_a100);
    checks[2] = (carried === 32'h0303_0302);
    #1 $finish;
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(u(&sim, "checks"), 0b111);
}

#[test]
fn packed_loop_fast_paths_respect_active_force() {
    let src = r#"
module tb;
  logic clk = 0;
  logic [3:0][7:0] sample_bus = 32'h1020_3040;
  logic [3:0][7:0] stage_bus = '0;
  logic [3:0][7:0] fill_bus = '0;
  logic [31:0] stage_seen = '0;
  logic [31:0] fill_seen = '0;
  logic [1:0] checks = 0;

  always #1 clk = ~clk;
  always @(posedge clk) begin
    for (int slot = 0; slot < 4; slot++)
      stage_bus[slot] <= sample_bus[slot];
  end

  initial begin
    force fill_bus = 32'h5aa5_6996;
    for (int slot = 0; slot < 4; slot++)
      fill_bus[slot] = 8'ha5;

    force stage_bus = 32'hc35a_9669;
    #4;
    fill_seen = fill_bus;
    stage_seen = stage_bus;
    checks[0] = (fill_seen === 32'h5aa5_6996);
    checks[1] = (stage_seen === 32'hc35a_9669);
    release fill_bus;
    release stage_bus;
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(
        u(&sim, "checks"),
        0b11,
        "fill={:08x} stage={:08x}",
        u(&sim, "fill_seen"),
        u(&sim, "stage_seen")
    );
}

#[test]
fn resumed_process_loop_keeps_locals_and_reference_updates() {
    let src = r#"
module loop_resume_probe;
  logic tick = 0;
  logic [3:0][15:0] bundle = '0;
  logic [15:0] state = 16'h1234;
  int completed = 0;

  always #1 tick = ~tick;

  function automatic logic [15:0] advance(ref logic [15:0] value);
    value = (value >> 1) ^ (-(value & 1'b1) & 16'hb400);
    return value;
  endfunction

  initial begin
    for (int pass = 0; pass < 7; pass++) begin
      @(negedge tick);
      for (int slot = 0; slot < 4; slot++) begin
        logic [15:0] item;
        item = advance(state);
        bundle[slot] <= item ^ pass;
      end
    end
    #1;
    completed = 1;
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");

    let mut state = 0x1234u16;
    let mut bundle = [0u16; 4];
    for pass in 0..7u16 {
        for item in &mut bundle {
            state = (state >> 1) ^ if state & 1 != 0 { 0xb400 } else { 0 };
            *item = state ^ pass;
        }
    }
    let expected = bundle
        .iter()
        .rev()
        .fold(0u64, |packed, item| (packed << 16) | u64::from(*item));
    assert_eq!(u(&sim, "completed"), 1);
    assert_eq!(u(&sim, "state"), u64::from(state));
    assert_eq!(u(&sim, "bundle"), expected);
}

#[test]
fn unrolled_loop_var_keeps_declared_signedness() {
    // #278: an unrolled `for` bound its loop variable as a 32-bit unsigned
    // constant, so `for (int i = 1; i > -1; i--)` and `for (int i = -2; i <
    // 2; i++)` ran zero times. §6.11/§11.8.1: `int` is signed and the compare
    // is signed; narrow loop variables wrap at their declared width.
    // Expected values are the reference simulator's.
    let sim =
        simulate(include_str!("unrolled_for_signed_bounds.sv"), 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|l| l.message.strip_prefix("T|"))
        .collect();
    let want = [
        "desc acc=3 cnt=2",
        "neg_start acc=2 cnt=4",
        "ge0 acc=10 cnt=4",
        "minus_eq acc=6 cnt=6",
        "plus_neg acc=6 cnt=4",
        "assign_sub acc=12 cnt=8",
        "uint_vs_neg acc=0 cnt=0",
        "ge0_from1 acc=3 cnt=2",
        "byte_wrap acc=381 cnt=3",
        "shortint acc=3 cnt=6",
        "longint acc=3 cnt=6",
        "integer acc=5 cnt=5",
        "bit3_wrap acc=24 cnt=5",
        "lsigned4 acc=0 cnt=7",
        "neg_val acc=-734",
        "lt0_body acc=203",
        "fdesc 23 222 6",
    ];
    assert_eq!(got, want);
}
