//! §7.4.1 label mapping on selects: a declared dimension that does not
//! start at zero, or runs ascending, addresses different physical bits than
//! its labels suggest. Those selects used to be handed to the AST
//! interpreter, which demoted any loop holding one; they compile now, and
//! every expected value here is the reference simulator's.
use std::path::PathBuf;
use std::process::Command;

fn run(name: &str, src: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("select_label_mapping");
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join(format!("{name}.sv"));
    std::fs::write(&sv, src).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "-s", "t", "--no-cache", sv.to_str().unwrap()])
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "run failed:\n{text}");
    text
}

/// Every select form (bit, constant range, `+:`, `-:`, on a plain vector, a
/// packed element and an unpacked-array element) against every declared
/// direction: descending from zero, descending from one, ascending from
/// zero, ascending from one.
#[test]
fn every_select_form_against_every_declared_direction() {
    let text = run(
        "matrix",
        r#"
// Differential matrix: every select form against every declared direction.
module t;
  logic clk = 0; always #5 clk = ~clk;
  logic [1:0] i2 = 2;
  logic [3:0] idx, idxh;
  logic [7:0] v_d0;
  logic [3:0][7:0] pm_d0;
  logic [7:0] um_d0 [0:3];
  logic r_d0_b;
  logic r_d0_bd;
  logic r_d0_pb;
  logic r_d0_ub;
  logic [2:0] r_d0_c;
  logic [2:0] r_d0_pc;
  logic [2:0] r_d0_uc;
  logic [2:0] r_d0_iu;
  logic [2:0] r_d0_id;
  logic [2:0] r_d0_pu;
  logic [2:0] r_d0_pd;
  logic [2:0] r_d0_uu;
  logic [8:1] v_dn;
  logic [3:0][8:1] pm_dn;
  logic [8:1] um_dn [0:3];
  logic r_dn_b;
  logic r_dn_bd;
  logic r_dn_pb;
  logic r_dn_ub;
  logic [2:0] r_dn_c;
  logic [2:0] r_dn_pc;
  logic [2:0] r_dn_uc;
  logic [2:0] r_dn_iu;
  logic [2:0] r_dn_id;
  logic [2:0] r_dn_pu;
  logic [2:0] r_dn_pd;
  logic [2:0] r_dn_uu;
  logic [0:7] v_a0;
  logic [3:0][0:7] pm_a0;
  logic [0:7] um_a0 [0:3];
  logic r_a0_b;
  logic r_a0_bd;
  logic r_a0_pb;
  logic r_a0_ub;
  logic [2:0] r_a0_c;
  logic [2:0] r_a0_pc;
  logic [2:0] r_a0_uc;
  logic [2:0] r_a0_iu;
  logic [2:0] r_a0_id;
  logic [2:0] r_a0_pu;
  logic [2:0] r_a0_pd;
  logic [2:0] r_a0_uu;
  logic [1:8] v_an;
  logic [3:0][1:8] pm_an;
  logic [1:8] um_an [0:3];
  logic r_an_b;
  logic r_an_bd;
  logic r_an_pb;
  logic r_an_ub;
  logic [2:0] r_an_c;
  logic [2:0] r_an_pc;
  logic [2:0] r_an_uc;
  logic [2:0] r_an_iu;
  logic [2:0] r_an_id;
  logic [2:0] r_an_pu;
  logic [2:0] r_an_pd;
  logic [2:0] r_an_uu;
  always @(posedge clk) begin
    r_d0_b  <= v_d0[3];
    r_d0_bd <= v_d0[idx];
    r_d0_pb <= pm_d0[i2][3];
    r_d0_ub <= um_d0[i2][idx];
    r_d0_c  <= v_d0[5:3];
    r_d0_pc <= pm_d0[i2][5:3];
    r_d0_uc <= um_d0[i2][5:3];
    r_d0_iu <= v_d0[0 +: 3];
    r_d0_id <= v_d0[7 -: 3];
    r_d0_pu <= pm_d0[i2][idx +: 3];
    r_d0_pd <= pm_d0[i2][idxh -: 3];
    r_d0_uu <= um_d0[i2][idx +: 3];
    r_dn_b  <= v_dn[4];
    r_dn_bd <= v_dn[idx];
    r_dn_pb <= pm_dn[i2][4];
    r_dn_ub <= um_dn[i2][idx];
    r_dn_c  <= v_dn[6:4];
    r_dn_pc <= pm_dn[i2][6:4];
    r_dn_uc <= um_dn[i2][6:4];
    r_dn_iu <= v_dn[1 +: 3];
    r_dn_id <= v_dn[8 -: 3];
    r_dn_pu <= pm_dn[i2][idx +: 3];
    r_dn_pd <= pm_dn[i2][idxh -: 3];
    r_dn_uu <= um_dn[i2][idx +: 3];
    r_a0_b  <= v_a0[3];
    r_a0_bd <= v_a0[idx];
    r_a0_pb <= pm_a0[i2][3];
    r_a0_ub <= um_a0[i2][idx];
    r_a0_c  <= v_a0[3:5];
    r_a0_pc <= pm_a0[i2][3:5];
    r_a0_uc <= um_a0[i2][3:5];
    r_a0_iu <= v_a0[0 +: 3];
    r_a0_id <= v_a0[7 -: 3];
    r_a0_pu <= pm_a0[i2][idx +: 3];
    r_a0_pd <= pm_a0[i2][idxh -: 3];
    r_a0_uu <= um_a0[i2][idx +: 3];
    r_an_b  <= v_an[4];
    r_an_bd <= v_an[idx];
    r_an_pb <= pm_an[i2][4];
    r_an_ub <= um_an[i2][idx];
    r_an_c  <= v_an[4:6];
    r_an_pc <= pm_an[i2][4:6];
    r_an_uc <= um_an[i2][4:6];
    r_an_iu <= v_an[1 +: 3];
    r_an_id <= v_an[8 -: 3];
    r_an_pu <= pm_an[i2][idx +: 3];
    r_an_pd <= pm_an[i2][idxh -: 3];
    r_an_uu <= um_an[i2][idx +: 3];
  end
  initial begin
    v_d0 = 8'h5a;
    for (int k = 0; k < 4; k++) begin pm_d0[k] = 8'(k*8'h33+7); um_d0[k] = 8'(k*8'h29+11); end
    v_dn = 8'h5a;
    for (int k = 0; k < 4; k++) begin pm_dn[k] = 8'(k*8'h33+7); um_dn[k] = 8'(k*8'h29+11); end
    v_a0 = 8'h5a;
    for (int k = 0; k < 4; k++) begin pm_a0[k] = 8'(k*8'h33+7); um_a0[k] = 8'(k*8'h29+11); end
    v_an = 8'h5a;
    for (int k = 0; k < 4; k++) begin pm_an[k] = 8'(k*8'h33+7); um_an[k] = 8'(k*8'h29+11); end
    idx = 4'd1; idxh = 4'd6;
    repeat (3) @(posedge clk);
    #1;
    $display("d0 b=%b bd=%b pb=%b ub=%b c=%b pc=%b uc=%b iu=%b id=%b pu=%b pd=%b uu=%b", r_d0_b, r_d0_bd, r_d0_pb, r_d0_ub, r_d0_c, r_d0_pc, r_d0_uc, r_d0_iu, r_d0_id, r_d0_pu, r_d0_pd, r_d0_uu);
    $display("dn b=%b bd=%b pb=%b ub=%b c=%b pc=%b uc=%b iu=%b id=%b pu=%b pd=%b uu=%b", r_dn_b, r_dn_bd, r_dn_pb, r_dn_ub, r_dn_c, r_dn_pc, r_dn_uc, r_dn_iu, r_dn_id, r_dn_pu, r_dn_pd, r_dn_uu);
    $display("a0 b=%b bd=%b pb=%b ub=%b c=%b pc=%b uc=%b iu=%b id=%b pu=%b pd=%b uu=%b", r_a0_b, r_a0_bd, r_a0_pb, r_a0_ub, r_a0_c, r_a0_pc, r_a0_uc, r_a0_iu, r_a0_id, r_a0_pu, r_a0_pd, r_a0_uu);
    $display("an b=%b bd=%b pb=%b ub=%b c=%b pc=%b uc=%b iu=%b id=%b pu=%b pd=%b uu=%b", r_an_b, r_an_bd, r_an_pb, r_an_ub, r_an_c, r_an_pc, r_an_uc, r_an_iu, r_an_id, r_an_pu, r_an_pd, r_an_uu);
    $finish;
  end
endmodule
"#,
    );
    assert!(
        text.contains(
            "d0 b=1 bd=1 pb=1 ub=0 c=011 pc=101 uc=011 iu=010 id=010 pu=110 pd=110 uu=110"
        ),
        "wrong values:\n{text}"
    );
    assert!(
        text.contains(
            "dn b=1 bd=0 pb=1 ub=1 c=011 pc=101 uc=011 iu=010 id=010 pu=101 pd=101 uu=101"
        ),
        "wrong values:\n{text}"
    );
    assert!(
        text.contains(
            "a0 b=1 bd=1 pb=0 ub=1 c=110 pc=011 uc=111 iu=010 id=010 pu=110 pd=110 uu=101"
        ),
        "wrong values:\n{text}"
    );
    assert!(
        text.contains(
            "an b=1 bd=0 pb=0 ub=0 c=110 pc=011 uc=111 iu=010 id=010 pu=011 pd=011 uu=010"
        ),
        "wrong values:\n{text}"
    );
}

/// The same shapes inside a clocked block, where a bail would take the whole
/// block to the interpreter. `XEZIM_FALLBACK_SITES` reports what still does.
#[test]
fn label_mapped_selects_compile() {
    let src = r#"
// Part-selects whose base is an ELEMENT with a non-zero-based or ascending
// declared dimension - the shapes the compiler hands to the AST interpreter.
module t;
  logic clk = 0; always #5 clk = ~clk;
  logic [7:0][8:1] pm;          // packed 2-D, element dim [8:1]
  logic [8:1]      um [0:15];   // unpacked array, element dim [8:1]
  logic [7:0][0:7] am;          // packed 2-D, element dim ASCENDING
  logic [0:31]     av;          // plain ascending vector
  logic [3:0] i4 = 2;
  logic [2:0] a, b, c, d, e, f;
  int cyc = 0;
  always @(posedge clk) begin
    cyc <= cyc + 1;
    a <= pm[i4][4:2];            // const range, non-zero-based element dim
    b <= um[i4][4:2];            // same, unpacked array element
    c <= am[i4][2:4];            // const range, ascending element dim
    d <= av[5:7];                // const range, ascending plain vector
    e <= pm[i4][2 +: 3];         // indexed-up on a non-zero-based element
    f <= am[i4][2 +: 3];         // indexed-up on an ascending element
  end
  initial begin
    for (int k = 0; k < 8; k++) begin pm[k] = 8'(k*8'h13+1); am[k] = 8'(k*8'h17+3); end
    foreach (um[k]) um[k] = 8'(k*8'h11+5);
    av = 32'hdead_beef;
    repeat (3) @(posedge clk);
    #1 $display("LBL a=%h b=%h c=%h d=%h e=%h f=%h", a, b, c, d, e, f);
    $finish;
  end
endmodule
"#;
    let text = run("labels", src);
    assert!(
        text.contains("LBL a=3 b=3 c=6 d=6 e=3 f=6"),
        "wrong values:\n{text}"
    );

    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("select_label_mapping");
    let sv = dir.join("labels.sv");
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "-s", "t", "--no-cache", sv.to_str().unwrap()])
        .env("XEZIM_FALLBACK_SITES", "1")
        .output()
        .unwrap();
    let trace = String::from_utf8_lossy(&out.stderr).to_string();
    // Only the unpacked-array element keeps the interpreter path.
    let sites: Vec<&str> = trace
        .lines()
        .filter(|l| l.starts_with("[FALLBACK]"))
        .filter(|l| !l.contains("Expr_display") && !l.contains("Expr_finish"))
        .collect();
    assert_eq!(
        sites.len(),
        1,
        "unexpected fallbacks:\n{}",
        sites.join("\n")
    );
}
