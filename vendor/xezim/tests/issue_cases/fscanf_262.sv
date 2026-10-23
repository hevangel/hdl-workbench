`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH

`define SVTEST_INIT \
  int failures = 0;

`define SVTEST_CHECK(expr, msg) \
  if (!(expr)) begin \
    failures++; \
    $display("FAIL: %s", msg); \
  end

`define SVTEST_PASSFAIL \
  if (failures == 0) begin \
    $display("TEST_PASS"); \
  end else begin \
    $display("TEST_FAIL count=%0d", failures); \
    $fatal(1); \
  end
`endif

// Issue #262: $fscanf(fd, "%c", c) must match exactly one character per
// IEEE 1800-2017 §21.3.4.3 ("For all descriptors except the character c,
// white space leading an input field is ignored"; Table 21-8: %c "Matches
// a single character, whose 8-bit ASCII value is returned") — %c (and
// literal-only formats) may NOT skip whitespace, repeated %c must keep
// every newline, and every byte value 0..255 must round-trip through
// $fwrite/$fscanf %c. Also covers the cousin fixes: task-form $fscanf
// converting and C-stdio pushback semantics ($ftell subtracts it, $fread
// drains it, $fseek/$rewind discard it).
module fscanf_262_tb;

  localparam string FNAME = "/tmp/xezim_issue262_fscanf.bin";

  integer fd, n, i, bad;
  reg [7:0] c, iv;
  reg [7:0] exp [0:5];
  reg [7:0] fbuf [0:2];

  initial begin
    `SVTEST_INIT

    // 1. The reported bug: %c on a whitespace-only file must match.
    fd = $fopen(FNAME, "w");
    $fwrite(fd, " ");
    $fclose(fd);
    fd = $fopen(FNAME, "r");
    c = 8'h00;
    n = $fscanf(fd, "%c", c);
    `SVTEST_CHECK(n == 1, "issue 262: %c on a single space must match (n==1)")
    `SVTEST_CHECK(c == 8'h20, "issue 262: %c must return the space itself")
    n = $fscanf(fd, "%c", c);
    `SVTEST_CHECK(n == -1, "issue 262: %c at EOF must return -1")
    $fclose(fd);

    // 2. Repeated %c across lines keeps every newline.
    fd = $fopen(FNAME, "w");
    $fwrite(fd, "ab\ncd\n");
    $fclose(fd);
    fd = $fopen(FNAME, "r");
    exp[0] = "a"; exp[1] = "b"; exp[2] = "\n";
    exp[3] = "c"; exp[4] = "d"; exp[5] = "\n";
    bad = 0;
    for (i = 0; i < 6; i = i + 1) begin
      c = 8'hxx;
      n = $fscanf(fd, "%c", c);
      if (n != 1 || c !== exp[i]) begin
        bad = bad + 1;
        $display("  char %0d: n=%0d c=%02x (want %02x)", i, n, c, exp[i]);
      end
    end
    `SVTEST_CHECK(bad == 0, "repeated %c must return every char incl. newlines")
    $fclose(fd);

    // 3. All 256 byte values round-trip through %c (write side must emit
    //    the raw byte, read side must return it — no UTF-8 mangling).
    fd = $fopen(FNAME, "wb");
    for (i = 0; i < 256; i = i + 1) begin
      iv = i;
      $fwrite(fd, "%c", iv);
    end
    $fclose(fd);
    fd = $fopen(FNAME, "r");
    bad = 0;
    for (i = 0; i < 256; i = i + 1) begin
      c = 8'hxx;
      n = $fscanf(fd, "%c", c);
      iv = i;
      if (n != 1 || c !== iv) begin
        bad = bad + 1;
        $display("  byte %0d: n=%0d c=%02x (want %02x)", i, n, c, iv);
      end
    end
    `SVTEST_CHECK(bad == 0, "all 256 byte values must round-trip via %c")
    n = $fscanf(fd, "%c", c);
    `SVTEST_CHECK(n == -1, "%c past the 256 bytes must hit EOF (-1)")
    $fclose(fd);

    // 4. Task-form $fscanf (result discarded) must still convert.
    fd = $fopen(FNAME, "w");
    $fwrite(fd, "Z");
    $fclose(fd);
    fd = $fopen(FNAME, "r");
    c = 8'h00;
    $fscanf(fd, "%c", c);
    `SVTEST_CHECK(c == 8'h5a, "task-form $fscanf must convert %c")
    $fclose(fd);

    // 5. Pushback after a partially matched line, C-stdio style.
    fd = $fopen(FNAME, "w");
    $fwrite(fd, "12 34");
    $fclose(fd);
    fd = $fopen(FNAME, "r");
    i = 0;
    n = $fscanf(fd, "%d", i);
    `SVTEST_CHECK(n == 1 && i == 12, "%d must parse 12 from '12 34'")
    `SVTEST_CHECK($ftell(fd) == 2, "$ftell must subtract pending pushback")
    n = $fread(fbuf, fd);
    `SVTEST_CHECK(n == 3, "$fread must drain the pushed-back tail first")
    `SVTEST_CHECK(fbuf[0] == 8'h20 && fbuf[1] == 8'h33 && fbuf[2] == 8'h34,
                  "$fread must see ' 34' after the %d match")
    $fseek(fd, 0, 0);
    n = $fscanf(fd, "%c", c);
    `SVTEST_CHECK(n == 1 && c == 8'h31, "$fseek must discard stale pushback")
    $rewind(fd);
    n = $fscanf(fd, "%c", c);
    `SVTEST_CHECK(n == 1 && c == 8'h31, "$rewind must discard stale pushback")
    $fclose(fd);

    // 6. Literals do not skip whitespace; explicit format whitespace does.
    fd = $fopen(FNAME, "w");
    $fwrite(fd, "  xY");
    $fclose(fd);
    fd = $fopen(FNAME, "r");
    c = 8'h00;
    n = $fscanf(fd, "x%c", c);
    `SVTEST_CHECK(n == 0, "literal 'x' must not skip leading whitespace")
    $fclose(fd);
    fd = $fopen(FNAME, "r");
    c = 8'h00;
    n = $fscanf(fd, " x%c", c);
    `SVTEST_CHECK(n == 1 && c == 8'h59,
                  "format whitespace must skip input ws, then %c reads 'Y'")
    $fclose(fd);

    `SVTEST_PASSFAIL
  end
endmodule