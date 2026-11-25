//! Phase 4.5 exit criterion — the end-to-end `PR` path: a final, fully-gated legal fact travels
//! Sibyl → LBRY → emit → signed envelope → fail-closed consumer (**held**, nothing executes),
//! and only after a **human ratifies** does the execution router dispatch to the
//! parity-certified primitive — exactly once.
//!
//! Real on-chain `PR` remains upstream-gated on Arrow's LEG-007 derivation table; this test
//! proves the off-chain routing discipline against the parity-certified ruleset.

use std::io::Write as _;
use tarquin_lex_exe::LifecycleView;

const NOW_UNIX: u64 = 1_800_000_000;

use arrow_adapter::ConsumerExecutionMode;
use arrow_core::types::event::LegJurisdiction;
use arrow_core::types::message::Action as ArrowAction;
use arrow_identity::crypto::ed25519::generate_test_keypair;
use arrow_identity::resolver::did_key::did_key_from_public;
use arrow_origin_leg::build_signed_envelope;
use base64::Engine as _;

use tarquin_engine::{canonical_ruleset, ruleset_hash};
use tarquin_lex_exe::consume::{InMemoryReplayStore, InMemoryReviewQueue, LegReviewDecision};
use tarquin_lex_exe::{
    consume_envelope, emit_leg_event, inputs_from_lbry, ConsumeConfig, ExecutionRouter,
    RefusalReason, RouteOutcome,
};
use tarquin_sibyl::lbry::{LbryRecord, LbrySignature, StructuredFact};
use tarquin_sibyl::registry::{LegSubtype, SibylStatus};
use tarquin_sibyl::SibylRegistry;

const EXAMPLE: &str = include_str!("../../../drafts/sibyl/sibyl-registry.example.json");
const NOW: &str = "2026-06-09T00:00:00Z";

fn active_registry() -> SibylRegistry {
    let mut reg = SibylRegistry::from_json(EXAMPLE).unwrap();
    reg.entries[0].status = SibylStatus::Active;
    reg
}

/// A final (Finality=1) judicial authorization — the PR-eligible legal fact.
fn final_record() -> LbryRecord {
    let mut rec = LbryRecord {
        lbry_id: "lbry-e2e-pr-001".into(),
        schema_version: "0.1".into(),
        attested_by: "did:example:sibyl-mx-court-001".into(),
        attested_at: "2026-06-08T00:00:00Z".into(),
        source_ref: "docket://mx/example/2026/final".into(),
        source_hash: "sha256:bb".into(),
        structured_fact: StructuredFact {
            instrument_ref: "court-order-2026-final".into(),
            leg_subtype: LegSubtype::Jud,
            leg_jurisdiction: LegJurisdiction {
                country: Some("MX".into()),
                supra: None,
            },
            legal_system_refs: vec!["LS-MX-STATE-01".into()],
            effective_at: "2026-06-08T00:00:00Z".into(),
            finality: 1,
            reason_code: "CO".into(),
            operative_claim: "explicit_authorization".into(),
            policy_claim: ArrowAction("PR".into()),
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
fn pr_path_executes_only_after_human_ratification() {
    let registry = active_registry();
    let record = final_record();

    // 1. Lex.exe emits through the OA-* gate and signs the transport envelope.
    let event = emit_leg_event(&record, &registry, NOW).expect("emission");
    let arrow_event_id = event.object_id.clone();

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
    let envelope = build_signed_envelope(event, &did, receiver, Some(kf.path())).expect("sign");

    // 2. The fail-closed consumer HOLDS — even a final, PR-eligible fact never auto-executes.
    let cfg = ConsumeConfig {
        allowed_senders: vec![did],
        receiver_id: receiver.into(),
    };
    let replay = InMemoryReplayStore::new(chrono::Duration::hours(24));
    let queue = InMemoryReviewQueue::new();
    let outcome = consume_envelope(
        &envelope,
        &cfg,
        &replay,
        &queue,
        ConsumerExecutionMode::Execute,
    )
    .expect("consume");
    assert_eq!(outcome.decision, LegReviewDecision::HoldForReview);
    assert!(outcome.queued, "review item must be enqueued for a human");

    // 3. The execution router, built against the parity certificate (PR-003).
    let rules = canonical_ruleset();
    let certificate = ruleset_hash(&rules);
    let mut router = ExecutionRouter::new(rules, &certificate).expect("certified ruleset");

    let sibyl = registry.active_by_did(&record.attested_by).unwrap();
    // Gate facts supplied explicitly by the deployment (OA-011 conds 2 and 5).
    let inputs = inputs_from_lbry(&record, sibyl, true, true);

    // 3a. Before ratification: refused.
    let before = router.route(
        &arrow_event_id,
        &inputs,
        false,
        &record.attested_by,
        &registry,
        &LifecycleView::empty(NOW_UNIX),
    );
    assert!(matches!(
        before,
        RouteOutcome::Refused {
            reason: RefusalReason::NotRatified,
            ..
        }
    ));

    // 3b. Human ratifies: dispatch to the parity-bound escrow-release primitive.
    let after = router.route(
        &arrow_event_id,
        &inputs,
        true,
        &record.attested_by,
        &registry,
        &LifecycleView::empty(NOW_UNIX),
    );
    match &after {
        RouteOutcome::Execute {
            rule_id,
            parity_ref,
            ..
        } => {
            assert_eq!(rule_id, "ENG-PR-ELEVATION");
            assert!(
                parity_ref.contains("escrow release"),
                "LC-009 binding: {parity_ref}"
            );
        }
        other => panic!("expected Execute after ratification, got {other:?}"),
    }

    // 3c. Single-shot (LC-010): the same event can never execute twice.
    let replayed = router.route(
        &arrow_event_id,
        &inputs,
        true,
        &record.attested_by,
        &registry,
        &LifecycleView::empty(NOW_UNIX),
    );
    assert!(matches!(
        replayed,
        RouteOutcome::Refused {
            reason: RefusalReason::ReplayRejected,
            ..
        }
    ));
}
