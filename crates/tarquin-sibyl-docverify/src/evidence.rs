//! Evidence manifest: a deterministic, content-hashed record of every observation made during
//! a verification run. The manifest hash (`sha256:<hex>`) goes into `ClaimedFinding.evidence_manifest_hash`
//! and pins all evidence IDs cited in `EvidenceRef.evidence_id`.

use sha2::{Digest, Sha256};

use crate::{ingest::IngestedArtifact, naat::NaatOutput, pdf::PdfSigFinding, qr::QrFinding};

/// A single observation recorded in the evidence manifest.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvidenceEntry {
    /// Stable, unique within this manifest. Used as `EvidenceRef.evidence_id`.
    pub id: String,
    /// `EvidenceKind` label (snake_case, matching the reducer vocabulary).
    pub kind: String,
    pub description: String,
    pub tool_identity: String,
}

/// The full set of observations from one verification run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvidenceManifest {
    pub artifact_sha256: String,
    pub profile_id: String,
    pub entries: Vec<EvidenceEntry>,
}

impl EvidenceManifest {
    /// Canonical SHA-256 of the manifest serialized as compact JSON with deterministic key order.
    /// This value becomes `ClaimedFinding.evidence_manifest_hash`.
    pub fn canonical_hash(&self) -> String {
        let json = serde_json::to_string(self).expect("EvidenceManifest is always serializable");
        let mut h = Sha256::new();
        h.update(json.as_bytes());
        let d = h.finalize();
        format!(
            "sha256:{}",
            d.iter().map(|b| format!("{b:02x}")).collect::<String>()
        )
    }
}

/// Build the evidence manifest from all observations collected in a single run.
pub fn build_manifest(
    artifact: &IngestedArtifact,
    profile_id: &str,
    qr: &QrFinding,
    pdf_sig: &PdfSigFinding,
    naat: Option<&NaatOutput>,
) -> EvidenceManifest {
    let mut entries = Vec::new();

    // Artifact hash is always recorded.
    entries.push(EvidenceEntry {
        id: "ev-artifact-hash".to_string(),
        kind: "artifact_hash".to_string(),
        description: format!(
            "SHA-256 of artifact bytes ({} bytes, {})",
            artifact.byte_len,
            artifact.media_type.as_str()
        ),
        tool_identity: "sha2::Sha256".to_string(),
    });

    // QR finding (informational — never grounds authenticity per DV-004).
    if !qr.texts.is_empty() {
        for (i, text) in qr.texts.iter().enumerate() {
            let class = crate::qr::classify_qr(text);
            entries.push(EvidenceEntry {
                id: format!("ev-qr-{:03}", i + 1),
                kind: "qr_decode".to_string(),
                description: format!(
                    "QR decoded text #{}: class={:?} len={}",
                    i + 1,
                    class,
                    text.len()
                ),
                tool_identity: qr.tool_identity.as_str().to_string(),
            });
        }
    } else {
        entries.push(EvidenceEntry {
            id: "ev-qr-absent".to_string(),
            kind: "qr_decode".to_string(),
            description: format!(
                "QR decode attempted; no barcodes found (tool: {})",
                qr.tool_identity.as_str()
            ),
            tool_identity: qr.tool_identity.as_str().to_string(),
        });
    }

    // PDF signature detection (informational — detection ≠ validation).
    entries.push(EvidenceEntry {
        id: "ev-pdf-sig-detect".to_string(),
        kind: "pdf_signature_detection".to_string(),
        description: format!(
            "PDF signature detection: {} signature object(s) found",
            pdf_sig.signature_count
        ),
        tool_identity: pdf_sig.tool_identity.clone(),
    });

    // NAAT output (optional OCR/text extraction — informational).
    if let Some(naat_out) = naat {
        entries.push(EvidenceEntry {
            id: "ev-naat-ocr".to_string(),
            kind: "ocr_extraction".to_string(),
            description: format!(
                "NAAT text/OCR extraction: {} chars extracted",
                naat_out.text.len()
            ),
            tool_identity: naat_out.adapter_identity.clone(),
        });
    }

    EvidenceManifest {
        artifact_sha256: artifact.sha256.clone(),
        profile_id: profile_id.to_string(),
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ingest::MediaType, qr::ToolIdentity};

    fn fake_artifact() -> IngestedArtifact {
        IngestedArtifact {
            sha256: "sha256:aabb".to_string(),
            media_type: MediaType::Pdf,
            byte_len: 100,
        }
    }

    fn empty_qr() -> QrFinding {
        QrFinding {
            texts: vec![],
            tool_identity: ToolIdentity::Missing,
        }
    }

    fn zero_pdf_sig() -> PdfSigFinding {
        PdfSigFinding {
            signature_count: 0,
            tool_identity: "pdfsig:unavailable".to_string(),
        }
    }

    #[test]
    fn manifest_always_has_artifact_hash_entry() {
        let m = build_manifest(
            &fake_artifact(),
            "MX-JUZGADO-OFICIO-01",
            &empty_qr(),
            &zero_pdf_sig(),
            None,
        );
        assert!(m.entries.iter().any(|e| e.id == "ev-artifact-hash"));
    }

    #[test]
    fn canonical_hash_is_deterministic() {
        let m1 = build_manifest(
            &fake_artifact(),
            "MX-JUZGADO-OFICIO-01",
            &empty_qr(),
            &zero_pdf_sig(),
            None,
        );
        let m2 = build_manifest(
            &fake_artifact(),
            "MX-JUZGADO-OFICIO-01",
            &empty_qr(),
            &zero_pdf_sig(),
            None,
        );
        assert_eq!(m1.canonical_hash(), m2.canonical_hash());
    }

    #[test]
    fn different_artifacts_give_different_hashes() {
        let a1 = IngestedArtifact {
            sha256: "sha256:aa".to_string(),
            media_type: MediaType::Pdf,
            byte_len: 10,
        };
        let a2 = IngestedArtifact {
            sha256: "sha256:bb".to_string(),
            media_type: MediaType::Pdf,
            byte_len: 20,
        };
        let m1 = build_manifest(
            &a1,
            "MX-JUZGADO-OFICIO-01",
            &empty_qr(),
            &zero_pdf_sig(),
            None,
        );
        let m2 = build_manifest(
            &a2,
            "MX-JUZGADO-OFICIO-01",
            &empty_qr(),
            &zero_pdf_sig(),
            None,
        );
        assert_ne!(m1.canonical_hash(), m2.canonical_hash());
    }

    #[test]
    fn qr_texts_produce_per_entry_ids() {
        let qr = QrFinding {
            texts: vec!["BAAB260101HMCRNS09".to_string()],
            tool_identity: ToolIdentity::Available {
                version: "0.23".to_string(),
            },
        };
        let m = build_manifest(
            &fake_artifact(),
            "MX-RENAPO-CURP-01",
            &qr,
            &zero_pdf_sig(),
            None,
        );
        assert!(m.entries.iter().any(|e| e.id == "ev-qr-001"));
    }

    #[test]
    fn naat_absent_leaves_no_ocr_entry() {
        let m = build_manifest(
            &fake_artifact(),
            "MX-JUZGADO-OFICIO-01",
            &empty_qr(),
            &zero_pdf_sig(),
            None,
        );
        assert!(!m.entries.iter().any(|e| e.kind == "ocr_extraction"));
    }
}
