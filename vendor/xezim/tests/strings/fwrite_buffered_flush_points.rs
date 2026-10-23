//! `$fwrite`/`$fdisplay` output to a regular file is buffered. These pin the
//! points where the buffered bytes must already be in the file: a read or
//! tell on the same handle, a second handle or `$readmemh` on the same path,
//! `$system`, `$fflush`, and the end of the run with or without `$fclose`.

use xezim::simulate;

fn subdir(test: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_fwbuf_{}_{}", std::process::id(), test));
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().into_owned()
}

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .collect()
}

/// A second handle opened for reading while the writer is still open sees
/// every line written so far, and later writes stay in program order.
#[test]
fn second_handle_reads_what_the_writer_wrote() {
    let d = subdir("second_handle");
    let src = format!(
        r#"
module tb;
  integer w, r, n;
  string line;
  initial begin
    w = $fopen("{d}/log.txt", "w");
    $fdisplay(w, "first");
    $fdisplay(w, "second");
    r = $fopen("{d}/log.txt", "r");
    n = $fgets(line, r);
    $display("R1 %0d %s", n, line);
    $fdisplay(w, "third");
    n = $fgets(line, r);
    $display("R2 %0d %s", n, line);
    n = $fgets(line, r);
    $display("R3 %0d %s", n, line);
    $fclose(r);
    $fclose(w);
  end
endmodule
"#
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"R1 6 first".to_string()), "{:?}", o);
    assert!(o.contains(&"R2 7 second".to_string()), "{:?}", o);
    assert!(o.contains(&"R3 6 third".to_string()), "{:?}", o);
    let _ = std::fs::remove_dir_all(&d);
}

/// A read-write handle: `$ftell` counts the buffered bytes, and `$rewind`
/// plus `$fgets` on the same handle read them back.
#[test]
fn same_handle_tell_and_read_back() {
    let d = subdir("same_handle");
    let src = format!(
        r#"
module tb;
  integer f, pos, n;
  string line;
  initial begin
    f = $fopen("{d}/rw.txt", "w+");
    $fwrite(f, "hello\n");
    $fwrite(f, "world\n");
    pos = $ftell(f);
    $display("POS %0d", pos);
    $rewind(f);
    n = $fgets(line, f);
    $display("L1 %s", line);
    n = $fgets(line, f);
    $display("L2 %s", line);
    $fclose(f);
  end
endmodule
"#
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"POS 12".to_string()), "{:?}", o);
    assert!(o.contains(&"L1 hello".to_string()), "{:?}", o);
    assert!(o.contains(&"L2 world".to_string()), "{:?}", o);
    let _ = std::fs::remove_dir_all(&d);
}

/// `$readmemh` of a file the design is still writing loads what was written.
#[test]
fn readmemh_sees_unclosed_writes() {
    let d = subdir("readmem");
    let src = format!(
        r#"
module tb;
  integer f;
  reg [7:0] mem [0:3];
  initial begin
    f = $fopen("{d}/mem.hex", "w");
    $fdisplay(f, "a1");
    $fdisplay(f, "b2");
    $fdisplay(f, "c3");
    $fdisplay(f, "d4");
    $readmemh("{d}/mem.hex", mem);
    $display("MEM %h %h %h %h", mem[0], mem[1], mem[2], mem[3]);
    $fclose(f);
  end
endmodule
"#
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"MEM a1 b2 c3 d4".to_string()), "{:?}", o);
    let _ = std::fs::remove_dir_all(&d);
}

/// `$system` runs after the buffered output reached the file.
#[test]
fn system_sees_unclosed_writes() {
    let d = subdir("system");
    let src = format!(
        r#"
module tb;
  integer f;
  initial begin
    f = $fopen("{d}/a.txt", "w");
    $fdisplay(f, "one");
    $fwrite(f, "two\n");
    $system("cp {d}/a.txt {d}/copy.txt");
    $fdisplay(f, "three");
    $fclose(f);
  end
endmodule
"#
    );
    simulate(&src, 1000).expect("simulate failed");
    assert_eq!(
        std::fs::read_to_string(format!("{}/copy.txt", d)).unwrap(),
        "one\ntwo\n"
    );
    assert_eq!(
        std::fs::read_to_string(format!("{}/a.txt", d)).unwrap(),
        "one\ntwo\nthree\n"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// `$fflush(fd)` pushes the output out while the handle stays open; a copy
/// taken by `$system` right after it is complete.
#[test]
fn fflush_then_copy_is_complete() {
    let d = subdir("fflush");
    let src = format!(
        r#"
module tb;
  integer f;
  initial begin
    f = $fopen("{d}/f.txt", "w");
    $fdisplay(f, "alpha");
    $fflush(f);
    $fdisplay(f, "beta");
    $fflush();
    $system("cp {d}/f.txt {d}/snap.txt");
    $fclose(f);
  end
endmodule
"#
    );
    simulate(&src, 1000).expect("simulate failed");
    assert_eq!(
        std::fs::read_to_string(format!("{}/snap.txt", d)).unwrap(),
        "alpha\nbeta\n"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// No `$fclose`: output is in the file after `$finish`, and after a run that
/// simply ran out of time.
#[test]
fn unclosed_handle_is_complete_at_end_of_run() {
    let d = subdir("end_of_run");
    let fin = format!(
        r#"
module tb;
  integer f;
  initial begin
    f = $fopen("{d}/fin.txt", "w");
    $fdisplay(f, "before finish");
    #10 $finish;
  end
endmodule
"#
    );
    // The simulator (and its handles) stays alive while the file is read.
    let sim = simulate(&fin, 1000).expect("simulate failed");
    assert_eq!(
        std::fs::read_to_string(format!("{}/fin.txt", d)).unwrap(),
        "before finish\n"
    );
    drop(sim);
    let timeout = format!(
        r#"
module tb;
  integer f, i;
  reg clk = 0;
  always #5 clk = ~clk;
  initial begin
    f = $fopen("{d}/tick.txt", "w");
    i = 0;
  end
  always @(posedge clk) begin
    $fdisplay(f, "tick %0d", i);
    i = i + 1;
  end
endmodule
"#
    );
    let sim = simulate(&timeout, 42).expect("simulate failed");
    assert_eq!(
        std::fs::read_to_string(format!("{}/tick.txt", d)).unwrap(),
        "tick 0\ntick 1\ntick 2\ntick 3\n"
    );
    drop(sim);
    let _ = std::fs::remove_dir_all(&d);
}

/// Two append handles on one file interleave in program order, and a volume
/// larger than the buffer arrives whole and in order.
#[test]
fn two_handles_and_large_volume_keep_order() {
    let d = subdir("order");
    let src = format!(
        r#"
module tb;
  integer a, b, big, i;
  initial begin
    a = $fopen("{d}/both.txt", "a");
    b = $fopen("{d}/both.txt", "a");
    $fdisplay(a, "a1");
    $fdisplay(b, "b1");
    $fdisplay(a, "a2");
    $fdisplay(b, "b2");
    $fclose(a);
    $fclose(b);
    big = $fopen("{d}/big.txt", "w");
    for (i = 0; i < 20000; i = i + 1)
      $fdisplay(big, "line %0d", i);
    $fclose(big);
  end
endmodule
"#
    );
    simulate(&src, 1000).expect("simulate failed");
    assert_eq!(
        std::fs::read_to_string(format!("{}/both.txt", d)).unwrap(),
        "a1\nb1\na2\nb2\n"
    );
    let big = std::fs::read_to_string(format!("{}/big.txt", d)).unwrap();
    let want: String = (0..20000).map(|i| format!("line {}\n", i)).collect();
    assert!(
        big == want,
        "big.txt: {} bytes, want {}",
        big.len(),
        want.len()
    );
    let _ = std::fs::remove_dir_all(&d);
}
