//! Sibyl registry types — a Rust model of `sibyl-registry.schema.json` (v0.1).
//!
//! The registry records recognized legal-oracle identities that may produce LBRY records.
//! It is designed as a profile of the RFC-007 Part B recognized-authority record so the two
//! schemas converge losslessly when Part B graduates.

use serde::{Deserialize, Serialize};

use crate::error::SibylError;

/// Top-level registry document: `{ registry_metadata, entries }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SibylRegistry {
    pub registry_metadata: RegistryMetadata,
    pub entries: Vec<SibylEntry>,
}

impl SibylRegistry {
    /// Parse a registry from its JSON representation.
    pub fn from_json(s: &str) -> Result<Self, SibylError> {
        serde_json::from_str(s).map_err(|e| SibylError::Parse(e.to_string()))
    }

    /// Find an entry by its DID, regardless of status.
    pub fn find_by_did(&self, did: &str) -> Option<&SibylEntry> {
        self.entries.iter().find(|e| e.did == did)
    }

    /// Find an entry by DID that is currently `active` (eligible to produce new LBRY records).
    pub fn active_by_did(&self, did: &str) -> Option<&SibylEntry> {
        self.find_by_did(did).filter(|e| e.is_active())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryMetadata {
    pub schema_version: String,
    pub registry_name: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SibylEntry {
    pub sibyl_id: String,
    pub did: String,
    pub name: String,
    pub authority_profile: AuthorityProfile,
    pub attestation_scope: AttestationScope,
    pub keys: Vec<Key>,
    pub status: SibylStatus,
    pub added_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl SibylEntry {
    pub fn is_active(&self) -> bool {
        self.status == SibylStatus::Active
    }

    /// True if at least one listed key is an `active` `attestation` key (SBYL-007).
    pub fn has_active_attestation_key(&self) -> bool {
        self.keys.iter().any(|k| k.is_active_attestation())
    }

    /// True if `key_ref` names an active attestation key on this Sibyl.
    pub fn active_attestation_key(&self, key_ref: &str) -> Option<&Key> {
        self.keys
            .iter()
            .find(|k| k.key_id == key_ref && k.is_active_attestation())
    }

    /// Whether the given LEG subtype is within this Sibyl's attestation scope.
    pub fn scope_allows_subtype(&self, subtype: LegSubtype) -> bool {
        self.attestation_scope.leg_subtypes.contains(&subtype)
    }

    /// Whether a jurisdiction code (country or supra) is within the authority profile.
    pub fn scope_allows_jurisdiction(&self, code: &str) -> bool {
        self.authority_profile
            .jurisdictions
            .iter()
            .any(|j| j == code)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityProfile {
    pub authority_type: AuthorityType,
    pub recognized_authority_ref: String,
    pub jurisdictions: Vec<String>,
    #[serde(default)]
    pub legal_system_refs: Vec<String>,
    pub source_authority_basis: Vec<SourceAuthorityBasis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttestationScope {
    pub leg_subtypes: Vec<LegSubtype>,
    #[serde(default)]
    pub domains: Vec<String>,
    pub max_confidence: Confidence,
    pub verification_regime: VerificationRegime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub key_id: String,
    pub purpose: KeyPurpose,
    pub algorithm: KeyAlgorithm,
    pub status: KeyStatus,
    pub valid_from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
}

impl Key {
    pub fn is_active_attestation(&self) -> bool {
        self.purpose == KeyPurpose::Attestation && self.status == KeyStatus::Active
    }
}

// ── Enums (mirror the schema's closed vocabularies) ───────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityType {
    CourtMonitor,
    RegulatoryMonitor,
    OfficialGazetteMonitor,
    NotarialAttestor,
    ArbitralAttestor,
    InstitutionalLegalOracle,
    ProtocolLegalOracle,
    /// A Sibyl identity role for the document-verification leg (it verifies legal documents rather
    /// than monitoring courts/regulators). It is a recognized-identity role only — **not** a
    /// protocol verification regime. Such a Sibyl uses an existing `VerificationRegime`
    /// (`InstitutionalAttestation`, Arrow `legal_attestation`, ceiling C). See
    /// `SBL-9/document-verification-attestation.md`.
    DocumentVerificationOracle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAuthorityBasis {
    OfficialDocket,
    CourtOrder,
    OfficialGazette,
    RegulatoryRegister,
    NotarialRecord,
    ArbitralAward,
    StatutoryPublication,
    InstitutionalAttestation,
    ProtocolRegistry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LegSubtype {
    Jud,
    Adm,
    Supra,
    LegText,
    Prec,
}

/// Confidence ladder. The schema bounds `max_confidence` to `B`/`C`/`D`, but the full A–E
/// scale is modelled so validation can reason about the ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    A,
    B,
    C,
    D,
    E,
}

impl Confidence {
    /// Higher = stronger. Used only for ceiling comparisons.
    pub fn rank(self) -> u8 {
        match self {
            Confidence::A => 5,
            Confidence::B => 4,
            Confidence::C => 3,
            Confidence::D => 2,
            Confidence::E => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationRegime {
    StructuredLegalAttestation,
    MultipleLegalAttestation,
    InstitutionalAttestation,
}

impl VerificationRegime {
    /// The Confidence ceiling this regime may claim, aligned to Arrow's live
    /// `confidence_subprofiles.json`: structured single-Sibyl/LBRY proof and independent
    /// multi-Sibyl proof both have a **B** ceiling. The ceiling is not the default; a Sibyl entry
    /// may still declare C or D if its artifacts or operating posture do not justify B.
    pub fn confidence_ceiling(self) -> Confidence {
        match self {
            VerificationRegime::StructuredLegalAttestation => Confidence::B,
            VerificationRegime::MultipleLegalAttestation => Confidence::B,
            VerificationRegime::InstitutionalAttestation => Confidence::C,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyPurpose {
    Attestation,
    RegistryAdmin,
    Revocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Ed25519,
    #[serde(rename = "secp256k1")]
    Secp256k1,
    #[serde(rename = "P-256")]
    P256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    Active,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SibylStatus {
    Active,
    Suspended,
    Revoked,
    Draft,
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../drafts/sibyl/sibyl-registry.example.json");
    const DOCVERIFY_EXAMPLE: &str =
        include_str!("../../../drafts/sibyl/sibyl-registry.docverify.example.json");

    #[test]
    fn parses_docverify_example_registry() {
        let reg = SibylRegistry::from_json(DOCVERIFY_EXAMPLE)
            .expect("docverify example registry must parse");
        let e = &reg.entries[0];
        assert_eq!(e.sibyl_id, "SBYL-MX-DOCVERIFY-001");
        assert_eq!(
            e.authority_profile.authority_type,
            AuthorityType::DocumentVerificationOracle
        );
        // Reuses an existing regime — not a new Arrow regime — at ceiling C.
        assert_eq!(
            e.attestation_scope.verification_regime,
            VerificationRegime::InstitutionalAttestation
        );
        assert_eq!(e.attestation_scope.max_confidence, Confidence::C);
        assert!(e.scope_allows_subtype(LegSubtype::Adm));
        assert!(e.scope_allows_jurisdiction("MX"));
        assert!(e.has_active_attestation_key());
    }

    #[test]
    fn parses_example_registry() {
        let reg = SibylRegistry::from_json(EXAMPLE).expect("example registry must parse");
        assert_eq!(reg.registry_metadata.registry_name, "sibyl_registry");
        assert_eq!(reg.entries.len(), 1);
        let e = &reg.entries[0];
        assert_eq!(e.sibyl_id, "SBYL-MX-COURT-001");
        assert_eq!(
            e.authority_profile.authority_type,
            AuthorityType::CourtMonitor
        );
        assert_eq!(e.attestation_scope.max_confidence, Confidence::C);
        assert!(e.scope_allows_subtype(LegSubtype::Jud));
        assert!(e.scope_allows_jurisdiction("MX"));
        assert!(e.has_active_attestation_key());
    }

    #[test]
    fn enum_string_mappings_roundtrip() {
        assert_eq!(
            serde_json::to_string(&LegSubtype::LegText).unwrap(),
            "\"LEG_TEXT\""
        );
        assert_eq!(
            serde_json::to_string(&KeyAlgorithm::P256).unwrap(),
            "\"P-256\""
        );
        assert_eq!(
            serde_json::to_string(&VerificationRegime::StructuredLegalAttestation).unwrap(),
            "\"structured_legal_attestation\""
        );
    }

    #[test]
    fn regime_ceilings() {
        assert_eq!(
            VerificationRegime::StructuredLegalAttestation.confidence_ceiling(),
            Confidence::B
        );
        assert!(Confidence::A.rank() > Confidence::B.rank());
        assert!(Confidence::B.rank() > Confidence::C.rank());
    }
}
