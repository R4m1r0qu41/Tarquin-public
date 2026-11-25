//! PDF digital-signature **detection** via `pdfsig` (Poppler). Detection is distinct from
//! validation: knowing a signature object exists does not confirm it is valid. At MVP only
//! detection counts are recorded; validation (CMS/PKCS#7 chain, OCSP/CRL) is a future phase.
//! Detection evidence alone never raises `signer_identity`, `document_integrity`, or
//! `document_authenticity`.

use std::path::Path;

use crate::runner::{path_arg, CliResult, CliRunner};

/// Result of the PDF signature-detection pass.
#[derive(Debug, Clone)]
pub struct PdfSigFinding {
    /// Number of signature objects found in the PDF structure (detection, not validation).
    pub signature_count: u32,
    /// Tool identity recorded in the evidence manifest.
    pub tool_identity: String,
}

/// Run `pdfsig <path>` and count how many signatures are reported. If the tool is absent or the
/// file is not a PDF, returns a zero-count finding with the appropriate tool identity.
pub fn detect_signatures(runner: &dyn CliRunner, path: &Path) -> PdfSigFinding {
    let path_str = path_arg(path);
    let result = runner.run("pdfsig", &[&path_str]);

    match result {
        CliResult::ToolMissing => PdfSigFinding {
            signature_count: 0,
            tool_identity: "pdfsig:unavailable".to_string(),
        },
        CliResult::Failed { stdout, .. } => {
            let count = count_signatures_in_output(&stdout);
            PdfSigFinding {
                signature_count: count,
                tool_identity: "pdfsig:error".to_string(),
            }
        }
        CliResult::Ok { stdout } => {
            let count = count_signatures_in_output(&stdout);
            let version = probe_version(runner);
            PdfSigFinding {
                signature_count: count,
                tool_identity: format!("pdfsig:{version}"),
            }
        }
    }
}

/// Parse `pdfsig` stdout and count "Signature #N" lines. Poppler's pdfsig prints one such line
/// per signature object in the PDF.
fn count_signatures_in_output(stdout: &str) -> u32 {
    stdout
        .lines()
        .filter(|l| l.trim_start().starts_with("Signature #"))
        .count() as u32
}

fn probe_version(runner: &dyn CliRunner) -> String {
    match runner.run("pdfsig", &["--version"]) {
        CliResult::Ok { stdout } | CliResult::Failed { stdout, .. } => stdout
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string(),
        CliResult::ToolMissing => "unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::FakeRunner;
    use std::path::PathBuf;

    fn fake_path() -> PathBuf {
        PathBuf::from("/synthetic/test.pdf")
    }

    #[test]
    fn missing_tool_returns_zero_count() {
        let runner = FakeRunner::new().with_missing("pdfsig");
        let f = detect_signatures(&runner, &fake_path());
        assert_eq!(f.signature_count, 0);
        assert_eq!(f.tool_identity, "pdfsig:unavailable");
    }

    #[test]
    fn two_signatures_counted() {
        // FakeRunner maps by program name; use one response for all "pdfsig" calls.
        let stdout = "Digital Signature Info of: /synthetic/test.pdf\nSignature #1:\n  - Signer Certificate Common Name: Test CA\nSignature #2:\n  - Signer Certificate Common Name: Test Signer\n";
        let runner = FakeRunner::new().with_ok("pdfsig", stdout);
        let f = detect_signatures(&runner, &fake_path());
        assert_eq!(f.signature_count, 2);
    }

    #[test]
    fn no_signature_lines_gives_zero() {
        let runner = FakeRunner::new()
            .with_ok("pdfsig", "Digital Signature Info of: test.pdf\n")
            .with_ok("pdfsig", "pdfsig version 22.12.0");
        let f = detect_signatures(&runner, &fake_path());
        assert_eq!(f.signature_count, 0);
    }
}
