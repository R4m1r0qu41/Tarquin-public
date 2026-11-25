//! QR / barcode decode via `zbarimg`. Detection only — a decoded QR string is informational
//! evidence and never by itself raises `document_authenticity` (DV-004).

use std::path::Path;

use crate::runner::{path_arg, CliResult, CliRunner};

/// Outcome of attempting QR/barcode decoding on an artifact.
#[derive(Debug, Clone)]
pub struct QrFinding {
    /// Decoded text strings (one per barcode found). Empty when none found or tool unavailable.
    pub texts: Vec<String>,
    /// Tool identity string recorded in the evidence manifest.
    pub tool_identity: ToolIdentity,
}

#[derive(Debug, Clone)]
pub enum ToolIdentity {
    /// Tool ran successfully; version string from `--version` probe (best-effort).
    Available { version: String },
    /// `zbarimg` binary not found; finding is empty.
    Missing,
}

impl ToolIdentity {
    pub fn as_str(&self) -> &str {
        match self {
            ToolIdentity::Available { version } => version,
            ToolIdentity::Missing => "unavailable",
        }
    }
}

/// Run `zbarimg --quiet --raw <path>` and collect all decoded strings. If the tool is absent
/// or reports an error, returns an empty finding — dimensions stay `not_determined`.
pub fn decode_qr(runner: &dyn CliRunner, path: &Path) -> QrFinding {
    let path_str = path_arg(path);
    let result = runner.run("zbarimg", &["--quiet", "--raw", &path_str]);

    match result {
        CliResult::ToolMissing => QrFinding {
            texts: vec![],
            tool_identity: ToolIdentity::Missing,
        },
        CliResult::Failed { .. } => {
            // Non-zero exit: zbarimg exits 4 when no barcode found. Treat as empty finding.
            let version = probe_version(runner);
            QrFinding {
                texts: vec![],
                tool_identity: ToolIdentity::Available { version },
            }
        }
        CliResult::Ok { stdout } => {
            let texts = stdout
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect();
            let version = probe_version(runner);
            QrFinding {
                texts,
                tool_identity: ToolIdentity::Available { version },
            }
        }
    }
}

fn probe_version(runner: &dyn CliRunner) -> String {
    match runner.run("zbarimg", &["--version"]) {
        CliResult::Ok { stdout } | CliResult::Failed { stdout, .. } => stdout
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string(),
        CliResult::ToolMissing => "unavailable".to_string(),
    }
}

/// Classify a decoded QR text string into a known MX document category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QrClass {
    /// CURP: 18-char alphanumeric (RENAPO identity).
    MexicoCurp,
    /// CFDI verification URL pattern (SAT).
    MexicoCfdiUrl,
    /// RFC: 12 or 13-char alphanumeric (SAT fiscal ID).
    MexicoRfc,
    /// None of the known patterns.
    Unclassified,
}

pub fn classify_qr(text: &str) -> QrClass {
    let t = text.trim();
    // CURP: exactly 18 alphanumeric uppercase (simplified check)
    if t.len() == 18 && t.chars().all(|c| c.is_ascii_alphanumeric()) && t == t.to_uppercase() {
        return QrClass::MexicoCurp;
    }
    // RFC: 12 (moral) or 13 (natural) uppercase alphanumeric
    if (t.len() == 12 || t.len() == 13)
        && t.chars().all(|c| c.is_ascii_alphanumeric())
        && t == t.to_uppercase()
    {
        return QrClass::MexicoRfc;
    }
    // CFDI verification URL
    if t.starts_with("https://verificacfdi.facturaelectronica.sat.gob.mx")
        || t.starts_with("https://cfdiv33.sat.gob.mx")
    {
        return QrClass::MexicoCfdiUrl;
    }
    QrClass::Unclassified
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
    fn missing_tool_returns_empty_finding() {
        let runner = FakeRunner::new().with_missing("zbarimg");
        let f = decode_qr(&runner, &fake_path());
        assert!(f.texts.is_empty());
        assert!(matches!(f.tool_identity, ToolIdentity::Missing));
    }

    #[test]
    fn ok_output_parsed_into_texts() {
        // FakeRunner maps by program name; use one response for all "zbarimg" calls.
        let runner = FakeRunner::new().with_ok("zbarimg", "BAAB260101HMCRNS09\n");
        let f = decode_qr(&runner, &fake_path());
        assert_eq!(f.texts, vec!["BAAB260101HMCRNS09"]);
        assert!(matches!(f.tool_identity, ToolIdentity::Available { .. }));
    }

    #[test]
    fn classify_curp() {
        assert_eq!(classify_qr("BAAB260101HMCRNS09"), QrClass::MexicoCurp);
    }

    #[test]
    fn classify_rfc_moral() {
        // 12-char RFC (moral person)
        assert_eq!(classify_qr("ABC010101AA0"), QrClass::MexicoRfc);
    }

    #[test]
    fn classify_cfdi_url() {
        let url = "https://verificacfdi.facturaelectronica.sat.gob.mx/main.aspx?id=xxx";
        assert_eq!(classify_qr(url), QrClass::MexicoCfdiUrl);
    }

    #[test]
    fn classify_unrecognised() {
        assert_eq!(classify_qr("random text"), QrClass::Unclassified);
    }
}
