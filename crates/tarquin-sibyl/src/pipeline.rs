//! Phase 3.2 — the Tier-1 Sibyl pipeline: EXTRACT → CLASSIFY → ANCHOR → SIGN → EMIT.
//!
//! One source type is supported in v0.1: a structured court-docket JSON document
//! (`tarquin-docket-json/0.1`). The EXTRACT step is a fixed serde parse — the determinism
//! boundary of `sibyl-legal-oracle.md` §4: no inference, no runtime clock; every function
//! here is a pure function of (source bytes, key material, supplied time).
//!
//! WATCH (polling/subscribing to a live source) is deliberately out of scope: the caller hands
//! in the source bytes it observed.

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use arrow_core::types::event::LegJurisdiction;
use arrow_core::types::message::Action;
use arrow_identity::crypto::ed25519::sign_ed25519;

use crate::error::SibylError;
use crate::lbry::{LbryRecord, LbrySignature, StructuredFact};
use crate::registry::LegSubtype;

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

/// The docket source format this pipeline knows how to EXTRACT.
pub const DOCKET_FORMAT: &str = "tarquin-docket-json/0.1";

/// LBRY schema version emitted by this pipeline.
pub const LBRY_SCHEMA_VERSION: &str = "0.1";

/// A court-docket source document (`tarquin-docket-json/0.1`).
///
/// This models *what the source publishes*, before classification. Unknown fields are
/// rejected: an EXTRACT that silently ignores content it does not model is not auditable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocketSource {
    /// MUST be [`DOCKET_FORMAT`].
    pub docket_format: String,
    /// Resolvable reference to the docket entry (becomes `source_ref`).
    pub docket_ref: String,
    pub case_ref: String,
    pub court: String,
    pub jurisdiction_country: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction_supra: Option<String>,
    #[serde(default)]
    pub legal_system_refs: Vec<String>,
    pub order: DocketOrder,
}

/// The operative order inside a docket entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocketOrder {
    pub instrument_ref: String,
    /// LEG subtype as published (e.g. `JUD`).
    pub kind: LegSubtype,
    pub effective_at: String,
    /// Arrow Finality: `0` while the appeal window is open, `1` once final.
    pub finality: u8,
    /// Short legal reason code (e.g. `CO`, `SA`, `LR`).
    pub reason_code: String,
    pub operative_claim: String,
    /// Arrow Action the order claims (e.g. `HL`).
    pub policy_claim: String,
    /// SLC artifact ids the source names (plain ids only). A docket cannot carry the sealed
    /// `SlcArtifactRef` binding — `artifact_hash`/`computed_result_hash` are Lex.Codex's seal,
    /// produced against a verified pinned bundle, and must be present in the structured fact
    /// *before* the Sibyl signs. Until the docket path wires that pre-signing step, a docket
    /// naming SLC artifacts fails extraction (fail-closed) rather than dropping the reference.
    #[serde(default)]
    pub related_slc_artifacts: Vec<String>,
}

/// EXTRACT + CLASSIFY: parse docket source bytes into the [`StructuredFact`] of an LBRY record.
///
/// Deterministic by construction — a fixed parse of a closed schema (SO-005 discipline).
pub fn extract(source_bytes: &[u8]) -> Result<(DocketSource, StructuredFact), SibylError> {
    let doc: DocketSource =
        serde_json::from_slice(source_bytes).map_err(|e| SibylError::Parse(e.to_string()))?;
    if doc.docket_format != DOCKET_FORMAT {
        return Err(SibylError::Parse(format!(
            "unsupported docket_format {:?} (expected {DOCKET_FORMAT:?})",
            doc.docket_format
        )));
    }
    if !doc.order.related_slc_artifacts.is_empty() {
        return Err(SibylError::Parse(
            "docket names related_slc_artifacts, but sealed SlcArtifactRef binding is not yet \
             wired on the docket extraction path; refusing rather than dropping the reference"
                .to_string(),
        ));
    }
    let fact = StructuredFact {
        instrument_ref: doc.order.instrument_ref.clone(),
        leg_subtype: doc.order.kind,
        leg_jurisdiction: LegJurisdiction {
            country: Some(doc.jurisdiction_country.clone()),
            supra: doc.jurisdiction_supra.clone(),
        },
        legal_system_refs: doc.legal_system_refs.clone(),
        effective_at: doc.order.effective_at.clone(),
        finality: doc.order.finality,
        reason_code: doc.order.reason_code.clone(),
        operative_claim: doc.order.operative_claim.clone(),
        policy_claim: Action(doc.order.policy_claim.clone()),
        related_slc_artifacts: vec![],
    };
    Ok((doc, fact))
}

/// ANCHOR: `sha256:<hex>` over the raw observed source bytes.
pub fn anchor_source(source_bytes: &[u8]) -> String {
    let digest = Sha256::digest(source_bytes);
    let mut s = String::with_capacity(7 + digest.len() * 2);
    s.push_str("sha256:");
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Identity and time inputs for [`attest`] — everything the pipeline must be *given* rather
/// than discover (no runtime clock, no ambient identity).
#[derive(Debug, Clone)]
pub struct AttestParams {
    /// The attesting Sibyl's DID (`attested_by`); must be registered and active.
    pub sibyl_did: String,
    /// DID URL of the registered attestation key (`signature.key_ref`).
    pub key_ref: String,
    /// ISO 8601 UTC instant of attestation, supplied by the caller.
    pub attested_at: String,
    /// Record id; if `None`, derived deterministically from the source anchor.
    pub lbry_id: Option<String>,
}

/// ANCHOR + SIGN + EMIT: run the full Tier-1 pipeline over observed source bytes and return a
/// complete, signed [`LbryRecord`].
///
/// The signature is Ed25519 over the record's canonical content bytes (which already cover
/// `source_hash`, so the anchor is signed too).
pub fn attest(
    source_bytes: &[u8],
    params: &AttestParams,
    private_key: &[u8; 32],
) -> Result<LbryRecord, SibylError> {
    let (doc, fact) = extract(source_bytes)?;
    let source_hash = anchor_source(source_bytes);
    let lbry_id = params.lbry_id.clone().unwrap_or_else(|| {
        // `sha256:` prefix is 7 chars; take 16 hex chars of the anchor.
        format!("lbry-{}", &source_hash[7..23])
    });

    let mut record = LbryRecord {
        lbry_id,
        schema_version: LBRY_SCHEMA_VERSION.to_string(),
        attested_by: params.sibyl_did.clone(),
        attested_at: params.attested_at.clone(),
        source_ref: doc.docket_ref.clone(),
        source_hash,
        structured_fact: fact,
        content_hash: String::new(),
        signature: LbrySignature {
            key_ref: params.key_ref.clone(),
            algorithm: "Ed25519".to_string(),
            sig_b64: String::new(),
        },
    };
    record.content_hash = record.compute_content_hash()?;
    let payload = record.canonical_content_bytes()?;
    record.signature.sig_b64 = B64.encode(sign_ed25519(private_key, &payload));
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_identity::crypto::ed25519::generate_test_keypair;

    const DOCKET: &str = include_str!("../../../tests/fixtures/sibyl/docket-mx-2026-001.json");

    fn params() -> AttestParams {
        AttestParams {
            sibyl_did: "did:example:sibyl-mx-court-001".to_string(),
            key_ref: "did:example:sibyl-mx-court-001#attestation-1".to_string(),
            attested_at: "2026-06-08T00:00:00Z".to_string(),
            lbry_id: None,
        }
    }

    #[test]
    fn extract_is_deterministic_and_maps_fields() {
        let (doc, fact) = extract(DOCKET.as_bytes()).unwrap();
        assert_eq!(doc.docket_format, DOCKET_FORMAT);
        assert_eq!(fact.leg_subtype, LegSubtype::Jud);
        assert_eq!(fact.leg_jurisdiction.country.as_deref(), Some("MX"));
        assert_eq!(fact.finality, 0);
        // Same bytes, same fact.
        let (_, fact2) = extract(DOCKET.as_bytes()).unwrap();
        assert_eq!(
            serde_json::to_string(&fact).unwrap(),
            serde_json::to_string(&fact2).unwrap()
        );
    }

    #[test]
    fn extract_rejects_unknown_format_and_unknown_fields() {
        let bad = DOCKET.replace(DOCKET_FORMAT, "somebody-elses-docket/9.9");
        assert!(extract(bad.as_bytes()).is_err());
        let extra = DOCKET.replacen('{', "{\"surprise\":true,", 1);
        assert!(extract(extra.as_bytes()).is_err());
    }

    #[test]
    fn attest_produces_anchored_signed_record() {
        let (private, public) = generate_test_keypair();
        let record = attest(DOCKET.as_bytes(), &params(), &private).unwrap();

        assert_eq!(record.source_hash, anchor_source(DOCKET.as_bytes()));
        assert_eq!(record.content_hash, record.compute_content_hash().unwrap());
        assert!(record.lbry_id.starts_with("lbry-"));
        assert!(record.verify_signature_raw(&public).unwrap());

        // Tampering with the anchored source hash breaks the signature.
        let mut tampered = record.clone();
        tampered.source_hash = "sha256:00".to_string();
        assert!(!tampered.verify_signature_raw(&public).unwrap());
    }

    #[test]
    fn attest_is_deterministic_for_fixed_inputs() {
        let (private, _) = generate_test_keypair();
        let a = attest(DOCKET.as_bytes(), &params(), &private).unwrap();
        let b = attest(DOCKET.as_bytes(), &params(), &private).unwrap();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }
}
