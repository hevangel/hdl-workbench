use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;
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
        }
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

    let sources = [source.to_string()];
    let mut sim = match xezim::simulate_multi_inline(&sources, max_time_ns, Some(top_module)) {
        Ok(sim) => sim,
        Err(message) => {
            return encode(RunResult::error("XEZIM_COMPILE", message, max_time_ns));
        }
    };

    let mut diagnostics = sim
        .compile_errors
        .drain(..)
        .map(|message| Diagnostic {
            severity: "error",
            code: "XEZIM_COMPILE",
            message,
        })
        .collect::<Vec<_>>();
    let joined_output = sim
        .output
        .iter()
        .map(|line| line.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let (stdout, truncated) = truncate_utf8(&joined_output, output_limit);
    if truncated {
        diagnostics.push(Diagnostic {
            severity: "warning",
            code: "XEZIM_WEB_OUTPUT_LIMIT",
            message: format!("Console output was truncated at {} bytes.", output_limit),
        });
    }
    let status = if sim.saw_fatal || !diagnostics.iter().all(|d| d.severity != "error") {
        RunStatus::Failed
    } else if truncated {
        RunStatus::ResourceLimit
    } else {
        RunStatus::Completed
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNTER: &str = include_str!("../fixtures/counter.sv");

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
}
