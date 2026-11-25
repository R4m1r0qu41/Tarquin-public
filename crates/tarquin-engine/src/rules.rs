//! The canonical v0.1 ruleset — the two rules the stack already specifies. Each `parity_ref`
//! names the on-chain `primitives/` counterpart the parity harness (PR-003) will bind.

use crate::ir::{Action, Condition, Rule};
use tarquin_sibyl::registry::{LegSubtype, VerificationRegime};

/// The v0.1 ruleset. Deliberately minimal; grows only when a real SLC needs a construct that has
/// a proven cross-target (Solidity + TS) encoding (`tarquin-native-engine.md` §3).
pub fn canonical_ruleset() -> Vec<Rule> {
    vec![
        // LEG-007 / OA-010 — supranational sanctions force-block, non-overridable.
        Rule {
            id: "ENG-SUPRA-SA-BL".to_string(),
            parity_ref: "primitives: sanctions force-block (LEG-007)".to_string(),
            when: vec![
                Condition::SubtypeEquals(LegSubtype::Supra),
                Condition::FinalityEquals(1),
                Condition::ReasonCodeEquals("SA".to_string()),
            ],
            yields: Action::Bl,
            forced: true,
        },
        // OA-011 / LC-006 — conjunctive PR gate. PR only when every condition holds.
        Rule {
            id: "ENG-PR-ELEVATION".to_string(),
            parity_ref: "primitives: PR-gated escrow release (OA-011/LC-006)".to_string(),
            when: vec![
                Condition::RegimeEquals(VerificationRegime::StructuredLegalAttestation),
                Condition::ProofInstrumentSpecific,
                Condition::SibylRegisteredActive,
                Condition::FinalityEquals(1),
                Condition::LexCodexPermitsPr,
            ],
            yields: Action::Pr,
            forced: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::evaluate;
    use crate::ir::Inputs;

    /// Inputs that satisfy the full PR gate.
    fn pr_ready() -> Inputs {
        Inputs {
            leg_subtype: LegSubtype::Jud,
            finality: 1,
            reason_code: "CO".to_string(),
            verification_regime: VerificationRegime::StructuredLegalAttestation,
            sibyl_registered_active: true,
            proof_instrument_specific: true,
            lexcodex_permits_pr: true,
        }
    }

    #[test]
    fn full_gate_yields_pr() {
        let d = evaluate(&canonical_ruleset(), &pr_ready());
        assert_eq!(d.action, Action::Pr);
        assert!(!d.forced);
        assert_eq!(d.rule, "ENG-PR-ELEVATION");
    }

    #[test]
    fn any_missing_condition_fails_closed_to_hl() {
        // Drop each PR condition in turn; every variant must fall to HL.
        let mutate: [fn(&mut Inputs); 4] = [
            |i| i.finality = 0,
            |i| i.proof_instrument_specific = false,
            |i| i.sibyl_registered_active = false,
            |i| i.lexcodex_permits_pr = false,
        ];
        for m in mutate {
            let mut i = pr_ready();
            m(&mut i);
            let d = evaluate(&canonical_ruleset(), &i);
            assert_eq!(d.action, Action::Hl, "expected HL for {i:?}");
        }
    }

    #[test]
    fn supra_sanctions_force_bl() {
        let i = Inputs {
            leg_subtype: LegSubtype::Supra,
            finality: 1,
            reason_code: "SA".to_string(),
            ..pr_ready()
        };
        let d = evaluate(&canonical_ruleset(), &i);
        assert_eq!(d.action, Action::Bl);
        assert!(d.forced, "sanctions block must be non-overridable");
        assert_eq!(d.rule, "ENG-SUPRA-SA-BL");
    }

    #[test]
    fn forced_bl_overrides_a_satisfied_pr_gate() {
        // Inputs that satisfy BOTH the PR gate and the sanctions force-block: BL must win.
        let i = Inputs {
            leg_subtype: LegSubtype::Supra,
            finality: 1,
            reason_code: "SA".to_string(),
            verification_regime: VerificationRegime::StructuredLegalAttestation,
            sibyl_registered_active: true,
            proof_instrument_specific: true,
            lexcodex_permits_pr: true,
        };
        let d = evaluate(&canonical_ruleset(), &i);
        assert_eq!(d.action, Action::Bl);
        assert!(d.forced);
    }

    #[test]
    fn no_rule_matches_defaults_hl() {
        let i = Inputs {
            leg_subtype: LegSubtype::Jud,
            finality: 0, // not PR-eligible; not SUPRA+SA
            ..pr_ready()
        };
        let d = evaluate(&canonical_ruleset(), &i);
        assert_eq!(d.action, Action::Hl);
        assert_eq!(d.rule, "ENG-DEFAULT-HL");
    }

    #[test]
    fn evaluation_is_deterministic() {
        let i = pr_ready();
        assert_eq!(
            evaluate(&canonical_ruleset(), &i),
            evaluate(&canonical_ruleset(), &i)
        );
    }

    /// Parity with the on-chain `LegalHold` primitive: a hold/restriction NEVER yields PR.
    /// `LegalHold.permits()` returns false for any PR attempt; the engine's restriction path
    /// (anything that isn't the full PR gate) likewise never yields PR.
    #[test]
    fn parity_restriction_never_yields_pr() {
        let restriction_scenarios = [
            Inputs {
                finality: 0,
                ..pr_ready()
            }, // under appeal
            Inputs {
                leg_subtype: LegSubtype::Supra,
                reason_code: "SA".to_string(),
                ..pr_ready()
            }, // sanctions
            Inputs {
                lexcodex_permits_pr: false,
                ..pr_ready()
            }, // Lex.Codex veto
        ];
        for i in restriction_scenarios {
            assert_ne!(evaluate(&canonical_ruleset(), &i).action, Action::Pr);
        }
    }
}
