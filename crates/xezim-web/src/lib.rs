use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;
const MAX_PROJECT_BYTES: usize = 16 * 1024 * 1024;
const MAX_VCD_BYTES: usize = 64 * 1024 * 1024;
const MAX_TOP_MODULE_BYTES: usize = 512;
const MAX_TIME_NS: u64 = 10_000_000;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum RunStatus {
    Completed,
    Failed,
    ResourceLimit,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic {
    severity: &'static str,
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunResult {
    schema_version: u32,
    status: RunStatus,
    max_time_ns: String,
    last_time_ticks: String,
    finished: bool,
    saw_fatal: bool,
    stdout: String,
    diagnostics: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    waveform: Option<WaveformMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vcd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_outputs: Option<Vec<FileOutput>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WaveformMetadata {
    format: &'static str,
    size_bytes: usize,
    complete: bool,
    timescale: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileOutput {
    path: String,
    contents: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFile {
    path: String,
    contents: String,
}

impl RunResult {
    fn error(code: &'static str, message: impl Into<String>, max_time_ns: u64) -> Self {
        Self {
            schema_version: 1,
            status: RunStatus::Failed,
            max_time_ns: max_time_ns.to_string(),
            last_time_ticks: "0".to_string(),
            finished: false,
            saw_fatal: false,
            stdout: String::new(),
            diagnostics: vec![Diagnostic {
                severity: "error",
                code,
                message: message.into(),
            }],
            snapshot_id: None,
            waveform: None,
            vcd: None,
            file_outputs: None,
        }
    }
}

#[derive(Default)]
struct VcdCapture {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}

struct BoundedVcdWriter(Arc<Mutex<VcdCapture>>);

impl Write for BoundedVcdWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut capture = self
            .0
            .lock()
            .map_err(|_| io::Error::other("VCD capture lock poisoned"))?;
        if capture.bytes.len().saturating_add(bytes.len()) > capture.limit {
            capture.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "VCD byte limit exceeded",
            ));
        }
        capture.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl VcdCapture {
    fn new(limit: usize) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            bytes: Vec::new(),
            limit,
            exceeded: false,
        }))
    }
}

fn encode(result: RunResult) -> String {
    serde_json::to_string(&result).unwrap_or_else(|_| {
        r#"{"schemaVersion":1,"status":"failed","maxTimeNs":"0","lastTimeTicks":"0","finished":false,"sawFatal":false,"stdout":"","diagnostics":[{"severity":"error","code":"XEZIM_WEB_SERIALIZE","message":"Could not encode the simulator result."}]}"#.to_string()
    })
}

fn truncate_utf8(value: &str, limit: usize) -> (String, bool) {
    if value.len() <= limit {
        return (value.to_string(), false);
    }
    let mut end = limit;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_string(), true)
}

fn vcd_timescale(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let start = text.find("$timescale")? + "$timescale".len();
    let end = text[start..].find("$end")? + start;
    Some(
        text[start..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn run_project(
    files_json: &str,
    snapshot_id: &str,
    top_module: &str,
    max_time_ns: u64,
    max_output_bytes: usize,
    max_vcd_bytes: usize,
) -> String {
    let parse_error = |code, message: String| encode(RunResult::error(code, message, max_time_ns));
    if snapshot_id.is_empty() || snapshot_id.len() > 256 {
        return parse_error(
            "XEZIM_WEB_SNAPSHOT",
            "Snapshot ID must contain 1 to 256 bytes.".into(),
        );
    }
    if top_module.is_empty() || top_module.len() > MAX_TOP_MODULE_BYTES {
        return parse_error(
            "XEZIM_WEB_TOP_MODULE",
            "Top module must contain 1 to 512 UTF-8 bytes.".into(),
        );
    }
    if max_time_ns > MAX_TIME_NS {
        return parse_error(
            "XEZIM_WEB_TIME_LIMIT",
            format!("Maximum simulation time is {} ns.", MAX_TIME_NS),
        );
    }
    let files: Vec<ProjectFile> = match serde_json::from_str(files_json) {
        Ok(files) => files,
        Err(error) => {
            return parse_error(
                "XEZIM_WEB_FILES",
                format!("Invalid project file map: {error}"),
            );
        }
    };
    let mut project_bytes = 0usize;
    let mut virtual_files = Vec::with_capacity(files.len());
    let mut sources = Vec::new();
    for file in files {
        project_bytes = project_bytes
            .saturating_add(file.path.len())
            .saturating_add(file.contents.len());
        if project_bytes > MAX_PROJECT_BYTES {
            return parse_error(
                "XEZIM_WEB_PROJECT_LIMIT",
                format!(
                    "Project files exceed the {} byte browser limit.",
                    MAX_PROJECT_BYTES
                ),
            );
        }
        let lower = file.path.to_ascii_lowercase();
        if lower.ends_with(".sv") {
            sources.push((file.path.clone(), file.contents.clone()));
        }
        virtual_files.push((file.path, file.contents.into_bytes()));
    }
    if sources.is_empty() {
        return parse_error(
            "XEZIM_WEB_NO_SOURCES",
            "Project must include at least one .sv source file.".into(),
        );
    }
    if max_time_ns > MAX_TIME_NS {
        return parse_error(
            "XEZIM_WEB_TIME_LIMIT",
            format!("Maximum simulation time is {} ns.", MAX_TIME_NS),
        );
    }

    let output_limit = max_output_bytes.min(MAX_OUTPUT_BYTES);
    let vcd_limit = max_vcd_bytes.min(MAX_VCD_BYTES);
    let capture = VcdCapture::new(vcd_limit);
    let mut sim = match xezim::simulate_project_inline_with_output_limit(
        &sources,
        &virtual_files,
        max_time_ns,
        Some(top_module),
        MAX_PROJECT_BYTES,
        Some(output_limit),
        Box::new(BoundedVcdWriter(capture.clone())),
    ) {
        Ok(sim) => sim,
        Err(message) => return parse_error("XEZIM_COMPILE", message),
    };
    let mut diagnostics = sim
        .compile_errors
        .drain(..)
        .map(|message| {
            let code = if message.starts_with("XEZIM_WEB_UNSUPPORTED_TASK:") {
                "XEZIM_WEB_UNSUPPORTED_TASK"
            } else if message.starts_with("XEZIM_WEB_FILE_ERROR:") {
                "XEZIM_WEB_FILE_ERROR"
            } else {
                "XEZIM_COMPILE"
            };
            let severity = "error";
            Diagnostic {
                severity,
                code,
                message,
            }
        })
        .collect::<Vec<_>>();
    let joined_output = sim
        .output
        .iter()
        .map(|line| line.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let (stdout, stdout_exceeds_response_limit) = truncate_utf8(&joined_output, output_limit);
    let output_truncated = sim.output_capture_truncated() || stdout_exceeds_response_limit;
    if output_truncated {
        diagnostics.push(Diagnostic {
            severity: "warning",
            code: "XEZIM_WEB_OUTPUT_LIMIT",
            message: format!("Console output was truncated at {} bytes.", output_limit),
        });
    }
    let (vcd_bytes, vcd_exceeded) = capture
        .lock()
        .map(|c| (c.bytes.clone(), c.exceeded))
        .unwrap_or_default();
    let file_limit_exceeded = sim.virtual_file_limit_exceeded();
    let file_outputs = sim
        .virtual_file_outputs()
        .into_iter()
        .map(|(path, bytes)| FileOutput {
            path,
            contents: String::from_utf8_lossy(&bytes).into_owned(),
        })
        .collect::<Vec<_>>();
    let resource_limit = output_truncated || vcd_exceeded || file_limit_exceeded;
    if vcd_exceeded {
        diagnostics.push(Diagnostic {
            severity: "error",
            code: "XEZIM_WEB_VCD_LIMIT",
            message: format!(
                "Generated VCD exceeded the {} byte browser limit.",
                vcd_limit
            ),
        });
    }
    if file_limit_exceeded {
        diagnostics.push(Diagnostic {
            severity: "error",
            code: "XEZIM_WEB_FILE_LIMIT",
            message: format!(
                "Virtual project files exceeded the {} byte browser limit.",
                MAX_PROJECT_BYTES
            ),
        });
    }
    let has_error = diagnostics.iter().any(|d| d.severity == "error");
    let status = if resource_limit {
        RunStatus::ResourceLimit
    } else if sim.saw_fatal || has_error {
        RunStatus::Failed
    } else {
        RunStatus::Completed
    };
    let timescale = vcd_timescale(&vcd_bytes);
    let waveform = (!vcd_bytes.is_empty() || vcd_exceeded).then_some(WaveformMetadata {
        format: "vcd",
        size_bytes: vcd_bytes.len(),
        complete: !vcd_exceeded,
        timescale,
    });
    let vcd = if vcd_exceeded {
        None
    } else {
        String::from_utf8(vcd_bytes).ok()
    };
    encode(RunResult {
        schema_version: 1,
        status,
        max_time_ns: max_time_ns.to_string(),
        last_time_ticks: sim.time.to_string(),
        finished: sim.finished,
        saw_fatal: sim.saw_fatal,
        stdout,
        diagnostics,
        snapshot_id: Some(snapshot_id.to_string()),
        waveform,
        vcd,
        file_outputs: Some(file_outputs),
    })
}

/// One-shot browser simulation entry point. Call it from a dedicated Worker.
/// The outer Worker owns the hard wall-time limit and can be terminated even
/// if a SystemVerilog process does not yield back to JavaScript.
#[wasm_bindgen(js_name = compileAndRun)]
pub fn compile_and_run(
    source: &str,
    top_module: &str,
    max_time_ns: u64,
    max_output_bytes: usize,
) -> String {
    let output_limit = max_output_bytes.min(MAX_OUTPUT_BYTES);
    if source.len() > MAX_SOURCE_BYTES {
        return encode(RunResult::error(
            "XEZIM_WEB_SOURCE_LIMIT",
            format!(
                "Source exceeds the {} byte browser limit.",
                MAX_SOURCE_BYTES
            ),
            max_time_ns,
        ));
    }
    if top_module.is_empty() || top_module.len() > MAX_TOP_MODULE_BYTES {
        return encode(RunResult::error(
            "XEZIM_WEB_TOP_MODULE",
            "Top module must contain 1 to 512 UTF-8 bytes.",
            max_time_ns,
        ));
    }
    if max_time_ns > MAX_TIME_NS {
        return encode(RunResult::error(
            "XEZIM_WEB_TIME_LIMIT",
            format!("Maximum simulation time is {} ns.", MAX_TIME_NS),
            max_time_ns,
        ));
    }

    let files_json = serde_json::json!([{"path":"main.sv", "contents":source}]).to_string();
    run_project(
        &files_json,
        "sha256:anonymous-source",
        top_module,
        max_time_ns,
        output_limit,
        MAX_VCD_BYTES,
    )
}

/// One-shot browser simulation for a named source snapshot. All files, VCD
/// output, and `$fopen` data use project-relative in-memory storage.
#[wasm_bindgen(js_name = compileAndRunProject)]
pub fn compile_and_run_project(
    files_json: &str,
    snapshot_id: &str,
    top_module: &str,
    max_time_ns: u64,
    max_output_bytes: usize,
    max_vcd_bytes: usize,
) -> String {
    run_project(
        files_json,
        snapshot_id,
        top_module,
        max_time_ns,
        max_output_bytes,
        max_vcd_bytes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNTER: &str = include_str!("../fixtures/counter.sv");

    fn project_json() -> String {
        let files = [
            (
                "rtl/marker_pkg.sv",
                include_str!("../fixtures/project/rtl/marker_pkg.sv"),
            ),
            ("rtl/top.sv", include_str!("../fixtures/project/rtl/top.sv")),
            (
                "rtl/includes/top_defs.svh",
                include_str!("../fixtures/project/rtl/includes/top_defs.svh"),
            ),
            (
                "rtl/includes/nested.svh",
                include_str!("../fixtures/project/rtl/includes/nested.svh"),
            ),
            ("rom.hex", include_str!("../fixtures/project/rom.hex")),
        ];
        serde_json::to_string(
            &files
                .iter()
                .map(|(path, contents)| serde_json::json!({ "path": path, "contents": contents }))
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn count_transitions(vcd: &str) -> Vec<(String, String)> {
        let code = vcd
            .lines()
            .find_map(|line| {
                let fields: Vec<_> = line.split_whitespace().collect();
                (fields.len() >= 6
                    && fields[0] == "$var"
                    && fields[2] == "4"
                    && fields[4] == "count")
                    .then(|| fields[3].to_string())
            })
            .expect("count VCD declaration");
        let mut time = "0".to_string();
        let mut values = Vec::new();
        for line in vcd.lines() {
            if let Some(tick) = line.strip_prefix('#') {
                time = tick.to_string();
                continue;
            }
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() == 2 && fields[1] == code && fields[0].starts_with('b') {
                let value = fields[0][1..].to_ascii_lowercase();
                if !value.contains('x') && !value.contains('z') {
                    values.push((time.clone(), value));
                }
            }
        }
        values
    }

    #[test]
    fn runs_the_real_four_bit_counter_and_captures_its_output() {
        let value: serde_json::Value =
            serde_json::from_str(&compile_and_run(COUNTER, "counter", 50, MAX_OUTPUT_BYTES))
                .unwrap();
        assert_eq!(value["status"], "completed", "{value}");
        assert!(value["stdout"].as_str().unwrap().contains("COUNT=4"));
        assert!(value["finished"].as_bool().unwrap());
    }

    #[test]
    fn rejects_oversized_source_before_entering_xezim() {
        let source = " ".repeat(MAX_SOURCE_BYTES + 1);
        let value: serde_json::Value =
            serde_json::from_str(&compile_and_run(&source, "counter", 50, MAX_OUTPUT_BYTES))
                .unwrap();
        assert_eq!(value["status"], "failed");
        assert_eq!(value["diagnostics"][0]["code"], "XEZIM_WEB_SOURCE_LIMIT");
    }

    #[test]
    fn enforces_output_limit_on_utf8_boundaries() {
        let (output, truncated) = truncate_utf8("éabc", 1);
        assert!(truncated);
        assert_eq!(output, "");
    }

    #[test]
    fn bounds_simulator_output_during_capture_and_preserves_utf8() {
        let source = r#"module top; initial begin $display("%c", 8'hE9); $display("later"); $finish; end endmodule"#;
        let sources = vec![("top.sv".to_string(), source.to_string())];
        let virtual_files = vec![("top.sv".to_string(), source.as_bytes().to_vec())];
        let sim = xezim::simulate_project_inline_with_output_limit(
            &sources,
            &virtual_files,
            1,
            Some("top"),
            MAX_PROJECT_BYTES,
            Some(2),
            Box::new(Vec::new()),
        )
        .unwrap();
        let stdout = sim
            .output
            .iter()
            .map(|line| line.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(stdout, "é");
        assert!(stdout.len() <= 2);
        assert!(sim.output_capture_truncated());
    }

    #[test]
    fn project_simulation_keeps_native_output_capture_unbounded_by_default() {
        let source = r#"module top; initial begin $display("abcdefgh"); $display("ijklmnop"); $finish; end endmodule"#;
        let sources = vec![("top.sv".to_string(), source.to_string())];
        let virtual_files = vec![("top.sv".to_string(), source.as_bytes().to_vec())];
        let sim = xezim::simulate_project_inline(
            &sources,
            &virtual_files,
            1,
            Some("top"),
            MAX_PROJECT_BYTES,
            Box::new(Vec::new()),
        )
        .unwrap();
        let stdout = sim
            .output
            .iter()
            .map(|line| line.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(stdout, "abcdefgh\nijklmnop");
        assert!(!sim.output_capture_truncated());
    }

    #[test]
    fn bounded_output_coalesces_many_small_writes() {
        let source = r#"module top; integer i; initial begin for (i = 0; i < 100; i = i + 1) $write("x"); $finish; end endmodule"#;
        let sources = vec![("top.sv".to_string(), source.to_string())];
        let virtual_files = vec![("top.sv".to_string(), source.as_bytes().to_vec())];
        let sim = xezim::simulate_project_inline_with_output_limit(
            &sources,
            &virtual_files,
            1,
            Some("top"),
            MAX_PROJECT_BYTES,
            Some(32),
            Box::new(Vec::new()),
        )
        .unwrap();
        let stdout = sim
            .output
            .iter()
            .map(|line| line.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(sim.output.len(), 1);
        assert_eq!(stdout, "x\n".repeat(16));
        assert!(sim.output_capture_truncated());
    }

    #[test]
    fn reports_output_budget_exhaustion_in_the_browser_result() {
        let source = r#"module top; integer i; initial begin for (i = 0; i < 100; i = i + 1) $display("x"); $finish; end endmodule"#;
        let files = serde_json::to_string(&[serde_json::json!({
            "path": "top.sv",
            "contents": source,
        })])
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&compile_and_run_project(
            &files,
            "sha256:stdout-limit",
            "top",
            1,
            10,
            MAX_VCD_BYTES,
        ))
        .unwrap();
        assert_eq!(value["status"], "resource_limit", "{value}");
        assert_eq!(value["stdout"], "x\nx\nx\nx\nx\n");
        assert!(
            value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| {
                    diagnostic["severity"] == "warning"
                        && diagnostic["code"] == "XEZIM_WEB_OUTPUT_LIMIT"
                })
        );
    }

    #[test]
    fn runs_named_sources_nested_includes_rom_and_virtual_file_io_into_valid_vcd() {
        let json = project_json();
        let value: serde_json::Value = serde_json::from_str(&run_project(
            &json,
            "sha256:fixture-project",
            "top",
            20,
            MAX_OUTPUT_BYTES,
            MAX_VCD_BYTES,
        ))
        .unwrap();
        assert_eq!(value["status"], "completed", "{value}");
        assert_eq!(value["snapshotId"], "sha256:fixture-project");
        assert!(
            value["stdout"].as_str().unwrap().contains("NBA_PRE=1"),
            "{value}"
        );
        assert!(
            value["stdout"].as_str().unwrap().contains("NBA_POST=2"),
            "{value}"
        );
        assert!(
            value["stdout"].as_str().unwrap().contains("ROM=f0 COUNT=3"),
            "{value}"
        );
        assert!(
            value["stdout"].as_str().unwrap().contains("FILE=ROM=f0"),
            "{value}"
        );
        assert_eq!(value["waveform"]["format"], "vcd");
        assert_eq!(value["waveform"]["complete"], true);
        let vcd = value["vcd"].as_str().unwrap();
        assert!(vcd.starts_with("$date"));
        assert!(vcd.contains("$timescale"));
        assert!(vcd.contains("$enddefinitions"));
        assert!(vcd.contains("10xz"), "VCD must preserve X/Z states: {vcd}");
        assert_eq!(
            count_transitions(vcd),
            vec![
                ("0".into(), "1".into()),
                ("1000".into(), "10".into()),
                ("3000".into(), "11".into()),
            ]
        );
        assert_eq!(value["fileOutputs"][0]["path"], "captured.txt");
        assert_eq!(value["fileOutputs"][0]["contents"], "ROM=f0\n");
    }

    #[test]
    fn vcd_overflow_is_reported_as_resource_limit_without_returning_partial_vcd() {
        let value: serde_json::Value = serde_json::from_str(&run_project(
            &project_json(),
            "sha256:overflow",
            "top",
            20,
            MAX_OUTPUT_BYTES,
            64,
        ))
        .unwrap();
        assert_eq!(value["status"], "resource_limit", "{value}");
        assert_eq!(value["waveform"]["complete"], false);
        assert_eq!(value["vcd"], serde_json::Value::Null);
        assert_eq!(value["diagnostics"][0]["code"], "XEZIM_WEB_VCD_LIMIT");
    }

    #[test]
    fn rejects_project_path_traversal_and_reports_disabled_host_system_task() {
        let bad_path = r#"[{"path":"../escape.sv","contents":"module top; endmodule"}]"#;
        let path_error: serde_json::Value = serde_json::from_str(&run_project(
            bad_path,
            "sha256:path",
            "top",
            1,
            MAX_OUTPUT_BYTES,
            MAX_VCD_BYTES,
        ))
        .unwrap();
        assert_eq!(path_error["status"], "failed");
        assert_eq!(path_error["diagnostics"][0]["code"], "XEZIM_COMPILE");

        let system_files = r#"[{"path":"top.sv","contents":"module top; initial begin $system(\"echo unsafe\"); $finish; end endmodule"}]"#;
        let system_error: serde_json::Value = serde_json::from_str(&run_project(
            system_files,
            "sha256:system",
            "top",
            1,
            MAX_OUTPUT_BYTES,
            MAX_VCD_BYTES,
        ))
        .unwrap();
        assert_eq!(system_error["status"], "failed", "{system_error}");
        assert_eq!(
            system_error["diagnostics"][0]["code"],
            "XEZIM_WEB_UNSUPPORTED_TASK"
        );
    }

    #[test]
    fn missing_virtual_rom_file_is_an_error_diagnostic() {
        let files = r#"[{"path":"top.sv","contents":"module top; logic [7:0] mem[0:0]; initial begin $readmemh(\"missing.hex\", mem); $finish; end endmodule"}]"#;
        let value: serde_json::Value = serde_json::from_str(&run_project(
            files,
            "sha256:missing-file",
            "top",
            1,
            MAX_OUTPUT_BYTES,
            MAX_VCD_BYTES,
        ))
        .unwrap();
        assert_eq!(value["status"], "failed", "{value}");
        assert_eq!(value["diagnostics"][0]["code"], "XEZIM_WEB_FILE_ERROR");
    }
}
