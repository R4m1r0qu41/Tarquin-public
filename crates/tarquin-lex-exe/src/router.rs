//! Phase 4.5 — the consumer **execution router** (`leg-consumer.md` §6).
//!
//! Routes a *ratified* disposition to the primitive that may enforce it. The router decides and
//! instructs; it performs no effect itself — the actual execution is the deployment's (and, on
//! chain, the `LegConsumerGate`'s). Nothing here runs without a prior human ratification
//! (ADR-0001).
//!
//! Enforced rules:
//! - **LC-009 (Fatal)** — execution dispatches only to a primitive bound by `parity_ref` to its
//!   off-chain engine counterpart, and only if that binding is parity-certified. The router is
//!   constructed against a parity certificate (the ruleset hash the harness ran green on); a
//!   ruleset with no passing parity run is structurally unreachable.
//! - **LC-010 (Fatal)** — single-shot execution per `arrow_event_id`; replays are rejected.
//! - **LC-003b** — the attesting Sibyl's active status is re-checked at ratification time, not
//!   just at emission.
//! - **LC-013 / LC-015 / LC-016** (`specs/consumer-lifecycle-scope-rules.md`) — before any
//!   dispatch, the lifecycle discipline runs over the caller-supplied [`LifecycleView`]:
//!   closed lifecycles are terminal, post-ratification reversals/supersessions send the item
//!   back to review, and an overdue-unclosed lifecycle is stale (and possibly under attack) —
//!   refused, never executed.

use std::collections::HashSet;

use crate::lifecycle::{check as lifecycle_check, LifecycleRefusal, LifecycleView};
use tarquin_engine::{evaluate, ruleset_hash, Action, Decision, Inputs, Rule};
use tarquin_sibyl::registry::SibylEntry;
use tarquin_sibyl::{LbryRecord, SibylRegistry};

/// Build the engine's typed [`Inputs`] from a validated LBRY record and its Sibyl entry.
///
/// The two gate facts the record cannot carry — whether the proof is instrument-specific
/// (OA-011 cond 2) and whether Lex.Codex `permits()` authorized `PR` (OA-011 cond 5) — are
/// explicit caller inputs, like time: the deployment supplies them, the router never infers.
pub fn inputs_from_lbry(
    record: &LbryRecord,
    sibyl: &SibylEntry,
    proof_instrument_specific: bool,
    lexcodex_permits_pr: bool,
) -> Inputs {
    Inputs {
        leg_subtype: record.structured_fact.leg_subtype,
        finality: record.structured_fact.finality,
        reason_code: record.structured_fact.reason_code.clone(),
        verification_regime: sibyl.attestation_scope.verification_regime,
        sibyl_registered_active: sibyl.is_active(),
        proof_instrument_specific,
        lexcodex_permits_pr,
    }
}

/// Router construction failed: the supplied ruleset is not parity-certified.
#[derive(Debug, thiserror::Error)]
#[error(
    "ruleset hash {actual} does not match the parity certificate {certified}; \
     an uncertified ruleset must not be reachable from the router (LC-009)"
)]
pub struct NotParityCertified {
    pub certified: String,
    pub actual: String,
}

/// The router's verdict for one ratification attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteOutcome {
    /// Dispatch to the bound primitive. The caller performs the effect (single-shot already
    /// recorded; a second call for the same event will be `Refused`).
    Execute {
        rule_id: String,
        /// The `parity_ref` binding the engine rule to its on-chain primitive (LC-009).
        parity_ref: String,
        decision: Decision,
    },
    /// The engine disposition is not `PR`: stays in the human-gated lane; nothing executes.
    Held { decision: Decision },
    /// A `PR` disposition was refused at the gate. Fail-closed: refusal, not escalation.
    Refused {
        reason: RefusalReason,
        decision: Decision,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalReason {
    /// No human ratification (LC-006 / consumer C-4 precedes C-5).
    NotRatified,
    /// Same `arrow_event_id` already executed (LC-010).
    ReplayRejected,
    /// Attesting Sibyl not active at ratification time (LC-003b).
    SibylNotActive,
    /// Lifecycle discipline refused (LC-013 / LC-015 / LC-016).
    Lifecycle(LifecycleRefusal),
}

/// The execution router. Holds the parity-certified ruleset and the single-shot record.
#[derive(Debug)]
pub struct ExecutionRouter {
    rules: Vec<Rule>,
    executed: HashSet<String>,
}

impl ExecutionRouter {
    /// Build a router over `rules`, refusing any ruleset whose hash differs from the parity
    /// certificate. The certificate is the `ruleset_sha256` printed by a green
    /// `tests/parity/run.sh` run (PR-003) and supplied by deployment configuration.
    pub fn new(rules: Vec<Rule>, parity_certificate: &str) -> Result<Self, NotParityCertified> {
        let actual = ruleset_hash(&rules);
        if actual != parity_certificate {
            return Err(NotParityCertified {
                certified: parity_certificate.to_string(),
                actual,
            });
        }
        Ok(ExecutionRouter {
            rules,
            executed: HashSet::new(),
        })
    }

    /// Route one disposition. `ratified` is the human decision on the held review item;
    /// `attested_by_did` + `registry` re-check the Sibyl at ratification time (LC-003b).
    ///
    /// Only a `PR` decision that is ratified, Sibyl-active, and not a replay yields
    /// [`RouteOutcome::Execute`]; everything else is `Held` or `Refused` (fail-closed).
    pub fn route(
        &mut self,
        arrow_event_id: &str,
        inputs: &Inputs,
        ratified: bool,
        attested_by_did: &str,
        registry: &SibylRegistry,
        lifecycle: &LifecycleView,
    ) -> RouteOutcome {
        let decision = evaluate(&self.rules, inputs);

        if decision.action != Action::Pr {
            return RouteOutcome::Held { decision };
        }
        // LC-013/015/016 run before every other execution gate: a closed,
        // superseded, or stale lifecycle refuses regardless of ratification.
        if let Some(refusal) = lifecycle_check(lifecycle) {
            return RouteOutcome::Refused {
                reason: RefusalReason::Lifecycle(refusal),
                decision,
            };
        }
        if !ratified {
            return RouteOutcome::Refused {
                reason: RefusalReason::NotRatified,
                decision,
            };
        }
        if registry.active_by_did(attested_by_did).is_none() {
            return RouteOutcome::Refused {
                reason: RefusalReason::SibylNotActive,
                decision,
            };
        }
        if self.executed.contains(arrow_event_id) {
            return RouteOutcome::Refused {
                reason: RefusalReason::ReplayRejected,
                decision,
            };
        }

        // The governing rule is in the certified ruleset by construction (the decision came
        // from evaluating it), so its parity_ref is a certified binding (LC-009).
        let parity_ref = self
            .rules
            .iter()
            .find(|r| r.id == decision.rule)
            .expect("governing rule is in the ruleset")
            .parity_ref
            .clone();

        self.executed.insert(arrow_event_id.to_string());
        RouteOutcome::Execute {
            rule_id: decision.rule.clone(),
            parity_ref,
            decision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tarquin_engine::canonical_ruleset;

    const NOW_UNIX: u64 = 1_800_000_000;
    use tarquin_sibyl::registry::SibylStatus;
    use tarquin_sibyl::SibylRegistry;

    const EXAMPLE: &str = include_str!("../../../drafts/sibyl/sibyl-registry.example.json");
    const SIBYL_DID: &str = "did:example:sibyl-mx-court-001";

    fn certified_router() -> ExecutionRouter {
        let rules = canonical_ruleset();
        let cert = ruleset_hash(&rules);
        ExecutionRouter::new(rules, &cert).unwrap()
    }

    fn active_registry() -> SibylRegistry {
        let mut reg = SibylRegistry::from_json(EXAMPLE).unwrap();
        reg.entries[0].status = SibylStatus::Active;
        reg
    }

    fn pr_inputs() -> Inputs {
        Inputs {
            leg_subtype: tarquin_sibyl::registry::LegSubtype::Jud,
            finality: 1,
            reason_code: "CO".to_string(),
            verification_regime:
                tarquin_sibyl::registry::VerificationRegime::StructuredLegalAttestation,
            sibyl_registered_active: true,
            proof_instrument_specific: true,
            lexcodex_permits_pr: true,
        }
    }

    #[test]
    fn uncertified_ruleset_is_unreachable() {
        let err = ExecutionRouter::new(canonical_ruleset(), "deadbeef").unwrap_err();
        assert_eq!(err.actual, ruleset_hash(&canonical_ruleset()));
    }

    #[test]
    fn pr_executes_only_after_ratification() {
        let mut router = certified_router();
        let reg = active_registry();

        let refused = router.route(
            "evt-1",
            &pr_inputs(),
            false,
            SIBYL_DID,
            &reg,
            &LifecycleView::empty(NOW_UNIX),
        );
        assert!(matches!(
            refused,
            RouteOutcome::Refused {
                reason: RefusalReason::NotRatified,
                ..
            }
        ));

        let executed = router.route(
            "evt-1",
            &pr_inputs(),
            true,
            SIBYL_DID,
            &reg,
            &LifecycleView::empty(NOW_UNIX),
        );
        match executed {
            RouteOutcome::Execute {
                rule_id,
                parity_ref,
                ..
            } => {
                assert_eq!(rule_id, "ENG-PR-ELEVATION");
                assert!(parity_ref.contains("escrow release"));
            }
            other => panic!("expected Execute, got {other:?}"),
        }
    }

    #[test]
    fn replay_is_rejected_single_shot() {
        let mut router = certified_router();
        let reg = active_registry();
        let first = router.route(
            "evt-replay",
            &pr_inputs(),
            true,
            SIBYL_DID,
            &reg,
            &LifecycleView::empty(NOW_UNIX),
        );
        assert!(matches!(first, RouteOutcome::Execute { .. }));
        let second = router.route(
            "evt-replay",
            &pr_inputs(),
            true,
            SIBYL_DID,
            &reg,
            &LifecycleView::empty(NOW_UNIX),
        );
        assert!(matches!(
            second,
            RouteOutcome::Refused {
                reason: RefusalReason::ReplayRejected,
                ..
            }
        ));
    }

    #[test]
    fn sibyl_revoked_at_ratification_refuses() {
        let mut router = certified_router();
        let mut reg = active_registry();
        reg.entries[0].status = SibylStatus::Suspended;
        let outcome = router.route(
            "evt-2",
            &pr_inputs(),
            true,
            SIBYL_DID,
            &reg,
            &LifecycleView::empty(NOW_UNIX),
        );
        assert!(matches!(
            outcome,
            RouteOutcome::Refused {
                reason: RefusalReason::SibylNotActive,
                ..
            }
        ));
    }

    #[test]
    fn non_pr_is_held_never_executed() {
        let mut router = certified_router();
        let reg = active_registry();
        let mut under_appeal = pr_inputs();
        under_appeal.finality = 0;
        // Ratified or not, a non-PR disposition stays in the human-gated lane.
        let outcome = router.route(
            "evt-3",
            &under_appeal,
            true,
            SIBYL_DID,
            &reg,
            &LifecycleView::empty(NOW_UNIX),
        );
        match outcome {
            RouteOutcome::Held { decision } => assert_eq!(decision.action, Action::Hl),
            other => panic!("expected Held, got {other:?}"),
        }
    }

    #[test]
    fn closed_lifecycle_refuses_even_ratified_pr() {
        use arrow_core::types::event::ArrowEvent;
        let mut router = certified_router();
        let reg = active_registry();
        let closure = ArrowEvent {
            arrow_code: "6:ETH:FN:CL:1:A".to_string(),
            event_id: "evt-closure".to_string(),
            lifecycle_id: "lfc-x".to_string(),
            timestamp: "2026-07-07T00:00:00Z".to_string(),
            object_type: "t".to_string(),
            object_id: "o".to_string(),
            ..Default::default()
        };
        let events = vec![closure];
        let view = LifecycleView {
            events: &events,
            ratified_at_len: 1,
            now_unix: NOW_UNIX,
        };
        // Fully ratified, Sibyl active, first execution — and still refused:
        // a closed lifecycle is terminal (LC-015).
        let outcome = router.route("evt-post-close", &pr_inputs(), true, SIBYL_DID, &reg, &view);
        assert!(matches!(
            outcome,
            RouteOutcome::Refused {
                reason: RefusalReason::Lifecycle(LifecycleRefusal::LifecycleClosed),
                ..
            }
        ));
    }

    #[test]
    fn forced_block_is_held_even_if_pr_gate_satisfied() {
        let mut router = certified_router();
        let reg = active_registry();
        let mut sanctions = pr_inputs();
        sanctions.leg_subtype = tarquin_sibyl::registry::LegSubtype::Supra;
        sanctions.reason_code = "SA".to_string();
        let outcome = router.route(
            "evt-4",
            &sanctions,
            true,
            SIBYL_DID,
            &reg,
            &LifecycleView::empty(NOW_UNIX),
        );
        match outcome {
            RouteOutcome::Held { decision } => {
                assert_eq!(decision.action, Action::Bl);
                assert!(decision.forced);
            }
            other => panic!("expected Held(BL), got {other:?}"),
        }
    }
}
