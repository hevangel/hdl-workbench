// Neighbours of issue #302: sum windows from >=/<= pairs and inside ranges,
// sum() with and without a with clause, product() and xor() reductions.
module top;
  int a [17];
  int b [8];
  bit [7:0] u [6];
  byte sb [5];
  int c, d, ok;
  bit [15:0] p0;
  bit [1:0] p1;
  int s;
  initial begin
    p0 = 256; p1 = 3; d = 17;
    // W1: window from a >=/<= pair around a rand center built with a cast
    ok = std::randomize(c, a) with {
      c == (d * 32'(p0) * (p1 + 1));
      a.sum() >= c - 10;
      a.sum() <= c + 10;
    };
    s = a.sum();
    $display("T|W1 %s", (ok && c == 17408 && s >= c - 10 && s <= c + 10) ? "PASS" : "FAIL");
    // W2: same window, sum with `with` cast, bounds on the other side
    ok = std::randomize(c, a) with {
      c == 17408;
      c - 10 <= a.sum() with (32'(item));
      c + 10 >= a.sum() with (32'(item));
    };
    s = a.sum() with (32'(item));
    $display("T|W2 %s", (ok && s >= c - 10 && s <= c + 10) ? "PASS" : "FAIL");
    // W3: unsigned cast in the sum, rand bounds via inside
    ok = std::randomize(c, a) with {
      c == (d * 32'(p0) * (p1 + 1));
      a.sum() with (int'(unsigned'(item))) inside {[c - 3 : c + 3]};
    };
    s = a.sum() with (int'(unsigned'(item)));
    $display("T|W3 %s", (ok && s >= c - 3 && s <= c + 3) ? "PASS" : "FAIL");
    // W4: sum equality to a rand var with a cast-built value
    ok = std::randomize(c, b) with {
      c == 32'(p0) * 4 + 7;
      b.sum() == c;
      foreach (b[i]) b[i] inside {[0:1000]};
    };
    $display("T|W4 %s", (ok && c == 1031 && b.sum() == 1031) ? "PASS" : "FAIL");
    // W5: two separate windows on the same sum (both halves via inside)
    ok = std::randomize(c, b) with {
      c inside {[100:200]};
      b.sum() inside {[c - 5 : c + 5]};
      b.sum() inside {[c : c + 20]};
    };
    s = b.sum();
    $display("T|W5 %s", (ok && c >= 100 && c <= 200 && s >= c && s <= c + 5) ? "PASS" : "FAIL");
    // P1: product of small unsigned elements
    ok = std::randomize(u) with {
      foreach (u[i]) u[i] inside {[1:3]};
      u.product() == 24;
    };
    $display("T|P1 %s", (ok && u.product() == 24) ? "PASS" : "FAIL");
    // P2: product with a `with` widening, rand target
    ok = std::randomize(c, u) with {
      foreach (u[i]) u[i] inside {[1:4]};
      c == 2 * 3 * 4;
      u.product() with (int'(item)) == c;
    };
    $display("T|P2 %s", (ok && c == 24 && (u.product() with (int'(item))) == 24) ? "PASS" : "FAIL");
    // X1: xor reduction
    ok = std::randomize(u) with { u.xor() == 8'h5A; };
    $display("T|X1 %s", (ok && u.xor() == 8'h5A) ? "PASS" : "FAIL");
    // X2: xor with `with`, rand target
    ok = std::randomize(c, u) with { c == 32'(p0) - 1; u.xor() with (int'(item)) == c; };
    $display("T|X2 %s", (ok && c == 255 && (u.xor() with (int'(item))) == 255) ? "PASS" : "FAIL");
    // S1: signed byte sum, window around a rand center
    ok = std::randomize(c, sb) with {
      c == -100;
      sb.sum() with (int'(item)) >= c - 2;
      sb.sum() with (int'(item)) <= c + 2;
    };
    s = sb.sum() with (int'(item));
    $display("T|S1 %s", (ok && s >= -102 && s <= -98) ? "PASS" : "FAIL");
    // S2: plain byte sum wraps at 8 bits (§7.12.3)
    ok = std::randomize(sb) with { sb.sum() == 8'sd100; foreach (sb[i]) sb[i] > 30; };
    $display("T|S2 %s", (ok && sb.sum() == 8'sd100) ? "PASS" : "FAIL");
  end
endmodule
