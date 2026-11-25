//! `tarquin-sibyl-docverify` — document-verification extraction engine.
//!
//! This crate owns all document I/O, external CLI invocation, and evidence manifest assembly.
//! The deterministic reduction (`docverify::reduce`) lives in `tarquin-sibyl` and is called here
//! after the extraction phase. The two layers are deliberately separated: this crate depends on
//! the environment; `tarquin-sibyl` does not.
//!
//! External commands (`zbarimg`, `pdfsig`) are invoked via the [`CliRunner`] trait so tests can
//! inject fakes without requiring those binaries to be installed.

pub mod claim;
pub mod error;
pub mod evidence;
pub mod ingest;
pub mod naat;
pub mod pdf;
pub mod profiles;
pub mod qr;
pub mod report;
pub mod runner;

use std::path::Path;

pub use error::DocVerifyError;
pub use naat::NaatConfig;
pub use runner::{CliRunner, SystemRunner};

use tarquin_sibyl::docverify::{reduce, DocumentVerificationFinding};

/// Top-level result of one verification run.
pub struct VerifyOutput {
    pub finding: DocumentVerificationFinding,
    pub manifest: evidence::EvidenceManifest,
    pub canonical_json: String,
    pub markdown: String,
}

/// Run the full document-verification pipeline on `artifact_path`.
///
/// `checked_at` must be an explicit ISO 8601 string supplied by the caller — the engine never
/// reads the system clock. `runner` is injectable for testing. If `naat_config.command` is
/// `None` the NAAT OCR pass is skipped silently.
pub fn verify(
    artifact_path: &Path,
    checked_at: &str,
    runner: &dyn CliRunner,
    naat_config: &NaatConfig,
) -> Result<VerifyOutput, DocVerifyError> {
    // 1. Ingest artifact: hash + media type.
    let artifact = ingest::ingest(artifact_path)?;

    // 2. QR decode (informational; never grounds authenticity per DV-004).
    let qr_finding = qr::decode_qr(runner, artifact_path);

    // 3. PDF signature detection (informational; detection ≠ validation).
    let pdf_finding = pdf::detect_signatures(runner, artifact_path);

    // 4. Optional NAAT OCR (informational).
    let naat_output = naat::run_naat(naat_config, runner, artifact_path, &artifact.sha256);

    // 5. Profile match.
    let profile = profiles::match_profile(artifact.media_type, &qr_finding.texts)
        .ok_or(DocVerifyError::NoProfileMatch)?;

    // 6. Build evidence manifest.
    let manifest = evidence::build_manifest(
        &artifact,
        profile.id,
        &qr_finding,
        &pdf_finding,
        naat_output.as_ref(),
    );
    let manifest_hash = manifest.canonical_hash();

    // 7. Assemble ClaimedFinding.
    let claimed = claim::build_claim(&artifact, profile, &manifest, &manifest_hash, checked_at);

    // 8. Reduce to constrained finding.
    let finding = reduce(&claimed);

    // 9. Generate reports.
    let canonical_json =
        report::to_canonical_json(&finding).map_err(DocVerifyError::ReportSerialize)?;
    let markdown = report::to_markdown(&finding);

    Ok(VerifyOutput {
        finding,
        manifest,
        canonical_json,
        markdown,
    })
}
