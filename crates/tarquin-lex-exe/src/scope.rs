//! LC-014 — TR routing scope (`specs/consumer-lifecycle-scope-rules.md`;
//! Arrow RFC-010 Part 4, ratified split: the Arrow gate checks the
//! `action_target` envelope's SHAPE (TRG-001); this module owns RESOLUTION).
//!
//! A validated TR event never executes directly (LC-002). The consumer
//! resolves `action_target.target_ref` against its deployment-configured
//! scope table and routes to the named workflow; anything unresolvable or
//! out of scope becomes a hold + review item. Deterministic: the table is
//! explicit configuration, resolution is a map lookup, no inference.

use std::collections::BTreeMap;

use arrow_core::types::event::ActionTarget;

/// One registered workflow a TR event may route to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredWorkflow {
    /// The workflow's identifier presented back to the deployment.
    pub workflow_id: String,
    /// The parity binding of the workflow's enforcement primitive (LC-009
    /// discipline carries over: a workflow whose primitive has no certified
    /// parity binding must not be registered).
    pub parity_ref: String,
}

/// Deployment-configured TR scope table: `target_ref` → workflow.
/// BTreeMap for deterministic iteration/serialization order.
#[derive(Debug, Clone, Default)]
pub struct TrScopeTable {
    workflows: BTreeMap<String, RegisteredWorkflow>,
}

/// LC-014 outcome. There is deliberately no `Execute` variant: TR routes or
/// holds; the routed workflow carries its own full gate, including human
/// ratification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrRoute {
    /// Route to the registered workflow. The workflow's own gates now apply.
    Route(RegisteredWorkflow),
    /// Hold + review item; never a proceed (fail-closed).
    Hold(TrHoldReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrHoldReason {
    /// `target_ref` is not in this consumer's scope table.
    TargetUnresolvable,
}

impl TrScopeTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a workflow target. Deployment-time configuration only.
    pub fn register(&mut self, target_ref: impl Into<String>, workflow: RegisteredWorkflow) {
        self.workflows.insert(target_ref.into(), workflow);
    }

    /// LC-014: resolve a TRG-001-valid envelope to a workflow, or hold.
    ///
    /// The envelope's shape (non-empty refs, authorizing_events ⊆
    /// related_events) is the Arrow gate's guarantee; callers pass only
    /// events that already validated. Resolution failure is a HOLD, never an
    /// error and never a proceed.
    pub fn resolve(&self, target: &ActionTarget) -> TrRoute {
        match self.workflows.get(&target.target_ref) {
            Some(wf) => TrRoute::Route(wf.clone()),
            None => TrRoute::Hold(TrHoldReason::TargetUnresolvable),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.workflows.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(target: &str) -> ActionTarget {
        ActionTarget {
            target_ref: target.to_string(),
            object_ref: "asset:x".to_string(),
            authorizing_events: vec!["evt-auth-1".to_string()],
        }
    }

    #[test]
    fn registered_target_routes_to_workflow() {
        let mut table = TrScopeTable::new();
        table.register(
            "workflow:escrow-release-7",
            RegisteredWorkflow {
                workflow_id: "escrow-release-7".to_string(),
                parity_ref: "primitives/src/generated/RuleEngine.sol#escrow".to_string(),
            },
        );
        match table.resolve(&envelope("workflow:escrow-release-7")) {
            TrRoute::Route(wf) => assert_eq!(wf.workflow_id, "escrow-release-7"),
            other => panic!("expected Route, got {other:?}"),
        }
    }

    #[test]
    fn unresolvable_target_holds_never_proceeds() {
        let table = TrScopeTable::new(); // empty scope: trusts no target
        assert_eq!(
            table.resolve(&envelope("workflow:unknown")),
            TrRoute::Hold(TrHoldReason::TargetUnresolvable)
        );
    }
}
