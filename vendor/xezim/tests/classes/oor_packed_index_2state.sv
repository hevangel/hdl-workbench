// Self-test: an out-of-range element index into a packed multi-dimensional
// array relativizes x per the element's 4-state/2-state type (IEEE 1800
// §11.5.1, §5.8). A 4-STATE (`logic`) out-of-range packed element reads x,
// but a 2-STATE (`bit`) one reads 0 — a `bit` destination cannot hold x, so
// reference simulators return a known zero instead. Xezim returned x for
// BOTH, leaking x into the byte-3 lane of a partial-register read in the
// `99partial_ro` reg bit-bash scenario (the 24-bit `bit` array reads its
// 3rd byte while a 4-byte register composite read covers address 3).
//
//     bit [2:0][7:0] r;   // 24-bit, elements 0..2
//     r = r1;             // e.g. 32'h1
//     b3 = r[3];          // out-of-range: 0 for `bit`, x for `logic`
module top;
  bit [31:0] r1;
  initial begin
    r1 = 32'h00000001;
    begin
      bit [2:0][7:0] r;
      logic [2:0][7:0] lr;
      bit [7:0] b3;        // 2-state destination
      logic [7:0] lb3;     // 4-state destination retains x
      r = r1;
      lr = r1;
      b3 = r[3];           // OOR on 2-state packed array -> 0
      lb3 = lr[3];         // OOR on 4-state packed array -> x
      $display("b3=%02h lb3=%02h", b3, lb3);
      if (b3 === 8'h00 && lb3 === 8'hxx)
        $display("TAG_PASS");
      else
        $display("TAG_FAIL b3=%02h lb3=%02h", b3, lb3);
    end
  end
endmodule