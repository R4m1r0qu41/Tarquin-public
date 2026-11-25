//! Document-verification attestation — the deterministic reduction for the SBL-9
//! document-verification Sibyl leg (`SBL-9/document-verification-attestation.md`).
//!
//! Document *extraction* (QR/barcode decode, PDF signature and CMS/PKCS#7 parsing,
//! OCSP/CRL/registry I/O) is I/O-bound and time-sensitive and runs **outside** this
//! AI-free runtime as an upstream producer (`crates/tarquin-sibyl-docverify`). What lives here
//! is the deterministic, fail-closed reduction of that producer's *per-dimension, evidence-backed*
//! claims into a constrained legal-semantic finding. No inference, no I/O.
//!
//! Every raised dimension must be independently auditable: it carries its own
//! [`SemanticState`], its own evidence references (each with a [`EvidenceKind`] and its own
//! `lex_refs`), and the reducer enforces a per-dimension **evidence-kind allowlist** plus the
//! predicate chains for the two derived dimensions. `lex_refs` are auditable pointers
//! (human-verifiable citations) — the reducer checks structural non-emptiness per dimension;
//! per-dimension semantic lex validation is deferred to the profile system in the engine crate.
//!
//! Rules:
//! - DV-001 every dimension defaults to `not_determined`.
//! - DV-002 `document_authenticity` requires confirmed `document_integrity` + `issuer_identity`.
//! - DV-003 `legal_effect` requires confirmed `signer_capacity` + `filing_status` +
//!   `document_authenticity` (so in practice `not_determined`, since capacity is not mechanized).
//! - DV-004 a QR decode never grounds authenticity.
//! - DV-005 `RegistryLookup` ≠ filing — a registry lookup can ground `issuer_identity` and
//!   contribute to `document_authenticity` (via confirmed issuer) but never raises `filing_status`
//!   or `legal_effect`.
//! - DV-006 each raised dimension needs at least one evidence reference of an allowed kind that
//!   carries its own non-empty `lex_refs`.

use serde::{Deserialize, Serialize};

/// Constrained per-dimension conclusion. Defaults to [`SemanticState::NotDetermined`]
/// (fail-closed): a dimension is only raised on evidence the producer supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SemanticState {
    /// Insufficient evidence — the safe default.
    #[default]
    NotDetermined,
    /// Evidence affirmatively contradicts the claim.
    Refuted,
    /// Some, but not sufficient, supporting evidence.
    PartiallyConfirmed,
    /// Fully supported by the cited evidence and grounding.
    Confirmed,
}

impl SemanticState {
    /// Whether the state is exactly [`SemanticState::Confirmed`].
    #[must_use]
    pub fn is_confirmed(self) -> bool {
        matches!(self, SemanticState::Confirmed)
    }

    fn is_raised(self) -> bool {
        matches!(
            self,
            SemanticState::Confirmed | SemanticState::PartiallyConfirmed
        )
    }
}

/// Closed vocabulary of how a piece of evidence was obtained. Detection and validation are
/// deliberately distinct: detecting a signature object is not validating it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    ArtifactHash,
    ProfileMatch,
    QrDecode,
    OcrExtraction,
    PdfSignatureDetection,
    PdfSignatureValidation,
    CertificateChainValidation,
    RegistryLookup,
    FilingLookup,
}

/// A single piece of evidence supporting a dimension claim. Grounding is per-evidence: each ref
/// names its own `lex_refs` and points (`evidence_id`) into the content-hashed evidence manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub kind: EvidenceKind,
    #[serde(default)]
    pub lex_refs: Vec<String>,
    /// Pointer into the evidence manifest (a content-hashed entry id).
    pub evidence_id: String,
}

impl EvidenceRef {
    fn is_grounded(&self) -> bool {
        !self.lex_refs.is_empty() && !self.evidence_id.trim().is_empty()
    }
}

/// One dimension's proposed state plus the evidence the producer offers for it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionClaim {
    #[serde(default)]
    pub state: SemanticState,
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
}

impl DimensionClaim {
    /// At least one grounded evidence ref of an allowed kind.
    fn has_allowed_grounding(&self, allowed: &[EvidenceKind]) -> bool {
        self.evidence
            .iter()
            .any(|e| e.is_grounded() && allowed.contains(&e.kind))
    }
}

/// Canonical content hash form used by the producer/reducer boundary.
fn is_canonical_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Raw per-dimension claims from the out-of-runtime extraction engine, before reduction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimedFinding {
    /// `sha256:<hex>` of the ingested artifact bytes.
    pub artifact_sha256: String,
    /// Matched document profile, e.g. `MX-RENAPO-CURP-QR-01`.
    pub profile_id: String,
    /// `sha256:<hex>` of the matched profile definition (pins which rules produced this claim).
    pub profile_hash: String,
    /// `sha256:<hex>` over the canonical evidence manifest the `evidence_id`s point into.
    pub evidence_manifest_hash: String,
    /// ISO 8601 time the verification was performed (supplied explicitly — never an implicit clock).
    pub checked_at: String,
    #[serde(default)]
    pub document_integrity: DimensionClaim,
    #[serde(default)]
    pub issuer_identity: DimensionClaim,
    #[serde(default)]
    pub signer_identity: DimensionClaim,
    #[serde(default)]
    pub signer_capacity: DimensionClaim,
    #[serde(default)]
    pub registry_status: DimensionClaim,
    #[serde(default)]
    pub filing_status: DimensionClaim,
    #[serde(default)]
    pub document_authenticity: DimensionClaim,
    #[serde(default)]
    pub legal_effect: DimensionClaim,
}

/// The reduced, constrained finding — safe to lower into an LBRY `StructuredFact`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentVerificationFinding {
    pub artifact_sha256: String,
    pub profile_id: String,
    pub profile_hash: String,
    pub evidence_manifest_hash: String,
    pub checked_at: String,
    pub document_integrity: SemanticState,
    pub issuer_identity: SemanticState,
    pub signer_identity: SemanticState,
    pub signer_capacity: SemanticState,
    pub registry_status: SemanticState,
    pub filing_status: SemanticState,
    pub document_authenticity: SemanticState,
    pub legal_effect: SemanticState,
    /// Human-readable record of every rule-driven downgrade applied during reduction.
    pub warnings: Vec<String>,
}

impl DocumentVerificationFinding {
    /// Whether this finding must be flagged for human legal review. Capacity and legal effect are
    /// legal judgments the leg does not mechanize, so this is true unless both were established —
    /// in practice, always.
    #[must_use]
    pub fn requires_human_review(&self) -> bool {
        !(self.signer_capacity.is_confirmed() && self.legal_effect.is_confirmed())
    }
}

/// Reduce a leaf (non-derived) dimension under its evidence-kind allowlist.
fn reduce_leaf(
    claim: &DimensionClaim,
    confirm_allowed: &[EvidenceKind],
    refute_allowed: &[EvidenceKind],
    name: &str,
    warnings: &mut Vec<String>,
) -> SemanticState {
    match claim.state {
        SemanticState::NotDetermined => SemanticState::NotDetermined,
        SemanticState::Refuted => pass_refuted(claim, refute_allowed, name, warnings),
        SemanticState::PartiallyConfirmed | SemanticState::Confirmed => {
            if claim.has_allowed_grounding(confirm_allowed) {
                claim.state
            } else {
                warnings.push(format!(
                    "DV-006: {name} raised without a grounded allowed-kind evidence ref; held at not_determined"
                ));
                SemanticState::NotDetermined
            }
        }
    }
}

/// A `refuted` finding passes through only if it carries at least one grounded ref of an allowed
/// negative-evidence kind for this dimension. Confirmation and refutation use separate allowlists:
/// evidence that fails to confirm a conclusion does not necessarily refute it.
fn pass_refuted(
    claim: &DimensionClaim,
    allowed: &[EvidenceKind],
    name: &str,
    warnings: &mut Vec<String>,
) -> SemanticState {
    if claim.has_allowed_grounding(allowed) {
        SemanticState::Refuted
    } else {
        warnings.push(format!(
            "DV-006: {name} refuted without allowed-kind grounding; held at not_determined"
        ));
        SemanticState::NotDetermined
    }
}

/// `document_integrity`: only `PdfSignatureValidation` (covered byte range) confirms it; an
/// `ArtifactHash` alone caps at `partially_confirmed`.
fn reduce_integrity(claim: &DimensionClaim, warnings: &mut Vec<String>) -> SemanticState {
    match claim.state {
        SemanticState::NotDetermined => SemanticState::NotDetermined,
        SemanticState::Refuted => pass_refuted(
            claim,
            &[EvidenceKind::PdfSignatureValidation],
            "document_integrity",
            warnings,
        ),
        SemanticState::PartiallyConfirmed | SemanticState::Confirmed => {
            if claim.has_allowed_grounding(&[EvidenceKind::PdfSignatureValidation]) {
                claim.state
            } else if claim.has_allowed_grounding(&[EvidenceKind::ArtifactHash]) {
                if claim.state == SemanticState::Confirmed {
                    warnings.push(
                        "document_integrity capped to partially_confirmed: artifact hash alone is not tamper-evidence (needs signature validation)"
                            .to_string(),
                    );
                }
                SemanticState::PartiallyConfirmed
            } else {
                warnings.push(
                    "DV-006: document_integrity raised without hash/signature-validation grounding; held at not_determined"
                        .to_string(),
                );
                SemanticState::NotDetermined
            }
        }
    }
}

/// `signer_capacity` is a legal judgment with no mechanized evidence kind: always not_determined.
fn reduce_capacity(claim: &DimensionClaim, warnings: &mut Vec<String>) -> SemanticState {
    if claim.state != SemanticState::NotDetermined {
        warnings.push(
            "signer_capacity is a legal judgment the leg does not mechanize; held at not_determined"
                .to_string(),
        );
    }
    SemanticState::NotDetermined
}

/// `document_authenticity` (derived): confirmed only with confirmed integrity + issuer AND a
/// grounded allowed-kind ref; QR decode never grounds it (DV-004).
fn reduce_authenticity(
    claim: &DimensionClaim,
    integrity: SemanticState,
    issuer: SemanticState,
    warnings: &mut Vec<String>,
) -> SemanticState {
    let confirm_allowed = [
        EvidenceKind::PdfSignatureValidation,
        EvidenceKind::CertificateChainValidation,
        EvidenceKind::RegistryLookup,
    ];
    if claim.state == SemanticState::Refuted {
        return pass_refuted(claim, &[], "document_authenticity", warnings);
    }
    if !claim.has_allowed_grounding(&confirm_allowed) {
        if claim.state.is_raised() {
            warnings.push(
                "DV-004: document_authenticity needs validated signature/certificate/registry evidence (a QR decode never grounds it); held at not_determined"
                    .to_string(),
            );
        }
        return SemanticState::NotDetermined;
    }
    if integrity.is_confirmed() && issuer.is_confirmed() {
        SemanticState::Confirmed
    } else {
        warnings.push(
            "DV-002: document_authenticity requires confirmed document_integrity and issuer_identity; downgraded to partially_confirmed"
                .to_string(),
        );
        SemanticState::PartiallyConfirmed
    }
}

/// `legal_effect` (derived): confirmed only with confirmed capacity + filing + authenticity.
fn reduce_legal_effect(
    claim: &DimensionClaim,
    capacity: SemanticState,
    filing: SemanticState,
    authenticity: SemanticState,
    warnings: &mut Vec<String>,
) -> SemanticState {
    if claim.state == SemanticState::Refuted {
        warnings
            .push("legal_effect refutation is not mechanized; held at not_determined".to_string());
        return SemanticState::NotDetermined;
    }
    if capacity.is_confirmed() && filing.is_confirmed() && authenticity.is_confirmed() {
        SemanticState::Confirmed
    } else {
        if claim.state.is_raised() {
            warnings.push(
                "DV-003: legal_effect held at not_determined; requires confirmed signer_capacity, filing_status, and document_authenticity"
                    .to_string(),
            );
        }
        SemanticState::NotDetermined
    }
}

/// Apply the fail-closed derivation rules to a raw claim. The two derived dimensions
/// (`document_authenticity`, `legal_effect`) are recomputed from their predicates and evidence,
/// never trusted from the producer's proposed state.
#[must_use]
pub fn reduce(claim: &ClaimedFinding) -> DocumentVerificationFinding {
    let mut warnings = Vec::new();

    let hash_fields = [
        ("artifact_sha256", claim.artifact_sha256.as_str()),
        ("profile_hash", claim.profile_hash.as_str()),
        (
            "evidence_manifest_hash",
            claim.evidence_manifest_hash.as_str(),
        ),
    ];
    for (name, value) in hash_fields {
        if !is_canonical_sha256(value) {
            warnings.push(format!(
                "{name} invalid: expected canonical sha256:<64 lowercase hex>"
            ));
        }
    }
    let anchors_invalid = hash_fields
        .iter()
        .any(|(_, value)| !is_canonical_sha256(value));

    let mut document_integrity = reduce_integrity(&claim.document_integrity, &mut warnings);
    let mut issuer_identity = reduce_leaf(
        &claim.issuer_identity,
        &[
            EvidenceKind::CertificateChainValidation,
            EvidenceKind::RegistryLookup,
        ],
        &[],
        "issuer_identity",
        &mut warnings,
    );
    let mut signer_identity = reduce_leaf(
        &claim.signer_identity,
        &[EvidenceKind::CertificateChainValidation],
        &[],
        "signer_identity",
        &mut warnings,
    );
    let mut signer_capacity = reduce_capacity(&claim.signer_capacity, &mut warnings);
    let mut registry_status = reduce_leaf(
        &claim.registry_status,
        &[EvidenceKind::RegistryLookup],
        &[EvidenceKind::RegistryLookup],
        "registry_status",
        &mut warnings,
    );
    let mut filing_status = reduce_leaf(
        &claim.filing_status,
        &[EvidenceKind::FilingLookup],
        &[EvidenceKind::FilingLookup],
        "filing_status",
        &mut warnings,
    );
    let mut document_authenticity = reduce_authenticity(
        &claim.document_authenticity,
        document_integrity,
        issuer_identity,
        &mut warnings,
    );
    let mut legal_effect = reduce_legal_effect(
        &claim.legal_effect,
        signer_capacity,
        filing_status,
        document_authenticity,
        &mut warnings,
    );

    // If any hash is missing or malformed, the artifact/profile/evidence chain cannot be
    // verified. Hold every non-not_determined conclusion, including refutations.
    if anchors_invalid {
        for (name, state) in [
            ("document_integrity", &mut document_integrity),
            ("issuer_identity", &mut issuer_identity),
            ("signer_identity", &mut signer_identity),
            ("signer_capacity", &mut signer_capacity),
            ("registry_status", &mut registry_status),
            ("filing_status", &mut filing_status),
            ("document_authenticity", &mut document_authenticity),
            ("legal_effect", &mut legal_effect),
        ] {
            if *state != SemanticState::NotDetermined {
                warnings.push(format!(
                    "evidence anchors invalid: {name} held at not_determined"
                ));
                *state = SemanticState::NotDetermined;
            }
        }
    }

    DocumentVerificationFinding {
        artifact_sha256: claim.artifact_sha256.clone(),
        profile_id: claim.profile_id.clone(),
        profile_hash: claim.profile_hash.clone(),
        evidence_manifest_hash: claim.evidence_manifest_hash.clone(),
        checked_at: claim.checked_at.clone(),
        document_integrity,
        issuer_identity,
        signer_identity,
        signer_capacity,
        registry_status,
        filing_status,
        document_authenticity,
        legal_effect,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(nibble: char) -> String {
        format!("sha256:{}", nibble.to_string().repeat(64))
    }

    fn ev(kind: EvidenceKind, lex: &str) -> EvidenceRef {
        EvidenceRef {
            kind,
            lex_refs: vec![lex.to_string()],
            evidence_id: "ev-001".to_string(),
        }
    }

    fn dim(state: SemanticState, evidence: Vec<EvidenceRef>) -> DimensionClaim {
        DimensionClaim { state, evidence }
    }

    fn base() -> ClaimedFinding {
        ClaimedFinding {
            artifact_sha256: digest('a'),
            profile_id: "MX-RENAPO-CURP-QR-01".to_string(),
            profile_hash: digest('b'),
            evidence_manifest_hash: digest('c'),
            checked_at: "2026-06-26T00:00:00Z".to_string(),
            document_integrity: DimensionClaim::default(),
            issuer_identity: DimensionClaim::default(),
            signer_identity: DimensionClaim::default(),
            signer_capacity: DimensionClaim::default(),
            registry_status: DimensionClaim::default(),
            filing_status: DimensionClaim::default(),
            document_authenticity: DimensionClaim::default(),
            legal_effect: DimensionClaim::default(),
        }
    }

    #[test]
    fn default_state_is_not_determined() {
        assert_eq!(SemanticState::default(), SemanticState::NotDetermined);
        let f = reduce(&base());
        assert_eq!(f.document_integrity, SemanticState::NotDetermined);
        assert_eq!(f.legal_effect, SemanticState::NotDetermined);
        assert!(f.requires_human_review());
    }

    #[test]
    fn dv006_ungrounded_claim_is_held() {
        let mut c = base();
        // Confirmed state but evidence ref has empty lex_refs → not grounded.
        c.registry_status = dim(
            SemanticState::Confirmed,
            vec![EvidenceRef {
                kind: EvidenceKind::RegistryLookup,
                lex_refs: vec![],
                evidence_id: "e".into(),
            }],
        );
        let f = reduce(&c);
        assert_eq!(f.registry_status, SemanticState::NotDetermined);
        assert!(f.warnings.iter().any(|w| w.starts_with("DV-006")));
    }

    #[test]
    fn cross_dimension_evidence_misuse_is_rejected() {
        let mut c = base();
        // signer_identity "confirmed" but only via a registry lookup (wrong kind).
        c.signer_identity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::RegistryLookup, "lex-us-ny-stt-304")],
        );
        let f = reduce(&c);
        assert_eq!(f.signer_identity, SemanticState::NotDetermined);
        assert!(f.warnings.iter().any(|w| w.contains("signer_identity")));
    }

    #[test]
    fn detection_without_validation_cannot_raise_signer_identity() {
        let mut c = base();
        c.signer_identity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::PdfSignatureDetection, "lex-us-ny-stt-304")],
        );
        let f = reduce(&c);
        assert_eq!(f.signer_identity, SemanticState::NotDetermined);
    }

    #[test]
    fn qr_only_cannot_raise_authenticity() {
        let mut c = base();
        c.document_integrity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::PdfSignatureValidation, "lex-x")],
        );
        c.issuer_identity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::RegistryLookup, "lex-y")],
        );
        c.document_authenticity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::QrDecode, "lex-z")],
        );
        let f = reduce(&c);
        assert_eq!(f.document_authenticity, SemanticState::NotDetermined);
        assert!(f.warnings.iter().any(|w| w.starts_with("DV-004")));
    }

    #[test]
    fn registry_only_cannot_raise_filing_status() {
        let mut c = base();
        c.filing_status = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::RegistryLookup, "lex-us-ny-cplr-4539")],
        );
        let f = reduce(&c);
        assert_eq!(f.filing_status, SemanticState::NotDetermined);
    }

    #[test]
    fn integrity_from_artifact_hash_only_caps_at_partial() {
        let mut c = base();
        c.document_integrity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::ArtifactHash, "lex-us-ny-cplr-4539")],
        );
        let f = reduce(&c);
        assert_eq!(f.document_integrity, SemanticState::PartiallyConfirmed);
        assert!(f.warnings.iter().any(|w| w.contains("artifact hash alone")));
    }

    #[test]
    fn authenticity_requires_integrity_and_issuer() {
        let mut c = base();
        // grounded allowed authenticity evidence, but integrity/issuer not confirmed.
        c.document_authenticity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::CertificateChainValidation, "lex-x")],
        );
        let f = reduce(&c);
        assert_eq!(f.document_authenticity, SemanticState::PartiallyConfirmed);
        assert!(f.warnings.iter().any(|w| w.starts_with("DV-002")));
    }

    #[test]
    fn legal_effect_held_without_capacity() {
        let mut c = base();
        c.document_integrity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::PdfSignatureValidation, "lex-x")],
        );
        c.issuer_identity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::CertificateChainValidation, "lex-y")],
        );
        c.filing_status = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::FilingLookup, "lex-z")],
        );
        c.document_authenticity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::PdfSignatureValidation, "lex-w")],
        );
        c.legal_effect = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::RegistryLookup, "lex-q")],
        );
        let f = reduce(&c);
        // authenticity should confirm, but capacity is never mechanized → legal_effect held.
        assert_eq!(f.document_authenticity, SemanticState::Confirmed);
        assert_eq!(f.signer_capacity, SemanticState::NotDetermined);
        assert_eq!(f.legal_effect, SemanticState::NotDetermined);
        assert!(f.warnings.iter().any(|w| w.starts_with("DV-003")));
        assert!(f.requires_human_review());
    }

    #[test]
    fn missing_profile_hash_is_invalid() {
        let mut c = base();
        c.profile_hash = String::new();
        let f = reduce(&c);
        assert!(f
            .warnings
            .iter()
            .any(|w| w.contains("profile_hash invalid")));
    }

    // A1: DV-005 allows RegistryLookup to ground issuer_identity (only filing_status / legal_effect
    // are barred). This test documents the intended allowlist as a regression guard.
    #[test]
    fn registry_lookup_grounds_issuer_identity() {
        let mut c = base();
        c.issuer_identity = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::RegistryLookup, "lex-mx-ccom-89")],
        );
        let f = reduce(&c);
        assert_eq!(f.issuer_identity, SemanticState::Confirmed);
        assert!(!f.warnings.iter().any(|w| w.contains("issuer_identity")));
    }

    // A2: if evidence_manifest_hash is empty every non-default conclusion is unauditable and must
    // be held. (profile_hash/artifact_sha256 invalid trigger the same path.)
    #[test]
    fn missing_manifest_hash_holds_all_raised_conclusions() {
        let mut c = base();
        c.evidence_manifest_hash = String::new();
        // Set up a dimension that would normally confirm.
        c.registry_status = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::RegistryLookup, "lex-mx-ccom-89")],
        );
        c.issuer_identity = dim(
            SemanticState::PartiallyConfirmed,
            vec![ev(EvidenceKind::RegistryLookup, "lex-mx-lfea-7")],
        );
        let f = reduce(&c);
        assert_eq!(f.registry_status, SemanticState::NotDetermined);
        assert_eq!(f.issuer_identity, SemanticState::NotDetermined);
        assert!(f.warnings.iter().any(|w| w.contains("anchors invalid")));
    }

    #[test]
    fn malformed_artifact_hash_holds_confirmed_and_refuted_conclusions() {
        let mut c = base();
        c.artifact_sha256 = "sha256:00".to_string();
        c.registry_status = dim(
            SemanticState::Refuted,
            vec![ev(EvidenceKind::RegistryLookup, "lex-mx-ccom-89")],
        );
        c.filing_status = dim(
            SemanticState::Confirmed,
            vec![ev(EvidenceKind::FilingLookup, "lex-us-ny-cplr-4539")],
        );
        let f = reduce(&c);
        assert_eq!(f.registry_status, SemanticState::NotDetermined);
        assert_eq!(f.filing_status, SemanticState::NotDetermined);
        assert!(f
            .warnings
            .iter()
            .any(|w| w.contains("artifact_sha256 invalid")));
    }

    #[test]
    fn allowed_registry_refutation_passes_with_valid_anchors() {
        let mut c = base();
        c.registry_status = dim(
            SemanticState::Refuted,
            vec![ev(EvidenceKind::RegistryLookup, "lex-mx-ccom-89")],
        );
        let f = reduce(&c);
        assert_eq!(f.registry_status, SemanticState::Refuted);
    }

    // A3: a Refuted state backed by an evidence kind outside the negative-evidence allowlist must
    // be held at not_determined.
    #[test]
    fn refuted_with_wrong_kind_is_held() {
        let mut c = base();
        // QrDecode is not in the issuer_identity allowlist; refuted must not pass through.
        c.issuer_identity = dim(
            SemanticState::Refuted,
            vec![ev(EvidenceKind::QrDecode, "lex-mx-ccom-89")],
        );
        let f = reduce(&c);
        assert_eq!(f.issuer_identity, SemanticState::NotDetermined);
        assert!(f.warnings.iter().any(|w| w.contains("issuer_identity")));
    }

    #[test]
    fn registry_lookup_cannot_refute_legal_effect() {
        let mut c = base();
        c.legal_effect = dim(
            SemanticState::Refuted,
            vec![ev(EvidenceKind::RegistryLookup, "lex-mx-ccom-89")],
        );
        let f = reduce(&c);
        assert_eq!(f.legal_effect, SemanticState::NotDetermined);
        assert!(f
            .warnings
            .iter()
            .any(|w| w.contains("legal_effect refutation is not mechanized")));
    }
}
