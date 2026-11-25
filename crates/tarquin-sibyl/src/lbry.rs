//! LBRY structured-attestation record — a Rust model of `lbry-legal-attestation.md`.
//!
//! An LBRY record is the payload a Sibyl emits and that Lex.exe uses as the `proof_ref` for an
//! Arrow LEG event. This module models the record, computes its content hash deterministically,
//! verifies its signature (via `arrow-identity`), and validates it against the Sibyl registry.

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use arrow_core::types::event::LegJurisdiction;
use arrow_core::types::message::Action;
use arrow_identity::crypto::ed25519::verify_ed25519;
use arrow_identity::resolver::DidResolver;
use arrow_identity::types::{SignatureAlgorithm, VerificationInput, VerificationOutcome};
use arrow_identity::verify::verify_signature;

use crate::error::SibylError;
use crate::registry::{LegSubtype, SibylRegistry};
use crate::validate::{Finding, Severity};

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

/// A Libri IX (LBRY) structured attestation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbryRecord {
    pub lbry_id: String,
    pub schema_version: String,
    /// Sibyl DID; MUST be active in the registry.
    pub attested_by: String,
    /// ISO 8601 timestamp of attestation.
    pub attested_at: String,
    pub source_ref: String,
    pub source_hash: String,
    pub structured_fact: StructuredFact,
    /// Hash over canonicalized content excluding `signature` and `content_hash`.
    pub content_hash: String,
    pub signature: LbrySignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredFact {
    pub instrument_ref: String,
    pub leg_subtype: LegSubtype,
    pub leg_jurisdiction: LegJurisdiction,
    #[serde(default)]
    pub legal_system_refs: Vec<String>,
    pub effective_at: String,
    /// Arrow Finality of the attested fact: `1` final, `0` non-final / under appeal.
    /// v0.1 concretization of the LBRY model — required to classify the LEG event.
    pub finality: u8,
    /// Short legal reason code (e.g. `CO` compliance, `SA` sanctions, `LR`
    /// legislative reference). Drives Arrow's LEG-006/007 Action derivation.
    pub reason_code: String,
    pub operative_claim: String,
    pub policy_claim: Action,
    #[serde(default)]
    pub related_slc_artifacts: Vec<SlcArtifactRef>,
}

/// Pinned reference to a Orlog SLC artifact bundle backing this legal fact.
///
/// MC-006 handoff convention: downstream references identify a fixture case by sealed
/// identifiers — `{artifact_id, version, artifact_hash, fixture_case}` — never by mutable
/// prose case names. `computed_result_hash` additionally seals the Lex.Codex-recomputed
/// result for that case, so a consumer can detect both bundle drift (artifact_hash) and
/// computation drift (computed_result_hash) without re-running the SLC logic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlcArtifactRef {
    /// Sealed Orlog artifact id, e.g. `slc-late-payment-001`.
    pub artifact_id: String,
    /// Artifact manifest version, e.g. `0.1.0`.
    pub version: String,
    /// MC-006 artifact hash over the sealed bundle (`sha256:<hex>`).
    pub artifact_hash: String,
    /// Stable fixture case id, e.g. `disputed-001`.
    pub fixture_case: String,
    /// `sha256:<hex>` over the sorted-key JSON of the recomputed fixture result.
    pub computed_result_hash: String,
}

/// Signature material. Concretizes the spec's bare `signature` field into a key reference,
/// algorithm, and base64 signature so it can be checked with `arrow-identity`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbrySignature {
    /// DID URL of the signing key, e.g. `did:key:z6Mk...#z6Mk...`.
    pub key_ref: String,
    /// Algorithm identifier, e.g. `Ed25519`.
    pub algorithm: String,
    /// Base64 (standard, no padding) signature over the canonical content bytes.
    pub sig_b64: String,
}

impl LbryRecord {
    pub fn from_json(s: &str) -> Result<Self, SibylError> {
        serde_json::from_str(s).map_err(|e| SibylError::Parse(e.to_string()))
    }

    /// Canonical content bytes: the record serialized with `signature` and `content_hash`
    /// removed. `serde_json::Value`'s default map is key-sorted, so this is deterministic.
    ///
    /// NOTE (v0.1): this is sorted-key JSON, not full RFC 8785 JCS. Aligning with Arrow's
    /// `arrow-canonical-json-v0` is a follow-up before cross-stack signature interop.
    pub fn canonical_content_bytes(&self) -> Result<Vec<u8>, SibylError> {
        let mut v = serde_json::to_value(self).map_err(|e| SibylError::Parse(e.to_string()))?;
        if let Some(obj) = v.as_object_mut() {
            obj.remove("signature");
            obj.remove("content_hash");
        }
        serde_json::to_vec(&v).map_err(|e| SibylError::Parse(e.to_string()))
    }

    /// Compute the expected `content_hash` value (`sha256:<hex>`).
    pub fn compute_content_hash(&self) -> Result<String, SibylError> {
        let bytes = self.canonical_content_bytes()?;
        let digest = Sha256::digest(&bytes);
        Ok(format!("sha256:{}", to_hex(&digest)))
    }

    /// Verify the signature against raw Ed25519 public-key bytes (no DID resolution).
    pub fn verify_signature_raw(&self, public_key_bytes: &[u8]) -> Result<bool, SibylError> {
        let sig = B64
            .decode(self.signature.sig_b64.as_bytes())
            .map_err(|e| SibylError::SignatureDecode(e.to_string()))?;
        let payload = self.canonical_content_bytes()?;
        Ok(verify_ed25519(public_key_bytes, &payload, &sig).is_ok())
    }

    /// Verify the signature by resolving `signature.key_ref` through a DID resolver.
    pub fn verify_signature_did<R: DidResolver>(
        &self,
        resolver: &R,
    ) -> Result<VerificationOutcome, SibylError> {
        let algorithm: SignatureAlgorithm = self
            .signature
            .algorithm
            .parse()
            .map_err(SibylError::Identity)?;
        let sig = B64
            .decode(self.signature.sig_b64.as_bytes())
            .map_err(|e| SibylError::SignatureDecode(e.to_string()))?;
        let payload = self.canonical_content_bytes()?;
        let input = VerificationInput {
            payload: &payload,
            key_ref: &self.signature.key_ref,
            algorithm,
            signature_bytes: &sig,
        };
        verify_signature(resolver, input).map_err(|e| SibylError::Identity(e.to_string()))
    }
}

/// Validate an LBRY record against the registry (the `LBRY-*` checks). Signature *cryptographic*
/// verification is separate (`verify_signature_*`); this checks structural/scope/registry rules.
///
/// `now` is an ISO 8601 UTC instant supplied by the caller (determinism: no runtime clock).
pub fn validate_lbry(record: &LbryRecord, registry: &SibylRegistry, now: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<Finding>, rule, sev, msg: String| {
        out.push(Finding {
            rule,
            severity: sev,
            message: msg,
            sibyl_id: None,
        })
    };

    // LBRY-001 / LBRY-002 — attested_by MUST be a registered, active Sibyl.
    let sibyl = match registry.find_by_did(&record.attested_by) {
        None => {
            push(
                &mut out,
                "LBRY-001",
                Severity::Fatal,
                format!("attested_by DID not in registry: {}", record.attested_by),
            );
            return out; // remaining checks need the entry
        }
        Some(s) => s,
    };
    if !sibyl.is_active() {
        push(
            &mut out,
            "LBRY-002",
            Severity::Fatal,
            format!("attesting Sibyl {} is not active", sibyl.sibyl_id),
        );
    }

    // LBRY-003 — leg_subtype within the Sibyl's scope.
    if !sibyl.scope_allows_subtype(record.structured_fact.leg_subtype) {
        push(
            &mut out,
            "LBRY-003",
            Severity::Fatal,
            "leg_subtype outside the Sibyl's attestation scope".to_string(),
        );
    }

    // LBRY-004 — jurisdiction (country or supra) within the Sibyl's authority profile.
    let j = &record.structured_fact.leg_jurisdiction;
    let jurisdiction_ok = match (&j.country, &j.supra) {
        (None, None) => false,
        _ => {
            j.country
                .as_deref()
                .map(|c| sibyl.scope_allows_jurisdiction(c))
                .unwrap_or(true)
                && j.supra
                    .as_deref()
                    .map(|s| sibyl.scope_allows_jurisdiction(s))
                    .unwrap_or(true)
        }
    };
    if !jurisdiction_ok {
        push(
            &mut out,
            "LBRY-004",
            Severity::Fatal,
            "leg_jurisdiction outside the Sibyl's authority profile (or absent)".to_string(),
        );
    }

    // LBRY-006 — content_hash MUST match the recomputed canonical hash.
    match record.compute_content_hash() {
        Ok(h) if h == record.content_hash => {}
        Ok(h) => push(
            &mut out,
            "LBRY-006",
            Severity::Fatal,
            format!(
                "content_hash mismatch: stored {} computed {h}",
                record.content_hash
            ),
        ),
        Err(e) => push(
            &mut out,
            "LBRY-006",
            Severity::Fatal,
            format!("content_hash could not be computed: {e}"),
        ),
    }

    // LBRY-007 — signature.key_ref MUST be an active attestation key on the Sibyl.
    if sibyl
        .active_attestation_key(&record.signature.key_ref)
        .is_none()
    {
        push(
            &mut out,
            "LBRY-007",
            Severity::Fatal,
            format!(
                "signature key_ref {} is not an active attestation key on {}",
                record.signature.key_ref, sibyl.sibyl_id
            ),
        );
    }

    // LBRY-008 — attested_at MUST NOT be in the future relative to `now`.
    if record.attested_at.as_str() > now {
        push(
            &mut out,
            "LBRY-008",
            Severity::Fatal,
            format!(
                "attested_at {} is in the future (now {now})",
                record.attested_at
            ),
        );
    }

    out
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::SibylRegistry;
    use crate::validate::has_fatal;
    use arrow_identity::crypto::ed25519::{generate_test_keypair, sign_ed25519};

    const EXAMPLE: &str = include_str!("../../../drafts/sibyl/sibyl-registry.example.json");

    fn sample_record(content_hash: String, sig_b64: String) -> LbryRecord {
        LbryRecord {
            lbry_id: "lbry-test-001".to_string(),
            schema_version: "0.1".to_string(),
            attested_by: "did:example:sibyl-mx-court-001".to_string(),
            attested_at: "2026-06-07T00:00:00Z".to_string(),
            source_ref: "docket://mx/example/2026".to_string(),
            source_hash: "sha256:00".to_string(),
            structured_fact: StructuredFact {
                instrument_ref: "court-order-2026-example".to_string(),
                leg_subtype: LegSubtype::Jud,
                leg_jurisdiction: LegJurisdiction {
                    country: Some("MX".to_string()),
                    supra: None,
                },
                legal_system_refs: vec!["LS-MX-STATE-01".to_string()],
                effective_at: "2026-06-07T00:00:00Z".to_string(),
                finality: 0,
                reason_code: "CO".to_string(),
                operative_claim: "explicit_authorization".to_string(),
                policy_claim: Action("HL".to_string()),
                related_slc_artifacts: vec![],
            },
            content_hash,
            signature: LbrySignature {
                key_ref: "did:example:sibyl-mx-court-001#attestation-1".to_string(),
                algorithm: "Ed25519".to_string(),
                sig_b64,
            },
        }
    }

    #[test]
    fn content_hash_is_deterministic_and_excludes_signature() {
        let mut rec = sample_record(String::new(), String::new());
        let h1 = rec.compute_content_hash().unwrap();
        // Mutating the signature must NOT change the content hash.
        rec.signature.sig_b64 = "ZZZZ".to_string();
        let h2 = rec.compute_content_hash().unwrap();
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
    }

    #[test]
    fn raw_signature_roundtrip_verifies() {
        let (private, public) = generate_test_keypair();
        let mut rec = sample_record(String::new(), String::new());
        let payload = rec.canonical_content_bytes().unwrap();
        let sig = sign_ed25519(&private, &payload);
        rec.signature.sig_b64 = B64.encode(sig);
        assert!(rec.verify_signature_raw(&public).unwrap());

        // A tampered fact invalidates the signature.
        rec.structured_fact.operative_claim = "tampered".to_string();
        assert!(!rec.verify_signature_raw(&public).unwrap());
    }

    #[test]
    fn validate_lbry_accepts_well_formed_record() {
        // Registry entry in the example is `draft`; flip it to active for this check.
        let mut reg = SibylRegistry::from_json(EXAMPLE).unwrap();
        reg.entries[0].status = crate::registry::SibylStatus::Active;

        let mut rec = sample_record(String::new(), "AA".to_string());
        rec.content_hash = rec.compute_content_hash().unwrap();

        let findings = validate_lbry(&rec, &reg, "2026-06-08T00:00:00Z");
        assert!(!has_fatal(&findings), "unexpected findings: {findings:?}");
    }

    #[test]
    fn validate_lbry_flags_unknown_sibyl_and_future_timestamp() {
        let reg = SibylRegistry::from_json(EXAMPLE).unwrap();
        let mut rec = sample_record(String::new(), "AA".to_string());
        rec.attested_by = "did:example:not-registered".to_string();
        let findings = validate_lbry(&rec, &reg, "2020-01-01T00:00:00Z");
        assert!(findings.iter().any(|f| f.rule == "LBRY-001"));
    }
}
