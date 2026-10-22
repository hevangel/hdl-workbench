//! §8.25 / §6.20.2: the width of a class property follows the class
//! parameters of the object's specialization — a packed range sized by a
//! value parameter, and a type typed by a type parameter, including one
//! bound by an ancestor's `extends C #(...)` clause. Fills (§5.7.1), casts,
//! arrays, statics, locals and returns all take that width.
//!
//! Every expected line is the reference simulator's output for the same
//! source.

fn t_lines(src: &str) -> Vec<String> {
    let sim = xezim::simulate(src, 100).expect("simulate");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|l| l.starts_with("T|"))
        .collect()
}

fn check(src: &str, expected: &str) {
    let got = t_lines(src);
    let want: Vec<String> = expected
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(got, want);
}

/// Gap 1: `bit [W-1:0] pv = '1` in `pbase #(8)` (W defaults to 4) filled only
/// the default width. Covers the other unbased fills, `W'(5)`, `{W{1'b1}}`,
/// an array pattern, `$bits`, truncation and wrap-around on assignment, a
/// typedef specialization, a named override, one and two levels of
/// `extends`, and a static property, per specialization.
#[test]
fn value_param_fill_initializers() {
    check(
        r#"
class pbase #(int W = 4);
  bit [W-1:0] pv = '1;
  logic [W-1:0] p0 = '0;
  logic [W-1:0] px = 'x;
  logic [W-1:0] pz = 'z;
  bit [W-1:0] pc = W'(5);
  bit [W-1:0] pr = {W{1'b1}};
  bit [W-1:0] parr [2] = '{'1, '0};
  bit [W-1:0] pplain;
  static bit [W-1:0] sv = '1;
  function void show(string tag);
    $display("T| %s pv=%h p0=%h px=%h pz=%h pc=%h pr=%h arr=%h,%h bits=%0d sbits=%0d sv=%h", tag, pv, p0, px, pz, pc, pr, parr[0], parr[1], $bits(pv), $bits(sv), sv);
    pplain = 9'h1ff; $display("T| %s trunc=%h", tag, pplain);
    pplain = '1; pplain = pplain + 1; $display("T| %s wrap=%h", tag, pplain);
    $display("T| %s sum=%0d", tag, pv + 1'b1);
  endfunction
endclass
typedef pbase#(12) p12_t;
class pd8 extends pbase#(8);
endclass
class pdd8 extends pd8;
endclass
module tb;
  pbase#(8) a; pbase b; p12_t c; pbase#(.W(16)) d; pd8 e; pdd8 f;
  initial begin
    a = new(); b = new(); c = new(); d = new(); e = new(); f = new();
    a.show("p8"); b.show("pdef"); c.show("p12td"); d.show("pW16"); e.show("ext8"); f.show("ext2x8");
    $display("T| ext pv=%h bits=%0d", f.pv, $bits(f.pv));
    $display("T| mod a.pv=%h c.pv=%h d.pv=%h", a.pv, c.pv, d.pv);
  end
endmodule
"#,
        r#"
T| p8 pv=ff p0=00 px=xx pz=zz pc=05 pr=ff arr=ff,00 bits=8 sbits=8 sv=ff
T| p8 trunc=ff
T| p8 wrap=00
T| p8 sum=0
T| pdef pv=f p0=0 px=x pz=z pc=5 pr=f arr=f,0 bits=4 sbits=4 sv=f
T| pdef trunc=f
T| pdef wrap=0
T| pdef sum=0
T| p12td pv=fff p0=000 px=xxx pz=zzz pc=005 pr=fff arr=fff,000 bits=12 sbits=12 sv=fff
T| p12td trunc=1ff
T| p12td wrap=000
T| p12td sum=0
T| pW16 pv=ffff p0=0000 px=xxxx pz=zzzz pc=0005 pr=ffff arr=ffff,0000 bits=16 sbits=16 sv=ffff
T| pW16 trunc=01ff
T| pW16 wrap=0000
T| pW16 sum=0
T| ext8 pv=ff p0=00 px=xx pz=zz pc=05 pr=ff arr=ff,00 bits=8 sbits=8 sv=ff
T| ext8 trunc=ff
T| ext8 wrap=00
T| ext8 sum=0
T| ext2x8 pv=ff p0=00 px=xx pz=zz pc=05 pr=ff arr=ff,00 bits=8 sbits=8 sv=ff
T| ext2x8 trunc=ff
T| ext2x8 wrap=00
T| ext2x8 sum=0
T| ext pv=ff bits=8
T| mod a.pv=ff c.pv=fff d.pv=ffff
"#,
    );
}

/// Gap 2: a property typed by a type parameter, inherited through `extends
/// comp_base #(byte)`, kept the default type's width. Covers a fixed array,
/// a queue and a static of type `T`, two levels, a typedef specialization,
/// a named `.T(bit [5:0])` argument, a parameter passed on through a
/// middle class, truncation and signedness.
#[test]
fn type_param_property_through_extends() {
    check(
        r#"
class comp_base #(type T = int);
  T data;
  T darr [3];
  T dq [$];
  static T sdata;
  function void show(string tag);
    data = '1; darr[1] = '1; sdata = '1; dq.push_back('1);
    $display("T| %s data=%h bits=%0d darr=%h abits=%0d sd=%h sbits=%0d dq=%h", tag, data, $bits(data), darr[1], $bits(darr), sdata, $bits(sdata), dq[0]);
    data = 'h1ffff; $display("T| %s trunc=%h", tag, data);
    data = -1; $display("T| %s neg=%0d", tag, data);
  endfunction
endclass
class cb_byte extends comp_base#(byte);
endclass
class cb_byte2 extends cb_byte;
endclass
typedef comp_base#(shortint) cs_t;
class cb_short extends cs_t;
endclass
class cb_named extends comp_base#(.T(bit [5:0]));
endclass
class mid #(type U = int) extends comp_base#(U);
endclass
class leaf extends mid#(byte);
endclass
module tb;
  cb_byte a; cb_byte2 b; cb_short c; cb_named d; leaf e; comp_base#(byte) f; comp_base g;
  initial begin
    a = new(); b = new(); c = new(); d = new(); e = new(); f = new(); g = new();
    a.show("ext_byte"); b.show("ext2_byte"); c.show("ext_td_short"); d.show("ext_named6"); e.show("mid_byte"); f.show("direct_byte"); g.show("default");
    a.data = 'h1ff; $display("T| outer a=%h bits=%0d", a.data, $bits(a.data));
    b.data = 'h1ff; $display("T| outer b=%h", b.data);
  end
endmodule
"#,
        r#"
T| ext_byte data=ff bits=8 darr=ff abits=24 sd=ff sbits=8 dq=ff
T| ext_byte trunc=ff
T| ext_byte neg=-1
T| ext2_byte data=ff bits=8 darr=ff abits=24 sd=ff sbits=8 dq=ff
T| ext2_byte trunc=ff
T| ext2_byte neg=-1
T| ext_td_short data=ffff bits=16 darr=ffff abits=48 sd=ffff sbits=16 dq=ffff
T| ext_td_short trunc=ffff
T| ext_td_short neg=-1
T| ext_named6 data=3f bits=6 darr=3f abits=18 sd=3f sbits=6 dq=3f
T| ext_named6 trunc=3f
T| ext_named6 neg=63
T| mid_byte data=ff bits=8 darr=ff abits=24 sd=ff sbits=8 dq=ff
T| mid_byte trunc=ff
T| mid_byte neg=-1
T| direct_byte data=ff bits=8 darr=ff abits=24 sd=ff sbits=8 dq=ff
T| direct_byte trunc=ff
T| direct_byte neg=-1
T| default data=ffffffff bits=32 darr=ffffffff abits=96 sd=ffffffff sbits=32 dq=ffffffff
T| default trunc=0001ffff
T| default neg=-1
T| outer a=ff bits=8
T| outer b=ff
"#,
    );
}

/// Initializers of type `T` (`'1`, `'0`, `T'(300)`), method locals of type
/// `T`, width-dependent arithmetic, named overrides and specializations
/// through a typedef — including a ranged vector type argument.
#[test]
fn type_param_initializers_locals_and_typedef_specs_1() {
    check(
        r#"
class tb_base #(type T = int, int W = 4);
  T ti = '1;
  T tz = '0;
  T tc = T'(300);
  bit [W-1:0] wv = '1;
  logic [W*2-1:0] w2 = 'x;
  function void show(string tag);
    T tmp;
    $display("T| %s ti=%h tz=%h tc=%h tbits=%0d wv=%h w2=%h w2bits=%0d", tag, ti, tz, tc, $bits(ti), wv, w2, $bits(w2));
    ti = ti + 1; $display("T| %s inc=%h", tag, ti);
    tmp = 'h12345; $display("T| %s tmp=%h", tag, tmp);
    wv = wv + 1; $display("T| %s wvinc=%h", tag, wv);
    $display("T| %s sum=%0d", tag, wv + 5'd31);
  endfunction
endclass
class d1 extends tb_base#(byte, 8); endclass
class d2 extends tb_base#(.W(12), .T(shortint)); endclass
typedef tb_base#(bit [5:0], 6) tb6_t;
class d3 extends tb6_t; endclass
class d4 extends d1; endclass
typedef logic [9:0] l10_t;
class d5 extends tb_base#(l10_t); endclass
module tb;
  d1 a; d2 b; d3 c; d4 d; d5 e; tb_base#(byte,8) f; tb6_t g;
  initial begin
    a = new(); b = new(); c = new(); d = new(); e = new(); f = new(); g = new();
    a.show("d1"); b.show("d2"); c.show("d3"); d.show("d4"); e.show("d5"); f.show("direct"); g.show("td6");
    $display("T| outer %h %h %0d %0d", d.ti, d.wv, $bits(d.ti), $bits(d.wv));
  end
endmodule
"#,
        r#"
T| d1 ti=ff tz=00 tc=2c tbits=8 wv=ff w2=xxxx w2bits=16
T| d1 inc=00
T| d1 tmp=45
T| d1 wvinc=00
T| d1 sum=31
T| d2 ti=ffff tz=0000 tc=012c tbits=16 wv=fff w2=xxxxxx w2bits=24
T| d2 inc=0000
T| d2 tmp=2345
T| d2 wvinc=000
T| d2 sum=31
T| d3 ti=3f tz=00 tc=2c tbits=6 wv=3f w2=xxx w2bits=12
T| d3 inc=00
T| d3 tmp=05
T| d3 wvinc=00
T| d3 sum=31
T| d4 ti=ff tz=00 tc=2c tbits=8 wv=ff w2=xxxx w2bits=16
T| d4 inc=00
T| d4 tmp=45
T| d4 wvinc=00
T| d4 sum=31
T| d5 ti=3ff tz=000 tc=12c tbits=10 wv=f w2=xx w2bits=8
T| d5 inc=000
T| d5 tmp=345
T| d5 wvinc=0
T| d5 sum=31
T| direct ti=ff tz=00 tc=2c tbits=8 wv=ff w2=xxxx w2bits=16
T| direct inc=00
T| direct tmp=45
T| direct wvinc=00
T| direct sum=31
T| td6 ti=3f tz=00 tc=2c tbits=6 wv=3f w2=xxx w2bits=12
T| td6 inc=00
T| td6 tmp=05
T| td6 wvinc=00
T| td6 sum=31
T| outer 00 00 8 8
"#,
    );
}

/// Initializers of type `T` (`'1`, `'0`, `T'(300)`), method locals of type
/// `T`, width-dependent arithmetic, named overrides and specializations
/// through a typedef — including a ranged vector type argument.
#[test]
fn type_param_initializers_locals_and_typedef_specs_2() {
    check(
        r#"
class tb_base #(type T = int, int W = 4);
  T ti = '1;
  function void show(string tag);
    T tmp;
    tmp = 'h12345;
    $display("T| %s ti=%h tmp=%h tb=%0d lb=%0d", tag, ti, tmp, $bits(ti), $bits(tmp));
  endfunction
endclass
typedef tb_base#(bit [5:0], 6) tb6_t;
typedef tb_base#(byte) tbb_t;
class d3 extends tb6_t; endclass
class d6 extends tbb_t; endclass
module tb;
  tb_base#(bit [5:0], 6) h; tb6_t g; d3 c; d6 e; tbb_t f; tb_base#(shortint) k;
  initial begin
    h = new(); g = new(); c = new(); e = new(); f = new(); k = new();
    h.show("direct6"); g.show("td6"); c.show("ext_td6"); e.show("ext_tdb"); f.show("tdb"); k.show("short");
  end
endmodule
"#,
        r#"
T| direct6 ti=3f tmp=05 tb=6 lb=6
T| td6 ti=3f tmp=05 tb=6 lb=6
T| ext_td6 ti=3f tmp=05 tb=6 lb=6
T| ext_tdb ti=ff tmp=45 tb=8 lb=8
T| tdb ti=ff tmp=45 tb=8 lb=8
T| short ti=ffff tmp=2345 tb=16 lb=16
"#,
    );
}

/// Fixed-array, queue and dynamic-array members of parameter-dependent
/// element type (initializers, indexed stores, `push_back('1)`, `$bits`),
/// a value parameter passed through a middle class, a class-local typedef
/// sized by a parameter, and statics per specialization (with and
/// without an initializer).
#[test]
fn member_arrays_queues_and_statics_1() {
    check(
        r#"
class plain;
  int arr [3];
  int q [$];
  static int s;
  static bit [7:0] s8 = '1;
  bit [7:0] a8 [2] = '{'1, '0};
  function void show();
    arr[1] = '1; q.push_back('1); s = '1;
    $display("T| plain arr=%h abits=%0d q=%h s=%h sbits=%0d s8=%h a8=%h,%h", arr[1], $bits(arr), q[0], s, $bits(s), s8, a8[0], a8[1]);
  endfunction
endclass
class pc #(int W = 4);
  bit [W-1:0] pc1 = W'(5);
  bit [W-1:0] pc2 = 5;
  int iw = W;
  bit [W-1:0] arr2 [2];
  function void show(string t);
    arr2[0] = '1;
    $display("T| %s pc1=%h pc2=%h iw=%0d arr2=%h a2bits=%0d", t, pc1, pc2, iw, arr2[0], $bits(arr2));
  endfunction
endclass
module tb;
  plain p; pc#(8) c8; pc cd;
  initial begin
    p = new(); p.show(); c8 = new(); c8.show("c8"); cd = new(); cd.show("cd");
  end
endmodule
"#,
        r#"
T| plain arr=ffffffff abits=96 q=ffffffff s=ffffffff sbits=32 s8=ff a8=ff,00
T| c8 pc1=05 pc2=05 iw=8 arr2=ff a2bits=16
T| cd pc1=5 pc2=5 iw=4 arr2=f a2bits=8
"#,
    );
}

/// Fixed-array, queue and dynamic-array members of parameter-dependent
/// element type (initializers, indexed stores, `push_back('1)`, `$bits`),
/// a value parameter passed through a middle class, a class-local typedef
/// sized by a parameter, and statics per specialization (with and
/// without an initializer).
#[test]
fn member_arrays_queues_and_statics_2() {
    check(
        r#"
class cc #(type T = int, int W = 4);
  T darr [3];
  bit [W-1:0] warr [2];
  function void show(string tag);
    darr[1] = '1; warr[0] = '1;
    $display("T| %s a=%h w=%h", tag, darr[1], warr[0]);
    darr[2] = 'h1ff; warr[1] = 'h1ff;
    $display("T| %s a=%h w=%h", tag, darr[2], warr[1]);
    this.darr[0] = '1; $display("T| %s this=%h", tag, darr[0]);
  endfunction
endclass
module tb;
  cc#(byte, 8) a; cc b;
  initial begin
    a = new(); b = new(); a.show("a"); b.show("b");
    a.darr[0] = '1; a.warr[0] = 'h1ff; $display("T| outer %h %h", a.darr[0], a.warr[0]);
  end
endmodule
"#,
        r#"
T| a a=ff w=ff
T| a a=ff w=ff
T| a this=ff
T| b a=ffffffff w=f
T| b a=000001ff w=f
T| b this=ffffffff
T| outer ff ff
"#,
    );
}

/// Fixed-array, queue and dynamic-array members of parameter-dependent
/// element type (initializers, indexed stores, `push_back('1)`, `$bits`),
/// a value parameter passed through a middle class, a class-local typedef
/// sized by a parameter, and statics per specialization (with and
/// without an initializer).
#[test]
fn member_arrays_queues_and_statics_3() {
    check(
        r#"
class pbase #(int W = 4);
  bit [W-1:0] pv = '1;
  static bit [W-1:0] sv = '1;
  static logic [W-1:0] sx;
  function new(); endfunction
endclass
class mid #(int N = 3) extends pbase#(N);
  bit [N:0] mv = '1;
endclass
class leaf extends mid#(8);
  bit [3:0] own = '1;
endclass
class tdc #(int W = 4);
  typedef bit [W-1:0] word_t;
  word_t w = '1;
  word_t wa [2] = '{'1, 0};
endclass
class arrc #(type T = int);
  T arr [2] = '{'1, '0};
  T q [$] = '{'1};
  T dyn [] = '{'1, '1};
endclass
module tb;
  leaf l; tdc#(12) t; tdc t4; arrc#(byte) ab; arrc ad; pbase#(6) p6;
  initial begin
    l = new(); t = new(); t4 = new(); ab = new(); ad = new(); p6 = new();
    $display("T| leaf pv=%h mv=%h own=%h %0d %0d", l.pv, l.mv, l.own, $bits(l.pv), $bits(l.mv));
    $display("T| tdc w=%h wa=%h %h bits=%0d", t.w, t.wa[0], t.wa[1], $bits(t.w));
    $display("T| tdc4 w=%h wa=%h %h bits=%0d", t4.w, t4.wa[0], t4.wa[1], $bits(t4.w));
    $display("T| arrb %h %h q=%h d=%h %h", ab.arr[0], ab.arr[1], ab.q[0], ab.dyn[0], ab.dyn[1]);
    $display("T| arrd %h %h q=%h d=%h %h", ad.arr[0], ad.arr[1], ad.q[0], ad.dyn[0], ad.dyn[1]);
    $display("T| static %h %h %0d %0d sx=%h", pbase#(6)::sv, pbase#(8)::sv, $bits(pbase#(6)::sv), $bits(pbase#(16)::sv), pbase#(5)::sx);
    pbase#(6)::sv = 'h1ff; pbase#(6)::sx = 'z;
    $display("T| static2 %h %h sx=%h", pbase#(6)::sv, pbase#()::sv, pbase#(6)::sx);
  end
endmodule
"#,
        r#"
T| leaf pv=ff mv=1ff own=f 8 9
T| tdc w=fff wa=fff 000 bits=12
T| tdc4 w=f wa=f 0 bits=4
T| arrb ff 00 q=ff d=ff ff
T| arrd ffffffff 00000000 q=ffffffff d=ffffffff ffffffff
T| static 3f ff 6 16 sx=xx
T| static2 3f f sx=zz
"#,
    );
}

/// N-D fixed-array and associative members of type `T` and of
/// `bit [W-1:0]`: indexed `'1` stores take this specialization's width.
#[test]
fn member_arrays_queues_and_statics_4() {
    check(
        r#"
class nd #(type T = int, int W = 4);
  T m [2][2];
  bit [W-1:0] wm [2][2];
  T aa [string];
  function void show(string tag);
    m[1][0] = '1; wm[0][1] = '1; aa["k"] = '1;
    $display("T| %s m=%h wm=%h aa=%h", tag, m[1][0], wm[0][1], aa["k"]);
  endfunction
endclass
module tb;
  nd#(byte, 8) a; nd b;
  initial begin a = new(); b = new(); a.show("a"); b.show("b"); end
endmodule
"#,
        r#"
T| a m=ff wm=ff aa=ff
T| b m=ffffffff wm=f aa=ffffffff
"#,
    );
}

/// Method returns of type `T` and of `bit [W-1:0]` (instance and static
/// calls, inherited through `extends`), and the signedness of `T`-typed
/// properties, locals and statics.
#[test]
fn type_param_returns_and_signedness_1() {
    check(
        r#"
typedef logic [11:0] l12_t;
class cb #(type T = int, int W = 4);
  T d;
  bit [W-1:0] v;
  static T sd;
  function T get_ones(); return '1; endfunction
  function bit [W-1:0] get_w(); return '1; endfunction
  function void chk(string tag);
    T loc = '1;
    d = -1;
    $display("T| %s neg=%0d lt0=%0d loc=%h lb=%0d", tag, d, d < 0, loc, $bits(loc));
    sd = -2; $display("T| %s sd=%0d", tag, sd);
  endfunction
endclass
typedef cb#(.W(10)) c10_t;
class cx extends cb#(shortint, 6); endclass
class cl extends cb#(l12_t); endclass
module tb;
  cb#(byte) a; c10_t b; cx c; cl e;
  initial begin
    a = new(); b = new(); c = new(); e = new();
    a.chk("byte"); b.chk("c10"); c.chk("cx"); e.chk("l12");
    a.d = '1; b.v = '1; c.v = '1; c.d = '1; e.d = '1;
    $display("T| outer %h %h %h %h %h", a.d, b.v, c.v, c.d, e.d);
    $display("T| ret %h %h %h %h", a.get_ones(), b.get_w(), c.get_ones(), c.get_w());
    $display("T| bits %0d %0d %0d %0d", $bits(a.d), $bits(b.v), $bits(c.d), $bits(e.d));
  end
endmodule
"#,
        r#"
T| byte neg=-1 lt0=1 loc=ff lb=8
T| byte sd=-2
T| c10 neg=-1 lt0=1 loc=ffffffff lb=32
T| c10 sd=-2
T| cx neg=-1 lt0=1 loc=ffff lb=16
T| cx sd=-2
T| l12 neg=4095 lt0=0 loc=fff lb=12
T| l12 sd=4094
T| outer ff 3ff 3f ffff fff
T| ret ff 3ff ffff 3f
T| bits 8 10 16 12
"#,
    );
}

/// Method returns of type `T` and of `bit [W-1:0]` (instance and static
/// calls, inherited through `extends`), and the signedness of `T`-typed
/// properties, locals and statics.
#[test]
fn type_param_returns_and_signedness_2() {
    check(
        r#"
class sb #(type T = int, int W = 4);
  static function T sones(); return '1; endfunction
  static function bit [W-1:0] swones(); return '1; endfunction
  static function int tbits(); T x; return $bits(x); endfunction
endclass
class ext_sb extends sb#(byte, 6);
  static function T eones(); return '1; endfunction
endclass
module tb;
  initial begin
    $display("T| s %h %h %0d", sb#(byte)::sones(), sb#(shortint, 8)::swones(), sb#(shortint)::tbits());
    $display("T| d %h %h %0d", sb#()::sones(), sb#()::swones(), sb#()::tbits());
    $display("T| e %h %h", ext_sb::eones(), ext_sb::swones());
  end
endmodule
"#,
        r#"
T| s ff ff 16
T| d ffffffff f 32
T| e ff 3f
"#,
    );
}
