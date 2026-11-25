//! The Rule IR types — small enough to compile faithfully to both Solidity and TS.

use serde::{Deserialize, Serialize};

use tarquin_sibyl::registry::{LegSubtype, VerificationRegime};

/// Action lattice, mirroring `primitives/src/core/Types.sol` `Action`. Ordered by
/// restrictiveness for the "more restrictive governs" rule (adapter §1).
/// Serializes as the Arrow action strings (`BL`/`HL`/`RW`/`TR`/`PR`) — the IR's wire format,
/// shared by the parity corpus and the generated TS backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    /// Block.
    Bl,
    /// Hold for human review (fail-closed default).
    Hl,
    /// Rework / return.
    Rw,
    /// Transfer (compelled custody move).
    Tr,
    /// Proceed / execute (reachable only via the full conjunctive gate).
    Pr,
}

impl Action {
    /// Higher = more restrictive. Used to fold a ruleset to its governing outcome.
    pub fn restrictiveness(self) -> u8 {
        match self {
            Action::Bl => 5,
            Action::Hl => 4,
            Action::Rw => 3,
            Action::Tr => 2,
            Action::Pr => 1,
        }
    }
}

/// The typed legal facts + gate context a rule evaluates over. This is the IR's input type;
/// every field is an explicit input (determinism — no ambient state, no clock).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inputs {
    pub leg_subtype: LegSubtype,
    /// Arrow Finality: `1` final, `0` non-final / under appeal.
    pub finality: u8,
    /// Short legal reason code (`CO`, `SA`, `LR`, …).
    pub reason_code: String,
    pub verification_regime: VerificationRegime,
    /// The attesting Sibyl is registered and active (adapter OA-003 / OA-011 cond 3).
    pub sibyl_registered_active: bool,
    /// `proof_ref` carries attested, instrument-specific authorization (OA-011 cond 2).
    pub proof_instrument_specific: bool,
    /// Lex.Codex `permits()` explicitly authorized `PR` for this event (OA-011 cond 5).
    pub lexcodex_permits_pr: bool,
}

/// A pure predicate over [`Inputs`]. No I/O, no time, no inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    SubtypeEquals(LegSubtype),
    FinalityEquals(u8),
    ReasonCodeEquals(String),
    RegimeEquals(VerificationRegime),
    SibylRegisteredActive,
    ProofInstrumentSpecific,
    LexCodexPermitsPr,
}

impl Condition {
    /// Evaluate the predicate. Total and pure.
    pub fn holds(&self, i: &Inputs) -> bool {
        match self {
            Condition::SubtypeEquals(s) => i.leg_subtype == *s,
            Condition::FinalityEquals(f) => i.finality == *f,
            Condition::ReasonCodeEquals(r) => i.reason_code == *r,
            Condition::RegimeEquals(r) => i.verification_regime == *r,
            Condition::SibylRegisteredActive => i.sibyl_registered_active,
            Condition::ProofInstrumentSpecific => i.proof_instrument_specific,
            Condition::LexCodexPermitsPr => i.lexcodex_permits_pr,
        }
    }
}

/// A rule: a conjunction of conditions that, when all hold, yields a Action. A `forced` rule is
/// non-overridable (the LEG-007 sanctions force-block).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    /// Name of the on-chain counterpart in `primitives/` — the parity-harness binding (PR-003).
    pub parity_ref: String,
    /// All conditions must hold for the rule to yield (conjunction).
    pub when: Vec<Condition>,
    pub yields: Action,
    pub forced: bool,
}

impl Rule {
    /// True iff every condition holds (conjunction). A rule with no conditions never yields
    /// (guards against an accidentally unconditional rule).
    pub fn matches(&self, i: &Inputs) -> bool {
        !self.when.is_empty() && self.when.iter().all(|c| c.holds(i))
    }
}
