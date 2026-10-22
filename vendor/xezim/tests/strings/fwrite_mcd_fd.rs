//! IEEE 1800-2023 §21.3.1/§21.3.2 `$fopen`/`$fwrite` multichannel-descriptor
//! (MCD) and file-descriptor (FD) addressing.
//!
//! Before this fix xezim treated every file value as a small plain integer and
//! wrote only when that integer was a live table key. UVM writes through
//! `UVM_STDOUT = 32'h8000_0001` (an FD with bit 31 set), so its report/printer
//! output silently vanished. The fix distinguishes the two modes by bit 31:
//!   * MCD (bit 31 clear): each set bit selects a channel; bit 0 = stdout.
//!     `$fopen(filename)` returns an MCD.
//!   * FD  (bit 31 set):   lower bits are the index; STDIN/STDOUT/STDERR are
//!     0x8000_0000/1/2. `$fopen(filename, type)` returns an FD.

use xezim::simulate;

/// A per-test temp subdir so parallel test runs never collide.
fn subdir(test: &str) -> String {
    let dir =
        std::env::temp_dir().join(format!("xezim_fwrite_mcd_{}_{}", std::process::id(), test));
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().into_owned()
}

fn m(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

/// `$fopen(filename)` (single-arg) returns an MCD: a single-bit power of two,
/// and bit 0 is reserved for stdout, so the first file is bit 1 (value 2).
#[test]
fn fopen_single_arg_returns_mcd_bit_pattern() {
    let d = subdir("mcd_bits");
    let src = format!(
        r#"
module tb;
  integer m1, m2;
  initial begin
    m1 = $fopen("{d}/a.txt");
    m2 = $fopen("{d}/b.txt");
    $fclose(m1);
    $fclose(m2);
  end
endmodule
"#,
        d = d,
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    assert_eq!(m(&sim, "m1"), 2, "first MCD file is bit 1 (value 2)");
    assert_eq!(m(&sim, "m2"), 4, "second MCD file is bit 2 (value 4)");
    let _ = std::fs::remove_dir_all(&d);
}

/// `$fopen(filename, type)` (two-arg) returns an FD with bit 31 set; the first
/// opened FD is index 3 (0/1/2 are STDIN/STDOUT/STDERR).
#[test]
fn fopen_two_arg_returns_fd_with_bit31_set() {
    let d = subdir("fd_bit31");
    let src = format!(
        r#"
module tb;
  integer fd;
  initial begin
    fd = $fopen("{d}/fd.txt", "w");
    $fclose(fd);
  end
endmodule
"#,
        d = d,
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    assert_eq!(
        m(&sim, "fd"),
        0x8000_0003,
        "first FD is 0x8000_0003 (bit 31 set, index 3)"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// Writing a combined MCD (stdout | file1 | file2) hits ALL three: stdout
/// gets the line and BOTH files contain it.
#[test]
fn mcd_broadcast_writes_to_all_channels() {
    let d = subdir("broadcast");
    let src = format!(
        r#"
module tb;
  integer m1, m2, all;
  initial begin
    m1 = $fopen("{d}/m1.txt");
    m2 = $fopen("{d}/m2.txt");
    all = 1 | m1 | m2;   // stdout | file1 | file2
    $fwrite(all, "BROADCAST\n");
    $fclose(m1);
    $fclose(m2);
  end
endmodule
"#,
        d = d,
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    // stdout (MCD bit 0) received the line
    let outs: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert!(
        outs.iter().any(|s| s.contains("BROADCAST")),
        "stdout missing broadcast: {:?}",
        outs
    );
    // both files received it
    assert_eq!(
        std::fs::read_to_string(format!("{}/m1.txt", d)).unwrap(),
        "BROADCAST\n"
    );
    assert_eq!(
        std::fs::read_to_string(format!("{}/m2.txt", d)).unwrap(),
        "BROADCAST\n"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// `$fwrite` to FD STDOUT (0x8000_0001) and MCD bit 0 (value 1) both reach
/// stdout; this is the UVM case: `UVM_STDOUT = 32'h8000_0001`.
#[test]
fn fwrite_to_fd_stdout_and_mcd_bit0_reach_stdout() {
    let src = r#"
module tb;
  initial begin
    $fwrite(32'h8000_0001, "FD_STDOUT\n");
    $fwrite(1, "MCD_STDOUT\n");
  end
endmodule
"#;
    let sim = simulate(src, 1000).expect("simulate failed");
    let outs: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert!(
        outs.iter().any(|s| s.contains("FD_STDOUT")),
        "FD STDOUT (0x8000_0001) must reach stdout: {:?}",
        outs
    );
    assert!(
        outs.iter().any(|s| s.contains("MCD_STDOUT")),
        "MCD bit 0 (stdout) must reach stdout: {:?}",
        outs
    );
}

/// FD-mode round trip: write then read back through a file descriptor.
#[test]
fn fd_mode_round_trip_read() {
    let d = subdir("roundtrip");
    let src = format!(
        r#"
module tb;
  integer fd, n;
  string s;
  initial begin
    fd = $fopen("{d}/rt.txt", "w");
    $fwrite(fd, "round-trip\n");
    $fclose(fd);
    fd = $fopen("{d}/rt.txt", "r");
    n = $fgets(s, fd);
    $fclose(fd);
  end
endmodule
"#,
        d = d,
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    assert_eq!(
        std::fs::read_to_string(format!("{}/rt.txt", d)).unwrap(),
        "round-trip\n"
    );
    assert_eq!(m(&sim, "n"), 11, "fgets byte count");
    let _ = std::fs::remove_dir_all(&d);
}

/// `$fclose` of an FD frees its index; the next `$fopen` reuses it.
#[test]
fn fclose_reuses_fd_index() {
    let d = subdir("reuse");
    let src = format!(
        r#"
module tb;
  integer fd;
  initial begin
    fd = $fopen("{d}/r1.txt", "w");
    $fclose(fd);
    fd = $fopen("{d}/r2.txt", "w");   // reuses index 3
    $fclose(fd);
  end
endmodule
"#,
        d = d,
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    assert_eq!(m(&sim, "fd"), 0x8000_0003, "FD index 3 reused after close");
    let _ = std::fs::remove_dir_all(&d);
}
