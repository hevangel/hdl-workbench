
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH

`define SVTEST_INIT \
int failures = 0;

`define SVTEST_CHECK(expr, msg) \
if (!(expr)) begin \
  failures++; \
  $display("FAIL @%0t : %s", $time, msg); \
end

`define SVTEST_PASSFAIL \
if (failures == 0) begin \
  $display("TEST_PASS"); \
end else begin \
  $display("TEST_FAIL count=%0d", failures); \
  $fatal(1); \
end

`endif

// ============================================================================
//
//   scope            root cause   construct used here
//   ---------------------------------------------------------------------
//   package          XZ-1         integ_pkg::CH_W / DEPTH element selects
//   class            XZ-2         cfg_c::MASK via static fn / qualified / field
//   parameterised C  XZ-3         stream_c#(W, RATES[]) specialisation qualifier
//   child module     XZ-4         chan_c body localparam arrays, hierarchical
//   interface        XZ-5         dma_if body localparam array, hierarchical
//   parser           XZ-6         pkg array element as an UNPACKED DIMENSION
//
// LRM 6.18 / 6.19 / 6.20.2 / 8.3.2 / 8.16 / 8.25 / 23.11 / 25.2.1.
//
// ============================================================================
package integ_pkg;
  // ---- XZ-1: package-scope array constants ---------------------------------
  localparam int unsigned CH_W      [4] = '{8, 16, 32, 64};
  localparam int unsigned DEPTH     [4] = '{2, 4, 8, 16};
  localparam int unsigned NCHAN         = 4;                  // ctl scalar
  localparam int unsigned SIGNATURE     = 32'h0A5A_15A5;      // ctl scalar

  // ---- XZ-2: class-scope array localparam ----------------------------------
  class cfg_c;
    localparam int unsigned MASK [3] = '{8'h0F, 8'hF0, 8'hA5};
    localparam int unsigned TAG      = 32'h0BAD;              // ctl scalar
    function int unsigned mask(int i);  return MASK[i]; endfunction      // ctl path
    static function int unsigned smask(int i); return MASK[i]; endfunction
    function int unsigned qmask(int i);  return cfg_c::MASK[i]; endfunction
    int unsigned f_mask = MASK[1];
    int unsigned f_chw  = CH_W[2];        // XZ-1 x XZ-2: field from pkg element
  endclass

  // ---- XZ-3: parameterised class with an ARRAY value parameter -------------
  class stream_c #(int unsigned W = 32, int unsigned RATES [4] = '{1, 2, 4, 8});
    localparam int unsigned STEPS [4] = '{W, W/2, W/4, W/8};
    static function int unsigned step(int i); return STEPS[i]; endfunction
    static function int unsigned rate(int i); return RATES[i]; endfunction
  endclass
endpackage

// ---- XZ-5: interface-scope array localparam --------------------------------
interface dma_if;
  localparam int unsigned BURST [3] = '{1, 4, 16};
  localparam int unsigned WIDTH     = 32;                     // ctl scalar
  int unsigned fifo [integ_pkg::DEPTH[3]];   // XZ-6 x XZ-5: 16 entries
endinterface

// ---- XZ-4: child-module body array localparam ------------------------------
module chan_c #(parameter int unsigned K = 0);
  localparam int unsigned LA     [3] = '{7, 8, 9};
  localparam int unsigned SEL    [3] = '{2, 0, 1};            // usable as indices
  localparam int unsigned SCALAR     = 42;                    // ctl scalar
  int unsigned hist [integ_pkg::DEPTH[2]];   // XZ-6 x XZ-4: 8 entries
endmodule

module c99_tb;
  import integ_pkg::*;
  `SVTEST_INIT

  // ---- XZ-6: unpacked dimensions taken from package array elements ---------
  int unsigned mem     [integ_pkg::DEPTH[3]];   // 16 - PARSE ERROR today
  int unsigned ctl_mem [integ_pkg::NCHAN];      //  4 - parses today (ctl)

  int unsigned tmp [4];
  int unsigned s;

  chan_c u_chan ();
  dma_if u_dma  ();

  cfg_c o = new;

  initial begin
    // ================= XZ-1 : package array constants =================
    `SVTEST_CHECK(CH_W[0] == 8,  "c99 XZ-1 imported element CH_W[0]")
    `SVTEST_CHECK(integ_pkg::CH_W[3] == 64, "c99 XZ-1 qualified element CH_W[3]")
    `SVTEST_CHECK(integ_pkg::DEPTH[1] == 4, "c99 XZ-1 qualified element DEPTH[1]")
    `SVTEST_CHECK(NCHAN == 4,          "c99 XZ-1 scalar ctl NCHAN")
    `SVTEST_CHECK(SIGNATURE == 32'h0A5A_15A5, "c99 XZ-1 scalar ctl SIGNATURE")
    `SVTEST_CHECK($size(CH_W) == 4,    "c99 XZ-1 $size")
    `SVTEST_CHECK($bits(CH_W) == 128,  "c99 XZ-1 $bits")
    s = 0; foreach (CH_W[k]) s += CH_W[k];
    `SVTEST_CHECK(s == 120,            "c99 XZ-1 foreach sum")
    `SVTEST_CHECK(32 inside {CH_W},    "c99 XZ-1 inside hit")
    tmp = CH_W;
    `SVTEST_CHECK(tmp[2] == 32,        "c99 XZ-1 whole-array copy")
    // ================= XZ-2 : class array localparam ===================
    `SVTEST_CHECK(o.mask(1) == 8'hF0,  "c99 XZ-2 instance fn, bare name (ctl)")
    `SVTEST_CHECK(cfg_c::TAG == 32'h0BAD, "c99 XZ-2 class scalar ctl")
    `SVTEST_CHECK(integ_pkg::cfg_c::smask(1) == 8'hF0,
                  "c99 XZ-2 static fn, bare name")
    `SVTEST_CHECK(o.qmask(2) == 8'hA5, "c99 XZ-2 instance fn, qualified name")
    `SVTEST_CHECK(o.f_mask == 8'hF0,   "c99 XZ-2 field initializer")
    `SVTEST_CHECK(integ_pkg::cfg_c::MASK[2] == 8'hA5,
                  "c99 XZ-2 qualified element")
    `SVTEST_CHECK($size(integ_pkg::cfg_c::MASK) == 3, "c99 XZ-2 $size")
    s = 0; foreach (integ_pkg::cfg_c::MASK[k]) s += integ_pkg::cfg_c::MASK[k];
    `SVTEST_CHECK(s == 420,            "c99 XZ-2 qualified foreach sum")
    `SVTEST_CHECK(o.f_chw == 32,       "c99 XZ-2 x XZ-1 field from pkg element")

    // ============ XZ-3 : parameterised class, array value param =========
    `SVTEST_CHECK(integ_pkg::stream_c#(32, '{1,2,4,8})::STEPS[3] == 4,
                  "c99 XZ-3 spec-qualified array localparam")
    `SVTEST_CHECK(integ_pkg::stream_c#(32, '{1,2,4,8})::step(3) == 4,
                  "c99 XZ-3 spec-qualified static fn")
    `SVTEST_CHECK(integ_pkg::stream_c#()::STEPS[1] == 16,
                  "c99 XZ-3 default-spec array localparam")
    `SVTEST_CHECK(integ_pkg::stream_c#(32, '{9,8,7,6})::rate(3) == 6,
                  "c99 XZ-3 array value parameter")
    `SVTEST_CHECK(integ_pkg::stream_c#()::rate(2) == 4,
                  "c99 XZ-3 default array value parameter")

    // ================ XZ-4 : child module array localparam =============
    `SVTEST_CHECK(u_chan.SCALAR == 42, "c99 XZ-4 child scalar ctl")
    `SVTEST_CHECK(u_chan.LA[1] == 8,   "c99 XZ-4 child element LA[1]")
    `SVTEST_CHECK(u_chan.LA[2] == 9,   "c99 XZ-4 child element LA[2]")
    `SVTEST_CHECK($size(u_chan.LA) == 3, "c99 XZ-4 child $size")
    s = 0; foreach (u_chan.LA[k]) s += u_chan.LA[k];
    `SVTEST_CHECK(s == 24,             "c99 XZ-4 child foreach sum")
    `SVTEST_CHECK($size(u_chan.hist) == 8, "c99 XZ-4 x XZ-6 child memory depth")
    // ================ XZ-5 : interface array localparam ================
    `SVTEST_CHECK(u_dma.WIDTH == 32,   "c99 XZ-5 interface scalar ctl")
    `SVTEST_CHECK(u_dma.BURST[0] == 1, "c99 XZ-5 interface element [0]")
    `SVTEST_CHECK(u_dma.BURST[1] == 4, "c99 XZ-5 interface element [1]")
    `SVTEST_CHECK(u_dma.BURST[2] == 16,"c99 XZ-5 interface element [2]")
    `SVTEST_CHECK($size(u_dma.BURST) == 3, "c99 XZ-5 interface $size")
    s = 0; foreach (u_dma.BURST[k]) s += u_dma.BURST[k];
    `SVTEST_CHECK(s == 21,             "c99 XZ-5 interface foreach sum")
    `SVTEST_CHECK($size(u_dma.fifo) == 16, "c99 XZ-5 x XZ-6 interface fifo depth")

    // ============ XZ-6 : pkg array element as unpacked dimension ========
    `SVTEST_CHECK($size(ctl_mem) == 4,  "c99 XZ-6 scoped-scalar dimension ctl")
    `SVTEST_CHECK($size(mem) == 16,     "c99 XZ-6 dimension from pkg element")
    mem[15] = 32'hDEAD_BEEF;
    `SVTEST_CHECK(mem[15] == 32'hDEAD_BEEF, "c99 XZ-6 write/read that array")

    // ============ CROSS-SCOPE: each of these needs TWO fixes at once ====
    // interface element selects into a package array          (XZ-5 x XZ-1)
    `SVTEST_CHECK(CH_W[u_dma.BURST[0]] == 16,
                  "c99 cross: interface element indexes package array")
    // child-module element selects into a class array         (XZ-4 x XZ-2)
    `SVTEST_CHECK(o.mask(u_chan.SEL[0]) == 8'hA5,
                  "c99 cross: child element indexes class array")
    // specialisation-qualified element selects into a class array (XZ-3 x XZ-2)
    `SVTEST_CHECK(integ_pkg::cfg_c::smask(
                    integ_pkg::stream_c#(32, '{1,2,4,8})::rate(1)) == 8'hA5,
                  "c99 cross: param-class element indexes class array")
    // package element supplies a loop bound over an interface array (XZ-1 x XZ-5)
    // DEPTH[0] == 2, so this sums BURST[0] + BURST[1] == 1 + 4 == 5.
    s = 0;
    for (int i = 0; i < int'(DEPTH[0]); i++) s += u_dma.BURST[i];
    `SVTEST_CHECK(s == 5, "c99 cross: package element as loop bound over interface")

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
