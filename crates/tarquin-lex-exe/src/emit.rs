//! Phase 3.3 — the LEG **emit** path.
//!
//! Maps a validated LBRY record (+ its registered Sibyl) into the `arrow-origin-leg`
//! [`LegalAttestation`] boundary object and on to a protocol-valid `ArrowEvent`. The producer
//! gate (`OA-*`) runs first; a fatal finding blocks emission.

use arrow_core::types::event::ArrowEvent;
use arrow_identity::resolver::{DidResolver, MultiResolver};
use arrow_origin_leg::{map_to_arrow, LegalAttestation};

use tarquin_sibyl::lbry::{validate_lbry, LbryRecord};
use tarquin_sibyl::registry::{Confidence, LegSubtype, SibylEntry, VerificationRegime};
use tarquin_sibyl::validate::{has_fatal, Finding, Severity};
use tarquin_sibyl::SibylRegistry;

/// Emission was blocked by one or more fatal preflight findings.
#[derive(Debug, Clone)]
pub struct EmissionBlocked {
    pub findings: Vec<Finding>,
}

impl std::fmt::Display for EmissionBlocked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "emission blocked by {} fatal finding(s)",
            self.findings.len()
        )
    }
}
impl std::error::Error for EmissionBlocked {}

/// Producer-side preflight: the `OA-*` gate (`lex-exe/leg-origin-adapter.md`).
///
/// Composes the Sibyl-layer LBRY checks (`LBRY-*`, which already cover OA-002/003) with the
/// emitter-specific obligations OA-006 (no Confidence `A`) and OA-007 (no false regime).
/// `now` is supplied by the caller (no runtime clock).
pub fn preflight(record: &LbryRecord, registry: &SibylRegistry, now: &str) -> Vec<Finding> {
    preflight_with_resolver(record, registry, now, &MultiResolver::default())
}

/// Producer-side preflight with an explicit DID resolver.
///
/// Use this when deployments need non-default identity methods. The plain [`preflight`] entry
/// point uses Arrow Identity's default resolver set (`did:key` offline, plus configured features).
pub fn preflight_with_resolver<R: DidResolver>(
    record: &LbryRecord,
    registry: &SibylRegistry,
    now: &str,
    resolver: &R,
) -> Vec<Finding> {
    let mut out = validate_lbry(record, registry, now);

    let Some(sibyl) = registry.find_by_did(&record.attested_by) else {
        return out; // LBRY-001 already recorded the unknown Sibyl
    };

    // OA-006 — Confidence A is never permissible for a LEG-origin event.
    if sibyl.attestation_scope.max_confidence == Confidence::A {
        out.push(Finding {
            rule: "OA-006",
            severity: Severity::Fatal,
            message: "Confidence A is prohibited for LEG-origin events".to_string(),
            sibyl_id: Some(sibyl.sibyl_id.clone()),
        });
    }

    // OA-007 — no false verification regime: structured legal attestation requires real anchoring.
    if sibyl.attestation_scope.verification_regime == VerificationRegime::StructuredLegalAttestation
        && (record.content_hash.trim().is_empty() || record.source_hash.trim().is_empty())
    {
        out.push(Finding {
            rule: "OA-007",
            severity: Severity::Fatal,
            message: "structured_legal_attestation claimed without content/source anchoring"
                .to_string(),
            sibyl_id: Some(sibyl.sibyl_id.clone()),
        });
    }

    let regime = sibyl.attestation_scope.verification_regime;
    let emitted_confidence = cap_confidence(sibyl.attestation_scope.max_confidence, regime);
    if emitted_confidence == Confidence::B {
        match record.verify_signature_did(resolver) {
            Ok(outcome) if outcome.is_valid() => {}
            Ok(outcome) => out.push(Finding {
                rule: "OA-014",
                severity: Severity::Fatal,
                message: format!(
                    "Confidence B requires a cryptographically valid LBRY signature, got {outcome:?}"
                ),
                sibyl_id: Some(sibyl.sibyl_id.clone()),
            }),
            Err(e) => out.push(Finding {
                rule: "OA-014",
                severity: Severity::Fatal,
                message: format!(
                    "Confidence B requires a resolvable, cryptographically valid LBRY signature: {e}"
                ),
                sibyl_id: Some(sibyl.sibyl_id.clone()),
            }),
        }
    }

    out
}

/// Map a validated LBRY record + its Sibyl into the `arrow-origin-leg` boundary object.
///
/// Confidence and verification regime come from the **Sibyl** (the adapter derives them from
/// record quality, capped by the Sibyl's ceiling); the legal fact comes from the LBRY record.
pub fn to_legal_attestation(record: &LbryRecord, sibyl: &SibylEntry) -> LegalAttestation {
    let fact = &record.structured_fact;
    let regime = sibyl.attestation_scope.verification_regime;
    // Honest confidence (OA-007): never claim more than the regime supports. Capped to Arrow's
    // *live* ceiling; B is a ceiling, not a default.
    let emitted_confidence = cap_confidence(sibyl.attestation_scope.max_confidence, regime);
    LegalAttestation {
        leg_subtype: leg_subtype_str(fact.leg_subtype).to_string(),
        country: fact.leg_jurisdiction.country.clone(),
        supra: fact.leg_jurisdiction.supra.clone(),
        reason_code: fact.reason_code.clone(),
        finality: fact.finality,
        confidence: confidence_str(emitted_confidence).to_string(),
        class: 2, // INSTRUCT default; Class taxonomy refinement tracked (OI-003, LEG-007).
        state: None, // arrow-origin-leg derives the State from finality.
        asserted_by: record.attested_by.clone(),
        case_ref: fact.instrument_ref.clone(),
        object_id: record.lbry_id.clone(),
        proof_type: None,
        proof_ref: Some(format!("lbry:{}", record.lbry_id)),
        verification_regime: arrow_regime_str(sibyl.attestation_scope.verification_regime)
            .to_string(),
        finality_basis: None, // arrow defaults `legal_appeal` when finality = 0 (FB-001).
        description: Some(fact.operative_claim.clone()),
    }
}

/// Run the producer gate and, if it passes, mint a protocol-valid LEG [`ArrowEvent`].
pub fn emit_leg_event(
    record: &LbryRecord,
    registry: &SibylRegistry,
    now: &str,
) -> Result<ArrowEvent, EmissionBlocked> {
    emit_leg_event_with_resolver(record, registry, now, &MultiResolver::default())
}

/// Run the producer gate with an explicit DID resolver and mint a protocol-valid LEG event.
pub fn emit_leg_event_with_resolver<R: DidResolver>(
    record: &LbryRecord,
    registry: &SibylRegistry,
    now: &str,
    resolver: &R,
) -> Result<ArrowEvent, EmissionBlocked> {
    let findings = preflight_with_resolver(record, registry, now, resolver);
    if has_fatal(&findings) {
        return Err(EmissionBlocked {
            findings: findings
                .into_iter()
                .filter(|f| f.severity == Severity::Fatal)
                .collect(),
        });
    }
    // Preflight guarantees the Sibyl is present and active.
    let sibyl = registry
        .active_by_did(&record.attested_by)
        .expect("preflight guarantees an active Sibyl");
    let attestation = to_legal_attestation(record, sibyl);
    Ok(map_to_arrow(&attestation))
}

// ── enum → Arrow string mappings (the integration seam) ───────────────────────

fn leg_subtype_str(s: LegSubtype) -> &'static str {
    match s {
        LegSubtype::Jud => "JUD",
        LegSubtype::Adm => "ADM",
        LegSubtype::Supra => "SUPRA",
        LegSubtype::LegText => "LEG_TEXT",
        LegSubtype::Prec => "PREC",
    }
}

fn confidence_str(c: Confidence) -> &'static str {
    match c {
        Confidence::A => "A",
        Confidence::B => "B",
        Confidence::C => "C",
        Confidence::D => "D",
        Confidence::E => "E",
    }
}

/// Map the Tarquin regime to the regime string Arrow expects. NOTE: Tarquin's
/// `institutional_attestation` maps to Arrow's `legal_attestation` (C ceiling). Reconciling the
/// two names across the stack is an open item.
fn arrow_regime_str(r: VerificationRegime) -> &'static str {
    match r {
        VerificationRegime::StructuredLegalAttestation => "structured_legal_attestation",
        VerificationRegime::MultipleLegalAttestation => "multiple_legal_attestation",
        VerificationRegime::InstitutionalAttestation => "legal_attestation",
    }
}

/// Arrow's legal-regime ceilings, per `registries/confidence_subprofiles.json`. Structured and
/// multiple legal attestations may reach B if their artifacts and accountability posture justify
/// it. Manual/institutional fallback remains capped at C.
fn arrow_confidence_ceiling(r: VerificationRegime) -> Confidence {
    match r {
        VerificationRegime::StructuredLegalAttestation => Confidence::B,
        VerificationRegime::MultipleLegalAttestation => Confidence::B,
        VerificationRegime::InstitutionalAttestation => Confidence::C,
    }
}

/// The weaker (lower-rank) of the Sibyl's declared ceiling and Arrow's regime ceiling.
fn cap_confidence(declared: Confidence, regime: VerificationRegime) -> Confidence {
    let ceiling = arrow_confidence_ceiling(regime);
    if declared.rank() > ceiling.rank() {
        ceiling
    } else {
        declared
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_identity::crypto::ed25519::generate_test_keypair;
    use arrow_identity::resolver::did_key::did_key_from_public;
    use tarquin_sibyl::pipeline::{attest, AttestParams};
    use tarquin_sibyl::registry::{Confidence, SibylStatus};

    const DOCKET: &str = include_str!("../../../tests/fixtures/sibyl/docket-mx-2026-001.json");
    const EXAMPLE: &str = include_str!("../../../drafts/sibyl/sibyl-registry.example.json");

    fn b_registry(did: String, key_ref: String) -> SibylRegistry {
        let mut reg = SibylRegistry::from_json(EXAMPLE).unwrap();
        let entry = &mut reg.entries[0];
        entry.did = did;
        entry.keys[0].key_id = key_ref;
        entry.status = SibylStatus::Active;
        entry.attestation_scope.max_confidence = Confidence::B;
        reg
    }

    fn signed_b_record() -> (LbryRecord, SibylRegistry) {
        let (private, public) = generate_test_keypair();
        let did = did_key_from_public(&public);
        let frag = did.strip_prefix("did:key:").unwrap();
        let key_ref = format!("{did}#{frag}");
        let params = AttestParams {
            sibyl_did: did.clone(),
            key_ref: key_ref.clone(),
            attested_at: "2026-06-08T00:00:00Z".to_string(),
            lbry_id: Some("lbry-b-confidence-test".to_string()),
        };
        let record = attest(DOCKET.as_bytes(), &params, &private).unwrap();
        (record, b_registry(did, key_ref))
    }

    #[test]
    fn b_confidence_requires_cryptographically_valid_lbry_signature() {
        let (mut record, registry) = signed_b_record();
        record.signature.sig_b64 = "AA".to_string();

        let findings = preflight(&record, &registry, "2026-06-09T00:00:00Z");

        assert!(
            findings
                .iter()
                .any(|f| f.rule == "OA-014" && f.severity == Severity::Fatal),
            "expected OA-014 fatal finding, got {findings:?}"
        );
    }

    #[test]
    fn b_confidence_emits_when_lbry_signature_verifies() {
        let (record, registry) = signed_b_record();

        let event = emit_leg_event(&record, &registry, "2026-06-09T00:00:00Z").unwrap();

        assert!(event.arrow_code.contains(":B"));
    }
}
