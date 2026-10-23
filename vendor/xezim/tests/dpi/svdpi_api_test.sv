// IEEE 1800-2017 Annex H from C (tests/dpi/svdpi_api.c): open-array queries
// and element access for each canonical element type, ascending and
// descending ranges, dynamic arrays and queues (empty too), output and inout
// open arrays, bit/part selects of plain vectors, per-scope user data,
// svGetCallerInfo and the disable protocol. Lines tagged `T|` are checked by
// the cargo test.
module leaf ();
  import "DPI-C" context function int ud_put(input int v);
  import "DPI-C" context function int ud_get();
endmodule

module top;
  import "DPI-C" function string int_shape(input int a[]);
  import "DPI-C" function string int_edges(input int a[]);
  import "DPI-C" function int sum_bytes(input byte a[]);
  import "DPI-C" function int sum_shorts(input shortint a[]);
  import "DPI-C" function longint sum_longs(input longint a[]);
  import "DPI-C" function real sum_reals(input real a[]);
  import "DPI-C" function real sum_shortreals(input shortreal a[]);
  import "DPI-C" function void double_bytes(inout byte a[]);
  import "DPI-C" function void scale_reals(inout real a[]);
  import "DPI-C" function void scale_ints(inout int a[], input int k);
  import "DPI-C" function string bitvec_shape(input bit [11:0] a[]);
  import "DPI-C" function string logicvec_dump(input logic [3:0] a[]);
  import "DPI-C" function void fill_bitvec(output bit [11:0] a[]);
  import "DPI-C" function void mark_logic(inout logic [3:0] a[]);
  import "DPI-C" function int flip_bits(inout bit a[]);
  import "DPI-C" function int resolve_logic(inout logic a[]);
  import "DPI-C" function int get_part(input bit [63:0] v, input int i, input int w);
  import "DPI-C" function void put_part(inout bit [63:0] v, input int i, input int w, input int val);
  import "DPI-C" function int get_bit(input bit [63:0] v, input int i);
  import "DPI-C" function int get_lpart(input logic [39:0] v, input int i, input int w);
  import "DPI-C" function void put_lpart(inout logic [39:0] v, input int i, input int w,
                                         input int aval, input int bval);
  import "DPI-C" context function string ud_rules();
  import "DPI-C" function string misc_state();

  int up[2:5];
  int down[7:4];
  int dyn[];
  int none[];
  int q[$];
  byte b[0:3];
  shortint s[1:3];
  longint l[2];
  real r[0:2];
  shortreal sr[0:1];
  bit [11:0] bv[3:1];
  bit [11:0] bout[0:2];
  logic [3:0] lv[0:2];
  bit sb[0:4];
  logic sl[3:0];
  bit [63:0] v64;
  logic [39:0] v40;

  leaf u1 ();
  leaf u2 ();

  initial begin
    // shape: an ascending and a descending range, a dynamic array, a queue
    // and an empty dynamic array
    foreach (up[i]) up[i] = i * 10;
    foreach (down[i]) down[i] = i * 100;
    dyn = new[3];
    foreach (dyn[i]) dyn[i] = i + 7;
    q = '{5, 6};
    $display("T|up %s", int_shape(up));
    $display("T|down %s", int_shape(down));
    $display("T|dyn %s", int_shape(dyn));
    $display("T|queue %s", int_shape(q));
    $display("T|none %s", int_shape(none));
    $display("T|edges %s", int_edges(down));

    // canonical element types
    b = '{-3, 100, 27, -128};
    s = '{-1000, 2000, 30000};
    l = '{64'd1 << 40, -5};
    r = '{1.25, -2.5, 100.0};
    sr = '{0.5, 2.25};
    $display("T|sums byte=%0d short=%0d long=%0d real=%0.3f shortreal=%0.3f", sum_bytes(b),
             sum_shorts(s), sum_longs(l), sum_reals(r), sum_shortreals(sr));
    b = '{1, -2, 3, 60};
    double_bytes(b);
    scale_reals(r);
    scale_ints(dyn, 3);
    $display("T|inout b=%0d,%0d,%0d,%0d r=%0.3f,%0.3f,%0.3f dyn=%0d,%0d,%0d", b[0], b[1], b[2],
             b[3], r[0], r[1], r[2], dyn[0], dyn[1], dyn[2]);

    // packed elements
    bv[3] = 12'habc;
    bv[2] = 12'h123;
    bv[1] = 12'hfff;
    $display("T|bitvec %s", bitvec_shape(bv));
    lv = '{4'b10xz, 4'b0101, 4'bzzzz};
    $display("T|logicvec %s", logicvec_dump(lv));
    fill_bitvec(bout);
    $display("T|fill %h %h %h", bout[0], bout[1], bout[2]);
    mark_logic(lv);
    $display("T|mark %b %b %b", lv[0], lv[1], lv[2]);

    // scalar elements
    sb = '{1, 0, 1, 1, 0};
    $display("T|flip ones=%0d now=%b%b%b%b%b", flip_bits(sb), sb[0], sb[1], sb[2], sb[3], sb[4]);
    sl = '{1'bx, 1'b0, 1'bz, 1'b1};
    $display("T|resolve unknown=%0d now=%b", resolve_logic(sl), {sl[3], sl[2], sl[1], sl[0]});

    // bit and part selects of plain vectors
    v64 = 64'h0123_4567_89ab_cdef;
    $display("T|part low=%h cross=%h top=%h bit4=%0d bit5=%0d", get_part(v64, 0, 16),
             get_part(v64, 28, 8), get_part(v64, 56, 8), get_bit(v64, 4), get_bit(v64, 5));
    put_part(v64, 30, 4, 4'hf);
    put_part(v64, 60, 4, 0);
    $display("T|put %h", v64);
    v40 = 40'hff_0000_00ff;
    v40[9:8] = 2'bxz;
    $display("T|lpart %h", get_lpart(v40, 6, 6));
    put_lpart(v40, 32, 4, 32'h5, 32'h3);
    $display("T|lput %b", v40[39:32]);

    // per-scope user data: each instance keeps its own value under one key
    $display("T|ud put %0d %0d", u1.ud_put(11), u2.ud_put(22));
    $display("T|ud get %0d %0d", u1.ud_get(), u2.ud_get());
    void'(u1.ud_put(33));
    $display("T|ud again %0d %0d", u1.ud_get(), u2.ud_get());
    $display("T|ud %s", ud_rules());
    $display("T|misc %s", misc_state());
    $display("T|done");
  end
endmodule
