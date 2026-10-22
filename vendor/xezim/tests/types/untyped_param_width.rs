//! §6.20.2 — a parameter with no type and no range takes the type of its
//! final value: the SELF-DETERMINED width of its initializer (§11.6.1).
//! Only a sized literal was honoured, so every other untyped parameter came
//! out 32 bits — a concatenation, replication, part-select, comparison,
//! sized arithmetic, a string literal, or a reference to another sized
//! parameter — and a replication or string wider than 32 bits lost its high
//! part (`{4{B}}` read 0xa, `"hello"` read "ello"). Module body, header,
//! package and sub-instance parameters; the expected lines are the
//! reference simulator's output for these sources (cross-checked against it).

const SRC: &str = r#"package pk;
  localparam PA = 5;
  localparam PB = 4'b1010;
  localparam PC = {8'h1, 8'h2};
endpackage
module sub #(parameter SP = 3, parameter SQ = 8'd7) ();
  localparam SL = SP * 2;
  localparam SM = SQ + 1;
  localparam SN = {SQ, 2'b01};
  initial $display("sub SP=%0d SQ=%0d SL=%0d SM=%0d SN=%0d", $bits(SP), $bits(SQ), $bits(SL), $bits(SM), $bits(SN));
endmodule
module tb #(parameter HP = {4'h3, 4'h4}, parameter HQ = 7) ();
  localparam A = 5;
  localparam B = 4'b1010;
  localparam C = {8'h1, 8'h2};
  localparam D = B + 1;
  localparam F = "hello";
  localparam G = 64'hFFFF_FFFF_FFFF;
  localparam H = A * B;
  localparam I = B == 4'b1010;
  localparam J = {4{B}};
  localparam K = B[2:0];
  localparam L = 3'd5 + 5'd1;
  localparam M = -B;
  localparam N = $clog2(1000);
  localparam O = 'hFF;
  localparam P = 40'd3 << 2;
  localparam Q = C;
  localparam R = B ? 6'd1 : 2'd0;
  parameter  S = 16'd9;
  localparam T = S;
  localparam U = 1'b1;
  localparam V = 'b1;
  localparam W = 2**40;
  localparam X = pk::PC;
  localparam Y = ~S;
  localparam Z = B[1 +: 2];
  sub u1();
  sub #(.SP(4'd2), .SQ(12'd9)) u2();
  initial begin
    $display("A=%0d B=%0d C=%0d D=%0d F=%0d G=%0d H=%0d I=%0d J=%0d K=%0d L=%0d",
             $bits(A), $bits(B), $bits(C), $bits(D), $bits(F), $bits(G), $bits(H), $bits(I), $bits(J), $bits(K), $bits(L));
    $display("M=%0d N=%0d O=%0d P=%0d Q=%0d R=%0d S=%0d T=%0d U=%0d V=%0d W=%0d X=%0d Y=%0d Z=%0d",
             $bits(M), $bits(N), $bits(O), $bits(P), $bits(Q), $bits(R), $bits(S), $bits(T), $bits(U), $bits(V), $bits(W), $bits(X), $bits(Y), $bits(Z));
    $display("pk PA=%0d PB=%0d PC=%0d HP=%0d HQ=%0d", $bits(pk::PA), $bits(pk::PB), $bits(pk::PC), $bits(HP), $bits(HQ));
    $display("vals C=%h F=%s J=%h L=%0d M=%b P=%0d Y=%h HP=%h Z=%b", C, F, J, L, M, P, Y, HP, Z);
  end
endmodule
"#;

#[test]
fn untyped_parameters_take_the_self_determined_width() {
    let out: Vec<String> = xezim::simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .collect();
    for want in [
        "sub SP=32 SQ=8 SL=32 SM=32 SN=10",
        "sub SP=4 SQ=12 SL=32 SM=32 SN=14",
        "A=32 B=4 C=16 D=32 F=40 G=64 H=32 I=1 J=16 K=3 L=5",
        "M=4 N=32 O=32 P=40 Q=16 R=6 S=16 T=16 U=1 V=32 W=32 X=16 Y=16 Z=2",
        "pk PA=32 PB=4 PC=16 HP=8 HQ=32",
        "vals C=0102 F=hello J=aaaa L=6 M=0110 P=12 Y=fff6 HP=34 Z=01",
    ] {
        assert!(out.iter().any(|l| l == want), "missing `{want}`: {out:?}");
    }
}

/// The signedness comes with the width (§11.8.1): an expression over an
/// unsigned operand is unsigned, a comparison or concatenation is unsigned,
/// an unsized based literal is unsigned, and an explicit `unsigned` wins —
/// on body, header and generate-scope parameters. Kept signed, the narrowed
/// `UP + SP` read -6 instead of 250.
const SIGN_SRC: &str = r#"module gen_m;
  generate if (1) begin : g
    localparam GN = -4'sd1;
    localparam GU = 4'd0 - 1'b1;
    initial $display("gen GN<0=%0d GU=%0d bits=%0d/%0d", GN < 0, GU, $bits(GN), $bits(GU));
  end endgenerate
endmodule
module tb #(parameter HN = -4'sd1, parameter unsigned HU = 5, parameter HS = 4'd9 + 4'sd1) ();
  parameter        [7:0] UP = -3;
  parameter signed [7:0] SP = -3;
  localparam MIXED = UP + SP;
  localparam SS    = SP * SP;
  localparam X     = 4'd0 - 1;
  localparam Y     = -4'sd1;
  localparam O     = 'hFF;
  localparam CMP   = UP > 4'd3;
  localparam NEGC  = -{8'h1, 8'h2};
  localparam SH    = SP >>> 1;
  localparam A     = 5;
  localparam AN    = A - 10;
  localparam signed SX = 4'd15;
  localparam unsigned UX = -4'sd1;
  gen_m u_g();
  initial begin
    $display("MIXED=%0d SS=%0d X<0=%0d Y<0=%0d O>-1=%0d CMP=%0d NEGC=%0d SH=%0d AN<0=%0d",
             MIXED, SS, X < 0, Y < 0, O > -1, CMP, NEGC, SH, AN < 0);
    $display("SX=%0d UX=%0d HN<0=%0d HU>-1=%0d HS=%0d HSbits=%0d", SX, UX, HN < 0, HU > -1, HS, $bits(HS));
  end
endmodule
"#;

#[test]
fn untyped_parameters_take_the_signedness_of_their_value() {
    let out: Vec<String> = xezim::simulate(SIGN_SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .collect();
    for want in [
        "gen GN<0=1 GU=15 bits=4/4",
        "MIXED=250 SS=9 X<0=0 Y<0=1 O>-1=0 CMP=1 NEGC=65278 SH=-2 AN<0=1",
        "SX=-1 UX=15 HN<0=1 HU>-1=0 HS=10 HSbits=4",
    ] {
        assert!(out.iter().any(|l| l == want), "missing `{want}`: {out:?}");
    }
}
