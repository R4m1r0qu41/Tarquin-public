//! Phase 3.4 — the LEG **consume** path.
//!
//! A thin driver over `arrow-consume-leg`'s fail-closed, human-in-the-loop review queue. It
//! builds the provenance policy from a [`ConsumeConfig`] and delegates to `verify_and_review`,
//! which re-verifies the envelope signature, re-validates the event, reduces it, and routes the
//! disposition to a human review queue. **Nothing executes autonomously** (ADR-0001).

use anyhow::Result;
use arrow_adapter::ConsumerExecutionMode;
use arrow_consume_leg::{
    verify_and_review, LegConsumeOutcome, ProvenancePolicy, ReplayStore, ReviewQueue,
    TransportEnvelope,
};

// Convenience re-exports so callers don't need to depend on the Arrow crates directly.
pub use arrow_consume_leg::{InMemoryReviewQueue, LegReviewDecision, LegReviewItem};
pub use arrow_transport::replay::InMemoryReplayStore;

/// Deployment configuration for the consumer's provenance gate.
#[derive(Debug, Clone)]
pub struct ConsumeConfig {
    /// DIDs the consumer accepts as emitters (the Lex.exe transport identities). Empty = none.
    pub allowed_senders: Vec<String>,
    /// This consumer's DID; the envelope's `receiver_id` must include it.
    pub receiver_id: String,
}

/// Verify a signed LEG envelope and route its disposition to the human review queue.
///
/// `mode` selects side effects: `DryRun` computes the decision without recording replay state
/// or enqueuing; `RecordOnly`/`Execute` record idempotency and enqueue a held item (the queue
/// *is* the action — there is no further autonomous step).
pub fn consume_envelope(
    envelope: &TransportEnvelope,
    cfg: &ConsumeConfig,
    replay: &dyn ReplayStore,
    queue: &dyn ReviewQueue,
    mode: ConsumerExecutionMode,
) -> Result<LegConsumeOutcome> {
    let policy = ProvenancePolicy::new(cfg.allowed_senders.clone(), cfg.receiver_id.clone());
    verify_and_review(envelope, &policy, replay, queue, mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::emit_leg_event;

    use arrow_core::types::event::LegJurisdiction;
    use arrow_core::types::message::Action;
    use arrow_identity::crypto::ed25519::generate_test_keypair;
    use arrow_identity::resolver::did_key::did_key_from_public;
    use arrow_origin_leg::build_signed_envelope;
    use base64::Engine as _;
    use chrono::Duration;
    use std::io::Write;

    use tarquin_sibyl::lbry::{LbryRecord, LbrySignature, StructuredFact};
    use tarquin_sibyl::registry::{LegSubtype, SibylStatus};
    use tarquin_sibyl::SibylRegistry;

    const EXAMPLE: &str = include_str!("../../../drafts/sibyl/sibyl-registry.example.json");

    fn active_registry() -> SibylRegistry {
        let mut reg = SibylRegistry::from_json(EXAMPLE).unwrap();
        reg.entries[0].status = SibylStatus::Active;
        reg
    }

    fn sample_record() -> LbryRecord {
        let mut rec = LbryRecord {
            lbry_id: "lbry-e2e-001".into(),
            schema_version: "0.1".into(),
            attested_by: "did:example:sibyl-mx-court-001".into(),
            attested_at: "2026-06-07T00:00:00Z".into(),
            source_ref: "docket://mx/example/2026".into(),
            source_hash: "sha256:aa".into(),
            structured_fact: StructuredFact {
                instrument_ref: "court-order-2026-example".into(),
                leg_subtype: LegSubtype::Jud,
                leg_jurisdiction: LegJurisdiction {
                    country: Some("MX".into()),
                    supra: None,
                },
                legal_system_refs: vec!["LS-MX-STATE-01".into()],
                effective_at: "2026-06-07T00:00:00Z".into(),
                finality: 0,
                reason_code: "CO".into(),
                operative_claim: "explicit_authorization".into(),
                policy_claim: Action("HL".into()),
                related_slc_artifacts: vec![],
            },
            content_hash: String::new(),
            signature: LbrySignature {
                key_ref: "did:example:sibyl-mx-court-001#attestation-1".into(),
                algorithm: "Ed25519".into(),
                sig_b64: "AA".into(),
            },
        };
        rec.content_hash = rec.compute_content_hash().unwrap();
        rec
    }

    #[test]
    fn emit_then_consume_holds_for_review() {
        let reg = active_registry();
        let rec = sample_record();

        // 3.3 — emit a protocol-valid LEG event through the producer gate.
        let event = emit_leg_event(&rec, &reg, "2026-06-08T00:00:00Z").expect("emission");
        assert!(event.arrow_code.contains(":LEG:"));
        assert!(event.arrow_code.contains(":HL:")); // JUD + Finality 0 -> HL

        // Sign with a fresh did:key — the Lex.exe transport identity (distinct from the Sibyl).
        let (private, public) = generate_test_keypair();
        let did = did_key_from_public(&public);
        let frag = did.strip_prefix("did:key:").unwrap();
        let key_ref = format!("{did}#{frag}");
        let key_json = format!(
            r#"{{"private_key_b64":"{}","key_ref":"{}","did":"{}"}}"#,
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(private),
            key_ref,
            did,
        );
        let mut kf = tempfile::NamedTempFile::new().unwrap();
        kf.write_all(key_json.as_bytes()).unwrap();

        let receiver = "did:web:consumer.tarquin.example";
        let envelope =
            build_signed_envelope(event, "unused", receiver, Some(kf.path())).expect("sign");

        // 3.4 — drive the fail-closed consumer. DryRun: decide without enqueue/replay effects.
        let cfg = ConsumeConfig {
            allowed_senders: vec![did.clone()],
            receiver_id: receiver.into(),
        };
        let replay = InMemoryReplayStore::new(Duration::hours(24));
        let queue = InMemoryReviewQueue::new();
        let outcome = consume_envelope(
            &envelope,
            &cfg,
            &replay,
            &queue,
            ConsumerExecutionMode::DryRun,
        )
        .expect("consume");

        // Fail-closed: a LEG disposition is held for a human; nothing executes.
        assert_eq!(outcome.decision, LegReviewDecision::HoldForReview);
        assert!(outcome.review_item.is_some());
    }

    #[test]
    fn consume_rejects_unknown_sender() {
        let reg = active_registry();
        let rec = sample_record();
        let event = emit_leg_event(&rec, &reg, "2026-06-08T00:00:00Z").unwrap();

        let (private, public) = generate_test_keypair();
        let did = did_key_from_public(&public);
        let frag = did.strip_prefix("did:key:").unwrap();
        let key_json = format!(
            r#"{{"private_key_b64":"{}","key_ref":"{did}#{frag}","did":"{did}"}}"#,
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(private),
        );
        let mut kf = tempfile::NamedTempFile::new().unwrap();
        kf.write_all(key_json.as_bytes()).unwrap();
        let receiver = "did:web:consumer.tarquin.example";
        let envelope =
            build_signed_envelope(event, "unused", receiver, Some(kf.path())).expect("sign");

        // Empty allow-list trusts no one — provenance must fail (fail-closed).
        let cfg = ConsumeConfig {
            allowed_senders: vec![],
            receiver_id: receiver.into(),
        };
        let replay = InMemoryReplayStore::new(Duration::hours(24));
        let queue = InMemoryReviewQueue::new();
        let result = consume_envelope(
            &envelope,
            &cfg,
            &replay,
            &queue,
            ConsumerExecutionMode::DryRun,
        );
        assert!(result.is_err(), "unknown sender must be rejected");
    }
}
