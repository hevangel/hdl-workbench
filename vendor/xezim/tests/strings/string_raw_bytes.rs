//! IEEE 1800-2017 §6.16 / §5.9.1: a string is a sequence of BYTES. The text
//! form of a string value holds one Latin-1 char per byte, but several paths
//! treated it as UTF-8: a character select, `substr`, `putc`, a character
//! write, `foreach`, `$fgets` and the `compare` family read or rebuilt the
//! string through its UTF-8 encoding, so every byte >= 0x80 (`"\xff"`) became
//! two bytes or U+FFFD, and `toupper`/`tolower` changed Latin-1 letters. The
//! preprocessor also double-encoded non-ASCII source text (`"é"` was four
//! bytes). Expected values are the reference simulator's.

use xezim::simulate;

const SRC: &str = r#"
module top;
  string s, t; int fd, i, n; string line;
  initial begin
    s = "\xff"; $display("T|xff len=%0d b0=%0d", s.len(), s.getc(0));
    s = "\377"; $display("T|o377 len=%0d b0=%0d", s.len(), s.getc(0));
    s = "\x8"; $display("T|x8 len=%0d b0=%0d", s.len(), s.getc(0));
    s = "\x80\xC3\xA9"; $display("T|x80c3a9 len=%0d b=%0d,%0d,%0d", s.len(), s.getc(0), s.getc(1), s.getc(2));
    s = "\v\f\a"; $display("T|vfa len=%0d b=%0d,%0d,%0d", s.len(), s.getc(0), s.getc(1), s.getc(2));
    s = "é\xff"; $display("T|utf8mix len=%0d b=%0d,%0d,%0d", s.len(), s.getc(0), s.getc(1), s.getc(2));
    s = "é"; $display("T|utf8 len=%0d b=%0d,%0d s0=%0d", s.len(), s.getc(0), s.getc(1), s[0]);
    s = "\1\12\123\1234"; $display("T|oct len=%0d b=%0d,%0d,%0d,%0d", s.len(), s.getc(0), s.getc(1), s.getc(2), s.getc(3));
    s = "\xffA"; $display("T|xffA len=%0d b=%0d,%0d s0=%0d", s.len(), s.getc(0), s.getc(1), s[0]);
    t = $sformatf("%s|", "\xff"); $display("T|sfmt len=%0d b=%0d", t.len(), t.getc(0));
    $display("T|lit bits=%0d v=%0d h=%h", $bits("\xff"), "\xff", "\xff\x80");
    s = {"\xff", "\x01"}; $display("T|cat len=%0d b=%0d", s.len(), s.getc(0));
    s = "a\xe9\xffb"; t = s.substr(1, 2); $display("T|sub len=%0d b=%0d,%0d", t.len(), t.getc(0), t.getc(1));
    s = "\xe9a"; $display("T|case up=%0d,%0d lo=%0d", s.toupper().getc(0), s.toupper().getc(1), s.tolower().getc(0));
    s = "a\xffb"; s.putc(0, 8'hfe); $display("T|putc len=%0d b=%0d,%0d", s.len(), s.getc(0), s.getc(1));
    s = "\xff\xfe"; s[1] = "Z"; $display("T|wr len=%0d b=%0d,%0d", s.len(), s.getc(0), s.getc(1));
    s = "\xff\xfe"; n = 0; foreach (s[j]) n++; $display("T|foreach n=%0d", n);
    fd = $fopen("@BIN@", "wb"); $fwrite(fd, "%s\n", "\xff\x80"); $fclose(fd);
    fd = $fopen("@BIN@", "r"); i = $fgets(line, fd); $fclose(fd);
    $display("T|fgets r=%0d len=%0d b=%0d,%0d", i, line.len(), line.getc(0), line.getc(1));
  end
endmodule
"#;

#[test]
fn strings_hold_raw_bytes() {
    let bin = std::env::temp_dir().join(format!("xezim_str_bytes_{}.bin", std::process::id()));
    let src = SRC.replace("@BIN@", bin.to_str().expect("utf-8 temp path"));
    let sim = simulate(&src, 100).expect("simulate failed");
    let _ = std::fs::remove_file(&bin);
    let got: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect();
    let want = [
        "T|xff len=1 b0=-1",
        "T|o377 len=1 b0=-1",
        "T|x8 len=1 b0=8",
        "T|x80c3a9 len=3 b=-128,-61,-87",
        "T|vfa len=3 b=11,12,7",
        "T|utf8mix len=3 b=-61,-87,-1",
        "T|utf8 len=2 b=-61,-87 s0=-61",
        "T|oct len=5 b=1,10,83,83",
        "T|xffA len=2 b=-1,65 s0=-1",
        "T|sfmt len=2 b=-1",
        "T|lit bits=8 v=255 h=ff80",
        "T|cat len=2 b=-1",
        "T|sub len=2 b=-23,-1",
        "T|case up=-23,65 lo=-23",
        "T|putc len=3 b=-2,-1",
        "T|wr len=2 b=-1,90",
        "T|foreach n=2",
        "T|fgets r=3 len=3 b=-1,-128",
    ];
    assert_eq!(got, want);
}
