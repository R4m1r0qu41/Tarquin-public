//! Stage 2 — first cross-repo SLC handoff dry run (Orlog MC-006 → Tarquin).
//!
//! Proves the pinned-bundle discipline end to end against `slc-late-payment-001` at
//! Orlog commit `2826aea`:
//!
//! 1. The pinned bundle's MC-006 `artifact_hash` recomputes and matches both the bundle's own
//!    manifest and the pin — drift in either direction rejects the bundle.
//! 2. The Lex.Codex stub recomputes fixture case `disputed-001` deterministically and the
//!    result matches the sealed `expected` block (computation agreed; entitlement contested).
//! 3. **Stage A** — a `2:LEG:PD:HL:0:C` candidate carrying the sealed `related_slc_artifacts`
//!    reference emits through the OA-* gate and is **held** by the fail-closed consumer.
//! 4. **Negative twin** — the same facts claiming `PR` at Finality 0 are rejected by Arrow's
//!    SEM-013 (FN+0 executing-action ceiling).
//! 5. **Stage B** — the Finality 1 follow-up emits as `2:LEG:FN:PR:1:C` and is **still
//!    quarantined** pending human ratification; nothing executes in this test.
//!
//! Boundaries: Orlog authored the SLC; Tarquin verified a pinned copy. No live path
//! dependency, no Arrow events from Orlog, no SLC authoring here (ADR-0001 / MC-006).

use std::io::Write as _;
use std::path::{Path, PathBuf};

use arrow_adapter::ConsumerExecutionMode;
use arrow_core::registry::loader::Registries;
use arrow_core::types::event::{ArrowEvent, LegJurisdiction};
use arrow_core::types::message::Action as ArrowAction;
use arrow_core::validation::pipeline::ValidationPipeline;
use arrow_identity::crypto::ed25519::generate_test_keypair;
use arrow_identity::resolver::did_key::did_key_from_public;
use arrow_origin_leg::build_signed_envelope;
use base64::Engine as _;

use tarquin_lex_exe::consume::{InMemoryReplayStore, InMemoryReviewQueue, LegReviewDecision};
use tarquin_lex_exe::lexcodex::{build_artifact_ref, recompute_case, verify_pinned_bundle};
use tarquin_lex_exe::{consume_envelope, emit_leg_event, ConsumeConfig, LexCodexError};
use tarquin_sibyl::lbry::{LbryRecord, LbrySignature, SlcArtifactRef, StructuredFact};
use tarquin_sibyl::registry::LegSubtype;
use tarquin_sibyl::SibylRegistry;

const REGISTRY_JSON: &str = include_str!("../../../drafts/sibyl/sibyl-registry.usny.example.json");
const NOW: &str = "2026-06-11T00:00:00Z";

/// Pin facts for `slc-late-payment-001` (imports/orlog/PIN.json; Orlog 2826aea).
const PINNED_HASH: &str = "sha256:c0ac8c5ead20d8f07058164f7ab633c812494c408b3fdccc25300adf1e67b30e";
const FIXTURE_FILE: &str = "fixtures/disputed.json";
const FIXTURE_CASE: &str = "disputed-001";

fn bundle_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../imports/orlog/slc-late-payment-001")
}

fn usny_registry() -> SibylRegistry {
    SibylRegistry::from_json(REGISTRY_JSON).expect("US-NY example registry parses")
}

fn lbry_record(finality: u8, policy_claim: &str, slc_ref: SlcArtifactRef) -> LbryRecord {
    let mut rec = LbryRecord {
        lbry_id: format!("lbry-slc-handoff-f{finality}-001"),
        schema_version: "0.1".into(),
        attested_by: "did:example:sibyl-us-ny-court-001".into(),
        attested_at: "2026-06-11T00:00:00Z".into(),
        source_ref: "docket://us-ny/commercial/2026/late-payment-dispute-001".into(),
        source_hash: "sha256:aa".into(),
        structured_fact: StructuredFact {
            instrument_ref: "ny-commercial-order-2026-lp-001".into(),
            leg_subtype: LegSubtype::Jud,
            leg_jurisdiction: LegJurisdiction {
                country: Some("US".into()),
                supra: None,
            },
            legal_system_refs: vec!["LS-US-STATE-01".into()],
            effective_at: "2026-06-11T00:00:00Z".into(),
            finality,
            reason_code: "CO".into(),
            operative_claim: "late_payment_charge_determination".into(),
            policy_claim: ArrowAction(policy_claim.into()),
            related_slc_artifacts: vec![slc_ref],
        },
        content_hash: String::new(),
        signature: LbrySignature {
            key_ref: "did:example:sibyl-us-ny-court-001#attestation-1".into(),
            algorithm: "Ed25519".into(),
            sig_b64: "AA".into(),
        },
    };
    rec.content_hash = rec.compute_content_hash().unwrap();
    rec
}

/// Sign and consume an emitted event; return the consumer decision and queued flag.
fn consume(event: ArrowEvent) -> (LegReviewDecision, bool) {
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
    (outcome.decision, outcome.queued)
}

#[test]
fn pinned_bundle_hash_verifies_and_drift_rejects() {
    // Recomputed MC-006 hash matches the manifest and the pin.
    let hash = verify_pinned_bundle(&bundle_dir(), PINNED_HASH).expect("pinned bundle verifies");
    assert_eq!(hash, PINNED_HASH);

    // A wrong pin (simulated drift) rejects the bundle before any use.
    let wrong = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    match verify_pinned_bundle(&bundle_dir(), wrong) {
        Err(LexCodexError::PinMismatch { .. }) => {}
        other => panic!("expected PinMismatch on drifted pin, got {other:?}"),
    }
}

#[test]
fn lexcodex_recomputes_disputed_001_and_confirms_the_charge() {
    let recomp =
        recompute_case(&bundle_dir(), FIXTURE_FILE, FIXTURE_CASE).expect("recompute disputed-001");

    // The deterministic computation reproduces the sealed expectation exactly.
    assert_eq!(recomp.result.days_late, 60);
    assert_eq!(recomp.result.chargeable_days, 60);
    assert_eq!(recomp.result.late_charge, 5917.81);
    assert!(!recomp.result.capped);
    assert_eq!(recomp.result.total_due, 105917.81);
    assert_eq!(recomp.result.obligation, "LATE_CHARGE_DUE");

    // Computation agreed; entitlement contested — the dispute block mandates human review,
    // so the computed number must never be treated as a settled claim downstream.
    let dispute = recomp.case.dispute.as_ref().expect("dispute block present");
    assert!(!dispute.computation_contested);
    assert!(dispute.entitlement_contested);
    assert!(dispute.requires_human_review);
}

#[test]
fn e2e_slc_handoff_stage_a_negative_twin_stage_b() {
    let registry = usny_registry();
    let (slc_ref, recomp) =
        build_artifact_ref(&bundle_dir(), PINNED_HASH, FIXTURE_FILE, FIXTURE_CASE)
            .expect("verified bundle + recomputed disputed-001");
    assert_eq!(slc_ref.artifact_id, "slc-late-payment-001");
    assert_eq!(slc_ref.version, "0.1.0");
    assert_eq!(slc_ref.fixture_case, "disputed-001");
    assert_eq!(slc_ref.computed_result_hash, recomp.computed_result_hash);

    // ── Stage A: non-final, entitlement-contested fact → HL candidate, held by the consumer.
    let record_a = lbry_record(0, "HL", slc_ref.clone());
    let event_a = emit_leg_event(&record_a, &registry, NOW).expect("Stage A emission");
    assert_eq!(
        event_a.arrow_code, "2:LEG:PD:HL:0:C",
        "Stage A candidate must be the non-final hold shape"
    );
    let (decision_a, queued_a) = consume(event_a.clone());
    assert_eq!(decision_a, LegReviewDecision::HoldForReview);
    assert!(queued_a, "Stage A disposition must be queued for a human");

    // ── Negative twin: identical facts claiming PR at Finality 0 → Arrow SEM-013 rejects.
    let registries = Registries::load_embedded().expect("embedded Arrow registries");
    let pipeline = ValidationPipeline::new(&registries);
    let mut twin = event_a;
    twin.arrow_code = "2:LEG:PD:PR:0:C".into();
    let result = pipeline.validate_event(&twin);
    assert!(!result.passed, "PR at Finality 0 must not validate");
    assert!(
        result.violations.iter().any(|v| v.rule_id == "SEM-013"),
        "SEM-013 (FN+0 executing-action ceiling) must fire: {:?}",
        result.violations
    );

    // ── Stage B: the Finality 1 follow-up → PR candidate, still quarantined for a human.
    let record_b = lbry_record(1, "PR", slc_ref);
    let event_b = emit_leg_event(&record_b, &registry, NOW).expect("Stage B emission");
    assert_eq!(
        event_b.arrow_code, "2:LEG:FN:PR:1:C",
        "Stage B follow-up must be the final PR shape at the Sibyl's declared C ceiling"
    );
    let (decision_b, queued_b) = consume(event_b);
    assert_eq!(
        decision_b,
        LegReviewDecision::HoldForReview,
        "even a final PR-eligible fact never auto-executes (LC-001/LC-002)"
    );
    assert!(queued_b, "Stage B disposition must be queued for a human");
    // No ratification happens in this test; execution is exercised by e2e_pr_path.rs.
}
