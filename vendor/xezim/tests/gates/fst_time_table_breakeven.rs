//! The FST time table must survive break-even compression.
//!
//! `fst-writer`'s `write_time_table` chose between storing the delta-encoded
//! time table raw or zlib-compressed with `if compressed.len() > raw.len()`.
//! The comparison needs to be `>=`: a reader treats "compressed length ==
//! uncompressed length" as the sentinel meaning the section is stored RAW, so
//! when zlib output came out EXACTLY the same size as its input the writer
//! emitted compressed bytes while recording equal lengths. Readers then skipped
//! the inflate and parsed zlib's own header as varints.
//!
//! Nothing about the file looks wrong: the header, the block chain and every
//! length field validate, and the declared start/end times are correct. Only
//! the per-change timestamps are nonsense — the first two decode from the `78
//! 5e` zlib magic as 120 and 214 regardless of the design. gtkwave's `fst2vcd`
//! "succeeds" on such a file and prints times orders of magnitude off, so the
//! corruption reads as a simulator timing bug rather than a dump bug.
//!
//! Break-even is not exotic: it needs only a short run whose delta-encoded
//! table is small and incompressible. Nineteen irregular time steps do it.
//! Patched locally in `vendor/fst-writer`; drop that once upstream carries it.

use std::io::BufReader;

use fst_reader::{FstFilter, FstReader, FstSignalValue};

/// Irregular steps of 7..19ns, which is what makes the table incompressible.
fn source() -> (String, Vec<u64>) {
    let mut body = String::new();
    let mut acc: u64 = 0;
    let mut expect = Vec::new();
    for i in 0..19u64 {
        let d = 7 + i % 13;
        body.push_str(&format!("#{} x = {};", d, i % 2));
        acc += d;
        expect.push(acc * 1000); // ns source, ps precision
    }
    (
        format!(
            "`timescale 1ns/1ps\nmodule top;\n  logic x;\n  initial begin {} $finish; end\nendmodule\n",
            body
        ),
        expect,
    )
}

#[test]
fn fst_time_table_survives_breakeven_compression() {
    let (src, expect) = source();
    let mut sv = std::env::temp_dir();
    sv.push(format!("xezim_fst_be_{}.sv", std::process::id()));
    let mut fst = std::env::temp_dir();
    fst.push(format!("xezim_fst_be_{}.fst", std::process::id()));
    let _ = std::fs::remove_file(&sv);
    let _ = std::fs::remove_file(&fst);
    std::fs::write(&sv, src).unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top", "--fst"])
        .arg(&fst)
        .arg(&sv)
        .output()
        .expect("run xezim");
    assert!(out.status.success(), "xezim failed: {:?}", out);

    let file = std::fs::File::open(&fst).expect("open fst");
    let mut r = FstReader::open_and_read_time_table(BufReader::new(file))
        .expect("FST time table does not decode");
    r.read_hierarchy(|_| {}).expect("hierarchy");
    let mut times: Vec<u64> = Vec::new();
    r.read_signals(&FstFilter::all(), |t, _h, v| {
        let _ = match v {
            FstSignalValue::String(b) => String::from_utf8_lossy(b).to_string(),
            FstSignalValue::Real(x) => format!("{x}"),
        };
        if !times.contains(&t) {
            times.push(t);
        }
        Ok::<(), ()>(())
    })
    .expect("value changes");

    let _ = std::fs::remove_file(&sv);
    let _ = std::fs::remove_file(&fst);

    for want in &expect {
        assert!(
            times.contains(want),
            "FST lost time {want}; decoded {times:?}\n\
             (120 and 214 in that list are the zlib magic read as varints)"
        );
    }
    assert!(
        !times.contains(&120) && !times.contains(&214),
        "decoded times carry the zlib-header signature: {times:?}"
    );
}
