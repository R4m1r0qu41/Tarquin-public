//! The evaluator — total, deterministic, fail-closed.

use serde::{Deserialize, Serialize};

use crate::ir::{Action, Inputs, Rule};

/// The governing decision for an evaluation: the action, whether it was forced (non-overridable),
/// and the id of the rule that produced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub action: Action,
    pub forced: bool,
    pub rule: String,
}

/// Evaluate a ruleset against typed inputs. Semantics:
///
/// 1. A **forced** matching rule governs unconditionally (LEG-007 BL force-block).
/// 2. Otherwise the **most restrictive** matching rule governs ("more restrictive governs").
/// 3. If **no** rule matches, fail closed to `HL`.
///
/// Total (always returns), deterministic (no clock / RNG / I/O), and pure.
pub fn evaluate(rules: &[Rule], inputs: &Inputs) -> Decision {
    let matching: Vec<&Rule> = rules.iter().filter(|r| r.matches(inputs)).collect();

    // 1. Forced rules dominate. Among several, the most restrictive wins (deterministic tiebreak
    //    on rule id keeps output stable regardless of ruleset ordering).
    if let Some(r) = matching.iter().filter(|r| r.forced).max_by(|a, b| {
        a.yields
            .restrictiveness()
            .cmp(&b.yields.restrictiveness())
            .then_with(|| b.id.cmp(&a.id))
    }) {
        return Decision {
            action: r.yields,
            forced: true,
            rule: r.id.clone(),
        };
    }

    // 2. Most restrictive matching rule (stable tiebreak on id).
    match matching.iter().max_by(|a, b| {
        a.yields
            .restrictiveness()
            .cmp(&b.yields.restrictiveness())
            .then_with(|| b.id.cmp(&a.id))
    }) {
        Some(r) => Decision {
            action: r.yields,
            forced: false,
            rule: r.id.clone(),
        },
        // 3. Fail closed.
        None => Decision {
            action: Action::Hl,
            forced: false,
            rule: "ENG-DEFAULT-HL".to_string(),
        },
    }
}
