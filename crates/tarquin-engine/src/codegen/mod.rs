//! Backend code generation — the EN-O2/EN-O3 compilers from the Rule IR to the two targets.
//!
//! Both backends emit the ruleset in **governing order** (forced first, then restrictiveness
//! descending, then rule id ascending) as straight-line first-match-wins code. That collapse is
//! semantics-preserving: [`crate::eval::evaluate`] lets a matching forced rule dominate
//! unconditionally, otherwise picks the most restrictive matching rule with an id-ascending
//! tiebreak, and fails closed to `HL` — exactly what first-match over the pre-sorted sequence
//! returns. The generated artifacts are committed and re-checked by `tarquin-engine-gen --check`
//! (PR-003: divergence = build failure).

pub mod sol;
pub mod ts;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::eval::Decision;
use crate::ir::{Action, Inputs, Rule};
use tarquin_sibyl::registry::{LegSubtype, VerificationRegime};

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("duplicate rule id {0:?}")]
    DuplicateRuleId(String),
    #[error("rule id {0:?} exceeds 32 bytes (must fit a Solidity bytes32)")]
    RuleIdTooLong(String),
    #[error("reason code {0:?} exceeds 32 bytes (must fit a Solidity bytes32)")]
    ReasonCodeTooLong(String),
    #[error("rule {0:?} has an empty `when` (never matches; remove it from the ruleset)")]
    EmptyWhen(String),
    #[error("identifier {0:?} contains characters outside [A-Za-z0-9_-]")]
    BadIdentifier(String),
}

/// Validate the ruleset for cross-target encodability, then return the rules in the order in
/// which they govern. Generated targets emit them in this order, first match wins.
pub fn governing_order(rules: &[Rule]) -> Result<Vec<&Rule>, CodegenError> {
    for (i, r) in rules.iter().enumerate() {
        if rules[..i].iter().any(|p| p.id == r.id) {
            return Err(CodegenError::DuplicateRuleId(r.id.clone()));
        }
        if r.id.len() > 32 {
            return Err(CodegenError::RuleIdTooLong(r.id.clone()));
        }
        check_ident(&r.id)?;
        if r.when.is_empty() {
            return Err(CodegenError::EmptyWhen(r.id.clone()));
        }
        for c in &r.when {
            if let crate::ir::Condition::ReasonCodeEquals(rc) = c {
                if rc.len() > 32 {
                    return Err(CodegenError::ReasonCodeTooLong(rc.clone()));
                }
                check_ident(rc)?;
            }
        }
    }
    let mut ordered: Vec<&Rule> = rules.iter().collect();
    ordered.sort_by(|a, b| {
        b.forced
            .cmp(&a.forced)
            .then_with(|| b.yields.restrictiveness().cmp(&a.yields.restrictiveness()))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(ordered)
}

fn check_ident(s: &str) -> Result<(), CodegenError> {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err(CodegenError::BadIdentifier(s.to_string()))
    }
}

/// sha256 over the ruleset's canonical JSON — stamped into every generated artifact so the
/// parity harness can prove all targets were built from the same IR.
pub fn ruleset_hash(rules: &[Rule]) -> String {
    let json = serde_json::to_string(rules).expect("ruleset serializes");
    let digest = Sha256::digest(json.as_bytes());
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ── Parity corpus (EN-O4 / PR-003) ─────────────────────────────────────────────

/// One golden corpus case: typed inputs and the disposition every target must return.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusCase {
    pub name: String,
    pub inputs: Inputs,
    pub expected: Decision,
}

/// Parse the golden corpus from its JSON representation.
pub fn parse_corpus(s: &str) -> Result<Vec<CorpusCase>, serde_json::Error> {
    serde_json::from_str(s)
}

// ── Shared enum string mappings (kept here so both backends agree) ─────────────

pub(crate) fn subtype_str(s: LegSubtype) -> &'static str {
    match s {
        LegSubtype::Jud => "JUD",
        LegSubtype::Adm => "ADM",
        LegSubtype::Supra => "SUPRA",
        LegSubtype::LegText => "LEG_TEXT",
        LegSubtype::Prec => "PREC",
    }
}

pub(crate) fn regime_screaming(r: VerificationRegime) -> &'static str {
    match r {
        VerificationRegime::StructuredLegalAttestation => "STRUCTURED_LEGAL_ATTESTATION",
        VerificationRegime::MultipleLegalAttestation => "MULTIPLE_LEGAL_ATTESTATION",
        VerificationRegime::InstitutionalAttestation => "INSTITUTIONAL_ATTESTATION",
    }
}

pub(crate) fn regime_snake(r: VerificationRegime) -> &'static str {
    match r {
        VerificationRegime::StructuredLegalAttestation => "structured_legal_attestation",
        VerificationRegime::MultipleLegalAttestation => "multiple_legal_attestation",
        VerificationRegime::InstitutionalAttestation => "institutional_attestation",
    }
}

pub(crate) fn action_str(p: Action) -> &'static str {
    match p {
        Action::Bl => "BL",
        Action::Hl => "HL",
        Action::Rw => "RW",
        Action::Tr => "TR",
        Action::Pr => "PR",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Condition;
    use crate::rules::canonical_ruleset;

    #[test]
    fn governing_order_puts_forced_first_then_restrictive_then_id() {
        let rules = canonical_ruleset();
        let ordered = governing_order(&rules).unwrap();
        assert_eq!(ordered[0].id, "ENG-SUPRA-SA-BL"); // forced
        assert_eq!(ordered[1].id, "ENG-PR-ELEVATION");
    }

    /// The collapse claim: first-match over `governing_order` equals `evaluate` on every corpus
    /// of inputs we can enumerate from the canonical conditions (exhaustive over the bools and
    /// the discriminants that appear in the ruleset).
    #[test]
    fn first_match_over_governing_order_equals_evaluate() {
        let rules = canonical_ruleset();
        let ordered = governing_order(&rules).unwrap();
        for inputs in enumerate_inputs() {
            let expect = crate::eval::evaluate(&rules, &inputs);
            let got = ordered
                .iter()
                .find(|r| r.matches(&inputs))
                .map(|r| Decision {
                    action: r.yields,
                    forced: r.forced,
                    rule: r.id.clone(),
                })
                .unwrap_or(Decision {
                    action: Action::Hl,
                    forced: false,
                    rule: "ENG-DEFAULT-HL".to_string(),
                });
            assert_eq!(expect, got, "divergence for {inputs:?}");
        }
    }

    fn enumerate_inputs() -> Vec<Inputs> {
        let mut out = Vec::new();
        for subtype in [LegSubtype::Jud, LegSubtype::Supra] {
            for finality in [0u8, 1] {
                for reason in ["SA", "CO"] {
                    for regime in [
                        VerificationRegime::StructuredLegalAttestation,
                        VerificationRegime::InstitutionalAttestation,
                    ] {
                        for bits in 0u8..8 {
                            out.push(Inputs {
                                leg_subtype: subtype,
                                finality,
                                reason_code: reason.to_string(),
                                verification_regime: regime,
                                sibyl_registered_active: bits & 1 != 0,
                                proof_instrument_specific: bits & 2 != 0,
                                lexcodex_permits_pr: bits & 4 != 0,
                            });
                        }
                    }
                }
            }
        }
        out
    }

    #[test]
    fn rejects_unencodable_rulesets() {
        let mut rules = canonical_ruleset();
        rules.push(rules[0].clone());
        assert!(matches!(
            governing_order(&rules),
            Err(CodegenError::DuplicateRuleId(_))
        ));

        let empty = vec![Rule {
            id: "ENG-EMPTY".to_string(),
            parity_ref: "x".to_string(),
            when: vec![],
            yields: Action::Hl,
            forced: false,
        }];
        assert!(matches!(
            governing_order(&empty),
            Err(CodegenError::EmptyWhen(_))
        ));

        let long = vec![Rule {
            id: "X".repeat(33),
            parity_ref: "x".to_string(),
            when: vec![Condition::FinalityEquals(1)],
            yields: Action::Hl,
            forced: false,
        }];
        assert!(matches!(
            governing_order(&long),
            Err(CodegenError::RuleIdTooLong(_))
        ));
    }

    #[test]
    fn ruleset_hash_is_stable_and_input_sensitive() {
        let rules = canonical_ruleset();
        assert_eq!(ruleset_hash(&rules), ruleset_hash(&canonical_ruleset()));
        let mut altered = canonical_ruleset();
        altered[0].forced = false;
        assert_ne!(ruleset_hash(&rules), ruleset_hash(&altered));
    }
}
