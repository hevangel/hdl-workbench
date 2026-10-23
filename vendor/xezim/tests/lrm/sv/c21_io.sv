// top: c21
`timescale 1ns/1ps
module c21;
  logic [7:0] v8 = 8'hA5;
  logic [15:0] vx = 16'b1010_xxxx_zzzz_0101;
  logic signed [7:0] s8 = -8'sd5;
  int i = -42;
  real r = 3.14159;
  string s = "hi";
  logic [31:0] mem [0:7];
  logic [7:0] memb [0:3];
  typedef struct { int a; logic [3:0] b; string c; } st_t;
  st_t st = '{1, 4'hx, "zz"};
  typedef enum {E0, E1} e_t;
  e_t ev = E1;
  wire (strong1, weak0) sn = 1'b1;
  initial begin
    // 21.2.1 format specifiers
    $display("T|21.2.1a|%h %H %o %b %d %D", v8, v8, v8, v8, v8, v8);
    $display("T|21.2.1b|[%0h] [%0o] [%0b] [%0d]", v8, v8, v8, v8);
    $display("T|21.2.1c|[%5d] [%-5d] [%05d] [%x]", v8, v8, v8, v8);
    $display("T|21.2.1d|[%d] [%0d] [%h]", s8, s8, s8);
    $display("T|21.2.1e|[%d] [%0d] [%8d] [%-8d|", i, i, i, i);
    $display("T|21.2.1f|%h %o %b", vx, vx, vx);
    $display("T|21.2.1g|[%d] [%0d]", vx, vx);
    $display("T|21.2.1h|[%d] [%d]", 16'bxxxx_xxxx_xxxx_xxxx, 16'bzzzz_zzzz_zzzz_zzzz);
    $display("T|21.2.1i|[%d] [%d]", 8'b0000_000x, 8'b0000_zzzz);
    $display("T|21.2.1j|%e|%f|%g|%10.3f|%-10.2e|%0.1f", r, r, r, r, r, r);
    $display("T|21.2.1k|%g %g %g", 1.0e-10, 123456789.0, 0.0001);
    $display("T|21.2.1l|[%s] [%5s] [%-5s] [%0s]", s, s, s, s);
    $display("T|21.2.1m|%c%c", 8'h41, 16'h4243);
    $display("T|21.2.1n|%m");
    $display("T|21.2.1o|%t", $time);
    $display("T|21.2.1p|%v %v", sn, v8[0]);
    $display("T|21.2.1q|%p", st);
    $display("T|21.2.1r|%p %p", ev, v8);
    $display("T|21.2.1s|%0p", st);
    $display("T|21.2.1t|%u", 8'h41);
    $display("T|21.2.1u|%z", 4'b10xz);
    $display("T|21.2.1v|%s", 16'h0041);
    $display("T|21.2.1w|%0s|", 32'h00_00_41_42);
    $display("T|21.2.1x|%d", 64'hffff_ffff_ffff_ffff);
    $display("T|21.2.1y|%0d %0d", 100'd1 << 90, -100'sd5);
    $display("T|21.2.1z|%h", 100'd1 << 90);
    $display("T|21.2.1A|%l");
    $display("T|21.2.1B|%%|\\|\"");
    $display("T|21.2.1C|", 5, "x", 8'h3);
    $display("T|21.2.1D|%0b %b", 1'b1, 3'b0);
    $display("T|21.2.1E|%d", 1'bx);
    $display("T|21.2.1F|%h", 5'bx1010);
    $display("T|21.2.1G|%o", 6'b101xxx);
    $display("T|21.2.1H|%o", 6'b1010x1);
    $display("T|21.2.1I|%h", 8'bz000_zzzz);
    $display("T|21.2.1J|%f %d", 2.5, 2.5);
    $display("T|21.2.1K|%0e", 12345.678);
    $display("T|21.2.1L|%d", r);
    $display("T|21.2.1M|%b", r);
    $display("T|21.2.1N|%5.2f|%-8.3e|", -1.5, -0.00123);
    $display("T|21.2.1O|%0x %0X", 255, 255);
    $display("T|21.2.1P|%10t|%-10t|", 5, 5);
    $display("T|21.2.1Q|%s", "");
    $display("T|21.2.1R|", ev);
    $display("T|21.2.1S|%x", "AB");
    $display("T|21.2.1T|%3c|", "A");
    begin int q[$] = {1,2}; int aa[string] = '{"k":5}; $display("T|21.2.1V|%p %p", q, aa); end
    // no-format defaults
    $display("T|21.2.1W|", v8, " ", s8, " ", i);
    $displayh("T|21.2.1X|", v8, " ", i);
    $displayb("T|21.2.1Y|", 4'd5);
    $displayo("T|21.2.1Z|", 9);
    $write("T|21.2.2|w1 "); $write("w2"); $write("\n");
    // 21.3 file I/O
    begin int fd, fd2, n, c, pos; string line; int a, b2; string word; real rv;
      fd = $fopen("c21_out.txt", "w");
      $fdisplay(fd, "line1 %0d", 10);
      $fwrite(fd, "line2 %h\n", 8'hab);
      $fstrobe(fd, "line3");
      #1;
      $fclose(fd);
      fd = $fopen("c21_out.txt", "r");
      $display("T|21.3.1a|fd_nonzero=%0d", fd != 0);
      n = $fgets(line, fd); $display("T|21.3.4a|%0d [%s]", n, line);
      c = $fgetc(fd); $display("T|21.3.4b|%c", c);
      n = $ungetc(c, fd);
      n = $fscanf(fd, "%s %h", word, a); $display("T|21.3.4c|%0d %s %h", n, word, a);
      pos = $ftell(fd); $display("T|21.3.5a|%0d", pos);
      n = $fgets(line, fd); n = $fgets(line, fd); $display("T|21.3.4d|[%s]", line);
      n = $fgets(line, fd); $display("T|21.3.4e|%0d eof=%0d", n, $feof(fd));
      c = $fgetc(fd); $display("T|21.3.4f|%0d", c);
      n = $rewind(fd); n = $fgets(line, fd); $display("T|21.3.5b|[%s]", line);
      n = $fseek(fd, 2, 0); c = $fgetc(fd); $display("T|21.3.5c|%c", c);
      n = $fseek(fd, -1, 2); $display("T|21.3.5d|%0d", $ftell(fd));
      $fclose(fd);
      fd2 = $fopen("nonexistent_dir/x.txt", "r"); $display("T|21.3.1b|%0d", fd2);
      begin string em; int ec; ec = $ferror(fd2, em); $display("T|21.3.7|ec_nonzero=%0d", ec != 0); end
      // multichannel descriptor
      fd = $fopen("c21_mcd.txt"); $display("T|21.3.1c|mcd=%0d", fd[31] == 0 && fd != 0);
      $fdisplay(fd | 1, "T|21.3.1d|to mcd and stdout");
      $fclose(fd);
      // append mode
      fd = $fopen("c21_out.txt", "a"); $fwrite(fd, "app\n"); $fclose(fd);
      fd = $fopen("c21_out.txt", "r"); n = 0; while (!$feof(fd)) begin void'($fgets(line, fd)); n++; end $display("T|21.3.1e|lines=%0d", n); $fclose(fd);
      // $fread
      fd = $fopen("c21_out.txt", "rb"); begin logic [15:0] w16; n = $fread(w16, fd); $display("T|21.3.4g|%0d %h", n, w16); end $fclose(fd);
      // 21.3.3 $sformat/$sformatf/$swrite
      $sformat(line, "%0d-%s", 7, "x"); $display("T|21.3.3a|%s", line);
      line = $sformatf("%5.1f|%h", 2.25, 12'habc); $display("T|21.3.3b|%s", line);
      $swrite(line, "a", 5, "b"); $display("T|21.3.3c|[%s]", line);
      $swriteh(line, 255); $display("T|21.3.3d|[%s]", line);
      // $sscanf
      n = $sscanf("12 ab 3.5 word", "%d %h %f %s", a, b2, rv, word); $display("T|21.3.4h|%0d %0d %0d %f %s", n, a, b2, rv, word);
      n = $sscanf("1011,077", "%b,%o", a, b2); $display("T|21.3.4i|%0d %0d %0d", n, a, b2);
      n = $sscanf("abc", "%d", a); $display("T|21.3.4j|%0d", n);
      n = $sscanf("  42xyz", "%d%s", a, word); $display("T|21.3.4k|%0d %0d %s", n, a, word);
      n = $sscanf("x=5", "x=%d", a); $display("T|21.3.4l|%0d %0d", n, a);
      n = $sscanf("7 8", "%d %*d", a); $display("T|21.3.4m|%0d %0d", n, a);
      n = $sscanf("A", "%c", a); $display("T|21.3.4n|%0d %0d", n, a);
      n = $sscanf("", "%d", a); $display("T|21.3.4o|%0d", n);
    end
    // 21.4 readmem / writemem
    begin int fd; fd = $fopen("c21_mem.txt", "w");
      $fwrite(fd, "// comment\n@2\n1A 2b\nx_F 4\n@0 11\n"); $fclose(fd);
      foreach (mem[k]) mem[k] = 32'hdead;
      $readmemh("c21_mem.txt", mem);
      $display("T|21.4a|%h %h %h %h %h %h", mem[0], mem[1], mem[2], mem[3], mem[4], mem[5]);
      fd = $fopen("c21_memb.txt", "w"); $fwrite(fd, "1010_0101\n11\n0\nzzzz1111\n"); $fclose(fd);
      $readmemb("c21_memb.txt", memb); $display("T|21.4b|%p", memb);
      foreach (memb[k]) memb[k] = 0;
      $readmemb("c21_memb.txt", memb, 1, 2); $display("T|21.4c|%p", memb);
      $readmemb("c21_memb.txt", memb, 3, 0); $display("T|21.4d|%p", memb);
      $writememh("c21_wm.txt", memb);
      begin logic [7:0] rb[0:3]; $readmemh("c21_wm.txt", rb); $display("T|21.4.1|%p", rb); end
      $writememb("c21_wmb.txt", memb, 1, 2);
      begin logic [7:0] rb2[0:3]; $readmemb("c21_wmb.txt", rb2, 1, 2); $display("T|21.4.1b|%p", rb2); end
      begin int dm[]; dm = new[3]; $readmemh("c21_mem.txt", dm); $display("T|21.4e|%p", dm); end
    end
    // 21.6 plusargs
    begin int val; string sv; real rv;
      $display("T|21.6a|%0d %0d", $test$plusargs("NOPE"), $test$plusargs("FO"));
      $display("T|21.6b|%0d", $value$plusargs("NUM=%d", val));
    end
    // strobe/monitor writing to file
    $fmonitor(1, "T|21.2.3|fmon v8=%h", v8);
    #1 v8 = 8'h11;
    #1 $fmonitoroff;
  end
  // 21.7 VCD
  initial begin
    $dumpfile("c21.vcd");
    $dumpvars(0, c21);
    #5 $dumpoff; #1 $dumpon; $dumpall; $dumpflush;
    #1 $dumplimit(100000);
  end
  final begin int fd; string l; int nl, hasdef, hasvar, hasdump, hast; nl = 0; hasdef = 0; hasvar = 0; hasdump = 0; hast = 0;
    $dumpflush;
    fd = $fopen("c21.vcd", "r");
    if (fd == 0) $display("T|21.7|no vcd");
    else begin while ($fgets(l, fd)) begin nl++;
        if (l.substr(0, 14) == "$enddefinitions") hasdef = 1;
        if (l.substr(0, 3) == "$var") hasvar++;
        if (l.substr(0, 7) == "$dumpoff" || l.substr(0, 6) == "$dumpon") hasdump++;
        if (l.substr(0, 0) == "#") hast++;
      end
      $display("T|21.7|vcd defs=%0d vars>5=%0d dumponoff=%0d times>2=%0d", hasdef, hasvar > 5, hasdump > 0, hast > 2);
    end
  end
endmodule
