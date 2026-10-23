//! All three waveform sinks must date a change the same way.
//!
//! `dump_write_changes` fans out to VCD, XTrace and FST, so a scheduling bug
//! that misdates a change misdates it in all three — and a fix in that path
//! silently fixes all three too. Both directions were exercised for real: four
//! separate holes let time slots pass without a postponed region, and the
//! regression tests written for them only ever checked VCD. Nothing would have
//! caught FST or XTrace drifting away from it later.
//!
//! The design is the one the last of those holes was found with: a `#delay`
//! inside an edge block, with the write in the SAME process immediately after
//! the delay resumes. `sig` changes at 100ns and 600ns, and every sink must
//! say so.

use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::process::Command;

use fst_reader::{FstFilter, FstHierarchyEntry, FstReader, FstSignalValue};

const SRC: &str = r#"
`timescale 1ns/1ps
module top;
  logic tick = 0;
  logic sig  = 0;
  always #50 tick = ~tick;
  always @(posedge tick) begin
    #50  sig = 1;     // resumes and writes at t=100ns
    #500 sig = 0;     // and at t=600ns
  end
  initial begin
    $dumpfile("@VCD@");
    $dumpvars(0, top);
    #900 $finish;
  end
endmodule
"#;

fn tmp(stem: &str, ext: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "xezim_fmt_agree_{}_{}.{}",
        stem,
        std::process::id(),
        ext
    ));
    let _ = fs::remove_file(&p);
    p
}

/// Absolute `#time` records in a VCD.
fn vcd_stamps(text: &str) -> Vec<u64> {
    text.lines()
        .filter_map(|l| l.strip_prefix('#'))
        .filter_map(|t| t.trim().parse().ok())
        .collect()
}

/// XTrace `T,+<delta>` records accumulated into absolute times.
fn xtrace_stamps(text: &str) -> Vec<u64> {
    let mut t: u64 = 0;
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(d) = line.trim().strip_prefix("T,+") {
            if let Ok(d) = d.parse::<u64>() {
                t += d;
                out.push(t);
            }
        }
    }
    out
}

/// Times at which `sig` takes a new value in an FST.
fn fst_sig_changes(path: &std::path::Path) -> Vec<(u64, String)> {
    let file = fs::File::open(path).expect("open fst");
    let mut reader = FstReader::open_and_read_time_table(BufReader::new(file))
        .expect("FST header does not decode");
    let mut handle_of_sig = None;
    reader
        .read_hierarchy(|e| {
            if let FstHierarchyEntry::Var { name, handle, .. } = e {
                if name == "sig" {
                    handle_of_sig = Some(handle.get_index());
                }
            }
        })
        .expect("FST hierarchy does not decode");
    let want = handle_of_sig.expect("no `sig` in the FST hierarchy");

    let mut changes: HashMap<usize, Vec<(u64, String)>> = HashMap::new();
    reader
        .read_signals(&FstFilter::all(), |time, handle, value| {
            let rendered = match value {
                FstSignalValue::String(b) => String::from_utf8_lossy(b).to_string(),
                FstSignalValue::Real(f) => format!("r{}", f),
            };
            changes
                .entry(handle.get_index())
                .or_default()
                .push((time, rendered));
            Ok::<(), ()>(())
        })
        .expect("FST value changes do not decode");
    changes.remove(&want).unwrap_or_default()
}

#[test]
fn vcd_fst_and_xtrace_date_a_change_identically() {
    let sv = tmp("src", "sv");
    let vcd = tmp("out", "vcd");
    let fst = tmp("out", "fst");
    let xt = tmp("out", "xtrace");

    fs::write(&sv, SRC.replace("@VCD@", vcd.to_str().unwrap())).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top", "--fst"])
        .arg(&fst)
        .arg("--xtrace")
        .arg(&xt)
        .arg(&sv)
        .output()
        .expect("run xezim");
    assert!(out.status.success(), "xezim failed: {:?}", out);

    let vcd_t = vcd_stamps(&fs::read_to_string(&vcd).expect("read vcd"));
    let xt_t = xtrace_stamps(&fs::read_to_string(&xt).expect("read xtrace"));
    let fst_c = fst_sig_changes(&fst);

    for p in [&sv, &vcd, &fst, &xt] {
        let _ = fs::remove_file(p);
    }

    // 100ns / 600ns in the 1ps precision every sink records in.
    for t in [100_000u64, 600_000] {
        assert!(vcd_t.contains(&t), "VCD has no #{t}; stamps {vcd_t:?}");
        assert!(xt_t.contains(&t), "XTrace has no T at {t}; stamps {xt_t:?}");
        assert!(
            fst_c.iter().any(|(ft, _)| *ft == t),
            "FST has no `sig` change at {t}; changes {fst_c:?}"
        );
    }
}
