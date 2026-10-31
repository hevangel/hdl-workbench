//! Transcode the VCD that Xezim captured into an FST file held in memory.
//!
//! Native Xezim writes FST through a file path and a background thread, and a
//! browser has neither. The VCD already comes from the simulator's own dump, so
//! converting it keeps one source of truth for what is traced.

use fst_writer::{
    FstBodyWriter, FstFileType, FstHeaderWriter, FstInfo, FstScopeType, FstSignalId,
    FstSignalType, FstVarDirection, FstVarType,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Cursor, Seek, SeekFrom, Write};
use std::rc::Rc;

/// A seekable in-memory sink the caller can read after `fst-writer` consumes it.
#[derive(Clone, Default)]
struct SharedBuffer(Rc<RefCell<Cursor<Vec<u8>>>>);

impl Write for SharedBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.borrow_mut().flush()
    }
}

impl Seek for SharedBuffer {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.0.borrow_mut().seek(position)
    }
}

struct Signal {
    id: FstSignalId,
    width: usize,
    real: bool,
}

/// `$timescale 1 ns $end` or `$timescale 10ps $end` to a power of ten in seconds.
fn timescale_exponent(tokens: &[&str]) -> Result<i8, String> {
    let text: String = tokens.concat();
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(|| format!("Unrecognized VCD timescale '{text}'."))?;
    let (magnitude, unit) = text.split_at(split);
    let magnitude_exponent = match magnitude {
        "1" => 0,
        "10" => 1,
        "100" => 2,
        _ => return Err(format!("Unrecognized VCD timescale '{text}'.")),
    };
    let unit_exponent = match unit {
        "s" => 0,
        "ms" => -3,
        "us" => -6,
        "ns" => -9,
        "ps" => -12,
        "fs" => -15,
        _ => return Err(format!("Unrecognized VCD timescale '{text}'.")),
    };
    Ok(unit_exponent + magnitude_exponent)
}

fn scope_type(kind: &str) -> FstScopeType {
    match kind {
        "begin" => FstScopeType::Begin,
        "fork" => FstScopeType::Fork,
        "task" => FstScopeType::Task,
        "function" => FstScopeType::Function,
        _ => FstScopeType::Module,
    }
}

fn var_type(kind: &str) -> FstVarType {
    match kind {
        "reg" => FstVarType::Reg,
        "integer" => FstVarType::Integer,
        "real" | "realtime" => FstVarType::Real,
        "time" => FstVarType::Time,
        "event" => FstVarType::Event,
        "parameter" => FstVarType::Parameter,
        _ => FstVarType::Wire,
    }
}

/// Fit a VCD vector value to the declared width, applying the VCD left-extension
/// rule (0/1 extend with 0, x and z extend with themselves) and keeping the low
/// bits of an over-long value.
fn fit_bits(bits: &str, width: usize) -> Vec<u8> {
    let bytes = bits.as_bytes();
    if bytes.len() >= width {
        return bytes[bytes.len() - width..].to_ascii_lowercase();
    }
    let fill = match bytes.first() {
        Some(b'x' | b'X') => b'x',
        Some(b'z' | b'Z') => b'z',
        _ => b'0',
    };
    let mut out = vec![fill; width - bytes.len()];
    out.extend(bytes.to_ascii_lowercase());
    out
}

fn fst_error(error: fst_writer::FstWriteError) -> String {
    format!("FST conversion failed: {error}")
}

/// Convert a complete VCD document into FST bytes.
pub fn vcd_to_fst(vcd: &str) -> Result<Vec<u8>, String> {
    let mut tokens = vcd.split_whitespace();
    let mut exponent = -9;
    let mut signals: HashMap<&str, Signal> = HashMap::new();
    let sink = SharedBuffer::default();
    let info_for = |exponent| FstInfo {
        start_time: 0,
        timescale_exponent: exponent,
        version: "HDL Workbench (Xezim)".to_string(),
        date: String::new(),
        file_type: FstFileType::Verilog,
    };

    // The timescale precedes the first scope, but the header writer needs it up
    // front, so read the declarations once and replay them into the writer.
    enum Item<'a> {
        Scope(&'a str, &'a str),
        Up,
        Var(&'a str, usize, &'a str, String),
    }
    let mut items: Vec<Item> = Vec::new();
    loop {
        let Some(token) = tokens.next() else {
            return Err("The VCD ended before $enddefinitions.".to_string());
        };
        match token {
            "$enddefinitions" => {
                for end in tokens.by_ref() {
                    if end == "$end" {
                        break;
                    }
                }
                break;
            }
            "$timescale" => {
                let parts: Vec<&str> = tokens.by_ref().take_while(|part| *part != "$end").collect();
                exponent = timescale_exponent(&parts)?;
            }
            "$scope" => {
                let parts: Vec<&str> = tokens.by_ref().take_while(|part| *part != "$end").collect();
                let [kind, name, ..] = parts[..] else {
                    return Err("Malformed $scope in the VCD.".to_string());
                };
                items.push(Item::Scope(kind, name));
            }
            "$upscope" => {
                for end in tokens.by_ref() {
                    if end == "$end" {
                        break;
                    }
                }
                items.push(Item::Up);
            }
            "$var" => {
                let parts: Vec<&str> = tokens.by_ref().take_while(|part| *part != "$end").collect();
                let [kind, width, id, name @ ..] = &parts[..] else {
                    return Err("Malformed $var in the VCD.".to_string());
                };
                if name.is_empty() {
                    return Err("Malformed $var in the VCD.".to_string());
                }
                let width: usize = width
                    .parse()
                    .map_err(|_| format!("Invalid $var width '{width}' in the VCD."))?;
                items.push(Item::Var(kind, width, id, name.concat()));
            }
            _ if token.starts_with('$') => {
                // $date, $version, $comment and anything else we do not need.
                for end in tokens.by_ref() {
                    if end == "$end" {
                        break;
                    }
                }
            }
            _ => return Err(format!("Unexpected token '{token}' in the VCD header.")),
        }
    }

    let mut header = FstHeaderWriter::new(sink.clone(), &info_for(exponent)).map_err(fst_error)?;
    let mut depth = 0usize;
    for item in items {
        match item {
            Item::Scope(kind, name) => {
                header.scope(name, "", scope_type(kind)).map_err(fst_error)?;
                depth += 1;
            }
            Item::Up => {
                if depth > 0 {
                    header.up_scope().map_err(fst_error)?;
                    depth -= 1;
                }
            }
            Item::Var(kind, width, id, name) => {
                let real = matches!(kind, "real" | "realtime");
                let alias = signals.get(id).map(|signal| signal.id);
                let signal_type = if real {
                    FstSignalType::real()
                } else {
                    FstSignalType::bit_vec(width.max(1) as u32)
                };
                let fst_id = header
                    .var(&name, signal_type, var_type(kind), FstVarDirection::Implicit, alias)
                    .map_err(fst_error)?;
                signals.entry(id).or_insert(Signal {
                    id: fst_id,
                    width: width.max(1),
                    real,
                });
            }
        }
    }
    while depth > 0 {
        header.up_scope().map_err(fst_error)?;
        depth -= 1;
    }
    let mut body: FstBodyWriter<SharedBuffer> = header.finish().map_err(fst_error)?;

    let mut timed = false;
    let change = |body: &mut FstBodyWriter<SharedBuffer>,
                      timed: &mut bool,
                      id: &str,
                      value: &str,
                      real: Option<f64>|
     -> Result<(), String> {
        let Some(signal) = signals.get(id) else {
            return Ok(());
        };
        if !*timed {
            body.time_change(0).map_err(fst_error)?;
            *timed = true;
        }
        let bytes = if signal.real {
            real.unwrap_or(0.0).to_le_bytes().to_vec()
        } else {
            fit_bits(value, signal.width)
        };
        body.signal_change(signal.id, &bytes).map_err(fst_error)
    };

    while let Some(token) = tokens.next() {
        if let Some(time) = token.strip_prefix('#') {
            let time: u64 = time
                .parse()
                .map_err(|_| format!("Invalid VCD timestamp '{token}'."))?;
            body.time_change(time).map_err(fst_error)?;
            timed = true;
        } else if token == "$comment" {
            for end in tokens.by_ref() {
                if end == "$end" {
                    break;
                }
            }
        } else if token.starts_with('$') {
            // $dumpvars/$dumpall/$dumpon/$dumpoff/$end only delimit value changes.
        } else if let Some(bits) = token.strip_prefix(['b', 'B']) {
            let id = tokens
                .next()
                .ok_or_else(|| "A VCD vector change had no identifier.".to_string())?;
            change(&mut body, &mut timed, id, bits, None)?;
        } else if let Some(number) = token.strip_prefix(['r', 'R']) {
            let id = tokens
                .next()
                .ok_or_else(|| "A VCD real change had no identifier.".to_string())?;
            let value: f64 = number
                .parse()
                .map_err(|_| format!("Invalid VCD real value '{number}'."))?;
            change(&mut body, &mut timed, id, "", Some(value))?;
        } else if token.len() >= 2 && matches!(token.as_bytes()[0], b'0' | b'1' | b'x' | b'X' | b'z' | b'Z' | b'u' | b'U' | b'-' | b'h' | b'H' | b'l' | b'L' | b'w' | b'W') {
            let (value, id) = token.split_at(1);
            change(&mut body, &mut timed, id, value, None)?;
        } else {
            return Err(format!("Unexpected token '{token}' in the VCD body."));
        }
    }
    body.finish().map_err(fst_error)?;
    let bytes = sink.0.borrow().get_ref().clone();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VCD: &str = "$date x $end\n$timescale 1 ns $end\n$scope module top $end\n\
$var wire 1 ! clk $end\n$var reg 4 \" count [3:0] $end\n$var real 1 # temp $end\n\
$var wire 1 $ alias $end\n$upscope $end\n$enddefinitions $end\n#0\n$dumpvars\n0!\nb0000 \"\nr1.5 #\n$end\n#5\n1!\nb1 \"\n#10\n0!\nbx \"\nr2.25 #\n";

    #[test]
    fn converts_a_vcd_with_vectors_reals_and_aliases() {
        let fst = vcd_to_fst(VCD).unwrap();
        // The FST header block is block type 0 with a fixed-size header.
        assert_eq!(fst[0], 0, "FST files begin with the header block");
        assert!(fst.len() > 329, "header plus hierarchy and value changes: {}", fst.len());
    }

    #[test]
    fn rejects_unknown_timescale() {
        let vcd = "$timescale 7 ns $end\n$enddefinitions $end\n";
        assert!(vcd_to_fst(vcd).unwrap_err().contains("timescale"));
    }

    #[test]
    fn rejects_truncated_headers() {
        assert!(vcd_to_fst("$timescale 1 ns $end\n").is_err());
    }

    #[test]
    fn extends_short_vector_values_like_vcd() {
        assert_eq!(fit_bits("1", 4), b"0001");
        assert_eq!(fit_bits("x", 3), b"xxx");
        assert_eq!(fit_bits("10110", 4), b"0110");
    }
}
