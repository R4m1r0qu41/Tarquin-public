//! Sibyl registry validation — the `SBYL-001..008` rules from `sibyl-registry.md`.
//!
//! Validation returns a list of [`Finding`]s rather than failing fast, so a tool can report
//! every problem in a registry at once. A registry is *acceptable* when it produces no
//! `Severity::Fatal` findings.

use crate::registry::{SibylEntry, SibylRegistry, VerificationRegime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Fatal,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub sibyl_id: Option<String>,
}

impl Finding {
    fn fatal(rule: &'static str, sibyl_id: &str, msg: impl Into<String>) -> Self {
        Finding {
            rule,
            severity: Severity::Fatal,
            message: msg.into(),
            sibyl_id: Some(sibyl_id.to_string()),
        }
    }
    fn warning(rule: &'static str, sibyl_id: &str, msg: impl Into<String>) -> Self {
        Finding {
            rule,
            severity: Severity::Warning,
            message: msg.into(),
            sibyl_id: Some(sibyl_id.to_string()),
        }
    }
}

/// Validate a whole registry: cross-entry rules (SBYL-001 uniqueness) plus each entry.
pub fn validate_registry(reg: &SibylRegistry) -> Vec<Finding> {
    let mut out = Vec::new();

    // SBYL-001 — sibyl_id MUST be unique.
    for (i, e) in reg.entries.iter().enumerate() {
        if reg.entries[..i].iter().any(|p| p.sibyl_id == e.sibyl_id) {
            out.push(Finding::fatal(
                "SBYL-001",
                &e.sibyl_id,
                "duplicate sibyl_id in registry",
            ));
        }
    }

    for e in &reg.entries {
        out.extend(validate_entry(e));
    }
    out
}

/// Validate a single entry against SBYL-001..008 (entry-scoped rules).
pub fn validate_entry(e: &SibylEntry) -> Vec<Finding> {
    let mut out = Vec::new();
    let id = e.sibyl_id.as_str();

    // SBYL-001 — sibyl_id MUST be a stable, well-formed identifier.
    if !valid_sibyl_id(&e.sibyl_id) {
        out.push(Finding::fatal(
            "SBYL-001",
            id,
            "sibyl_id must match ^SBYL-[A-Z0-9-]+-[0-9]{3,}$",
        ));
    }

    // SBYL-002 — did MUST be present and well-formed (resolution is a separate runtime check).
    if !valid_did(&e.did) {
        out.push(Finding::fatal(
            "SBYL-002",
            id,
            "did must match ^did:[a-z0-9]+:.+",
        ));
    }

    // SBYL-003 — recognized_authority_ref MUST be present (provisional until RFC-007 Part B).
    if e.authority_profile
        .recognized_authority_ref
        .trim()
        .is_empty()
    {
        out.push(Finding::fatal(
            "SBYL-003",
            id,
            "recognized_authority_ref must be present",
        ));
    }

    // SBYL-004 / SBYL-005 — declared Confidence ceiling MUST NOT exceed the regime ceiling.
    let regime = e.attestation_scope.verification_regime;
    let ceiling = regime.confidence_ceiling();
    if e.attestation_scope.max_confidence.rank() > ceiling.rank() {
        let rule = match regime {
            VerificationRegime::StructuredLegalAttestation => "SBYL-005",
            VerificationRegime::MultipleLegalAttestation => "SBYL-005",
            VerificationRegime::InstitutionalAttestation => "SBYL-004",
        };
        out.push(Finding::fatal(
            rule,
            id,
            format!(
                "max_confidence {:?} exceeds ceiling {:?} for regime {:?}",
                e.attestation_scope.max_confidence, ceiling, regime
            ),
        ));
    }

    // SBYL-007 — at least one active attestation key MUST exist.
    if !e.has_active_attestation_key() {
        out.push(Finding::fatal(
            "SBYL-007",
            id,
            "at least one active attestation key is required",
        ));
    }

    // SBYL-008 — scope SHOULD be narrow (domains explicit).
    if e.attestation_scope.domains.is_empty() {
        out.push(Finding::warning(
            "SBYL-008",
            id,
            "attestation_scope.domains is empty; scope should be explicit",
        ));
    }

    out
}

/// True if any finding is fatal.
pub fn has_fatal(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::Fatal)
}

// ── pattern helpers (no regex dependency) ─────────────────────────────────────

/// `^SBYL-[A-Z0-9-]+-[0-9]{3,}$`
fn valid_sibyl_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("SBYL-") else {
        return false;
    };
    let Some(pos) = rest.rfind('-') else {
        return false;
    };
    let mid = &rest[..pos];
    let num = &rest[pos + 1..];
    if mid.is_empty() || num.len() < 3 || !num.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    mid.bytes()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
}

/// `^did:[a-z0-9]+:.+`
fn valid_did(did: &str) -> bool {
    let Some(rest) = did.strip_prefix("did:") else {
        return false;
    };
    let Some(colon) = rest.find(':') else {
        return false;
    };
    let method = &rest[..colon];
    let id = &rest[colon + 1..];
    !method.is_empty()
        && method
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && !id.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::SibylRegistry;

    const EXAMPLE: &str = include_str!("../../../drafts/sibyl/sibyl-registry.example.json");
    const DOCVERIFY_EXAMPLE: &str =
        include_str!("../../../drafts/sibyl/sibyl-registry.docverify.example.json");

    #[test]
    fn docverify_example_registry_has_no_fatal_findings() {
        let reg = SibylRegistry::from_json(DOCVERIFY_EXAMPLE).unwrap();
        let findings = validate_registry(&reg);
        assert!(
            !has_fatal(&findings),
            "expected no fatal findings, got: {findings:?}"
        );
    }

    #[test]
    fn docverify_institutional_attestation_ceiling_breach() {
        // institutional_attestation caps at Confidence C; B exceeds it (SBYL-004).
        let reg = SibylRegistry::from_json(DOCVERIFY_EXAMPLE).unwrap();
        let mut entry = reg.entries[0].clone();
        entry.attestation_scope.max_confidence = crate::registry::Confidence::B;
        let findings = validate_entry(&entry);
        assert!(findings.iter().any(|f| f.rule == "SBYL-004"));
    }

    #[test]
    fn example_registry_has_no_fatal_findings() {
        let reg = SibylRegistry::from_json(EXAMPLE).unwrap();
        let findings = validate_registry(&reg);
        assert!(
            !has_fatal(&findings),
            "expected no fatal findings, got: {findings:?}"
        );
    }

    #[test]
    fn id_pattern() {
        assert!(valid_sibyl_id("SBYL-MX-COURT-001"));
        assert!(valid_sibyl_id("SBYL-US-001"));
        assert!(!valid_sibyl_id("SBYL-MX-01")); // < 3 digits
        assert!(!valid_sibyl_id("sbyl-mx-001")); // wrong case / prefix
        assert!(!valid_sibyl_id("SBYL-001")); // no mid segment
    }

    #[test]
    fn did_pattern() {
        assert!(valid_did("did:example:abc"));
        assert!(valid_did("did:key:z6Mkfoo"));
        assert!(!valid_did("did:UPPER:abc"));
        assert!(!valid_did("notadid"));
    }

    #[test]
    fn detects_confidence_ceiling_breach() {
        let reg = SibylRegistry::from_json(EXAMPLE).unwrap();
        let mut entry = reg.entries[0].clone();
        entry.attestation_scope.max_confidence = crate::registry::Confidence::A;
        let findings = validate_entry(&entry);
        assert!(findings.iter().any(|f| f.rule == "SBYL-005"));
    }
}
