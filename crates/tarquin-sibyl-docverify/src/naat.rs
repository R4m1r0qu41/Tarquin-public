//! Optional NAAT text/OCR adapter. NAAT is a configurable external tool — its path is never
//! hardcoded. If no command is configured, the adapter is silently skipped and the affected
//! dimension stays `not_determined`. NAAT output is treated as OCR/text extraction evidence only;
//! it does not ground any dimension by itself.
//!
//! Interchange contract version: "1.0". NAAT must write newline-delimited JSON to stdout; this
//! crate reads only the `text` field. Wall-clock timestamps, absolute paths, and
//! machine-specific fields from NAAT output are excluded from the canonical finding.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::runner::{path_arg, CliResult, CliRunner};

/// Configuration for the NAAT adapter.
#[derive(Debug, Clone)]
pub struct NaatConfig {
    /// Absolute path to the NAAT `document_ingest` command. `None` disables the adapter.
    pub command: Option<PathBuf>,
    /// Expected interchange schema version. Mismatch is recorded as a warning.
    pub interchange_version: String,
}

impl NaatConfig {
    pub fn disabled() -> Self {
        NaatConfig {
            command: None,
            interchange_version: "1.0".to_string(),
        }
    }
}

/// Result of a NAAT invocation (or absence).
#[derive(Debug, Clone)]
pub struct NaatOutput {
    /// Extracted text content (may be empty).
    pub text: String,
    /// Adapter identity recorded in the evidence manifest.
    pub adapter_identity: String,
    /// `sha256:<hex>` of the raw input bytes passed to NAAT.
    pub input_hash: String,
    /// `sha256:<hex>` of the raw stdout bytes NAAT produced.
    pub output_hash: String,
}

/// Run the configured NAAT command on `artifact_path`. Returns `None` if the adapter is not
/// configured or the invocation fails. In both cases the caller records the outcome in evidence.
pub fn run_naat(
    config: &NaatConfig,
    runner: &dyn CliRunner,
    artifact_path: &Path,
    artifact_sha256: &str,
) -> Option<NaatOutput> {
    let command_path = config.command.as_ref()?;
    let cmd_str = path_arg(command_path);
    let artifact_str = path_arg(artifact_path);

    let result = runner.run(&cmd_str, &[&artifact_str]);

    match result {
        CliResult::ToolMissing => None,
        CliResult::Failed { .. } => None,
        CliResult::Ok { stdout } => {
            let output_hash = sha256_of_str(&stdout);
            let text = extract_text_from_naat_output(&stdout);
            Some(NaatOutput {
                text,
                adapter_identity: format!("naat:{}", config.interchange_version),
                input_hash: artifact_sha256.to_string(),
                output_hash,
            })
        }
    }
}

/// Parse the `text` field from NAAT's newline-delimited JSON output. Ignores all other fields
/// (wall-clock timestamps, paths, machine-specific data) to keep the canonical finding clean.
fn extract_text_from_naat_output(stdout: &str) -> String {
    for line in stdout.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                return text.to_string();
            }
        }
    }
    String::new()
}

fn sha256_of_str(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let d = h.finalize();
    format!(
        "sha256:{}",
        d.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::FakeRunner;
    use std::path::PathBuf;

    fn fake_artifact() -> PathBuf {
        PathBuf::from("/synthetic/test.pdf")
    }

    #[test]
    fn disabled_config_returns_none() {
        let config = NaatConfig::disabled();
        let runner = FakeRunner::new();
        let result = run_naat(&config, &runner, &fake_artifact(), "sha256:aabb");
        assert!(result.is_none());
    }

    #[test]
    fn missing_tool_returns_none() {
        let config = NaatConfig {
            command: Some(PathBuf::from("/usr/local/bin/document_ingest")),
            interchange_version: "1.0".to_string(),
        };
        let runner = FakeRunner::new().with_missing("/usr/local/bin/document_ingest");
        let result = run_naat(&config, &runner, &fake_artifact(), "sha256:aabb");
        assert!(result.is_none());
    }

    #[test]
    fn valid_naat_output_extracts_text() {
        let naat_json =
            r#"{"text": "CURP: BAAB260101HMCRNS09", "timestamp": "2026-06-27T00:00:00Z"}"#;
        let config = NaatConfig {
            command: Some(PathBuf::from("/usr/local/bin/document_ingest")),
            interchange_version: "1.0".to_string(),
        };
        let runner = FakeRunner::new().with_ok("/usr/local/bin/document_ingest", naat_json);
        let result = run_naat(&config, &runner, &fake_artifact(), "sha256:aabb").unwrap();
        assert_eq!(result.text, "CURP: BAAB260101HMCRNS09");
        // Machine-specific fields (timestamp) must not appear in the output struct.
        assert!(!result.output_hash.is_empty());
        assert!(result.output_hash.starts_with("sha256:"));
    }

    #[test]
    fn wall_clock_fields_are_not_captured() {
        // NAAT may include timestamps, paths, etc. We must NOT expose them in NaatOutput.
        let naat_json =
            r#"{"text": "hello", "timestamp": "2026-01-01", "path": "/home/user/doc.pdf"}"#;
        let config = NaatConfig {
            command: Some(PathBuf::from("/naat")),
            interchange_version: "1.0".to_string(),
        };
        let runner = FakeRunner::new().with_ok("/naat", naat_json);
        let result = run_naat(&config, &runner, &fake_artifact(), "sha256:0011").unwrap();
        // Only `text` is captured; timestamp and path are absent from `NaatOutput`.
        assert_eq!(result.text, "hello");
        // Struct fields: text, adapter_identity, input_hash, output_hash — no timestamp/path.
    }
}
