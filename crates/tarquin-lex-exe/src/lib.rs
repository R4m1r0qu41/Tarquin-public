//! # tarquin-lex-exe
//!
//! Lex.exe — the only Tarquin component that touches the Arrow LEG interface. It brackets the
//! Arrow boundary with two thin drivers over Arrow's own crates:
//!
//! - [`emit`] (Phase 3.3) — turn a **validated** LBRY record + its Sibyl into a protocol-valid
//!   `origin = LEG` event, via `arrow-origin-leg`. Runs the producer-side preflight gate
//!   (`OA-*` from `lex-exe/leg-origin-adapter.md`) before anything is built.
//! - [`consume`] (Phase 3.4) — drive the fail-closed, human-in-the-loop `arrow-consume-leg`
//!   review queue. Nothing executes; a verified LEG disposition is held for a human.
//! - [`router`] (Phase 4.5) — the post-ratification execution router (LC-009/LC-010): a `PR`
//!   disposition dispatches to its parity-certified primitive only after a human ratifies,
//!   exactly once per event.
//! - [`lexcodex`] (Stage 2 SLC handoff) — pinned Orlog bundle verification (MC-006
//!   artifact hash), deterministic fixture recomputation, and `related_slc_artifacts` binding.
//!
//! Lex.exe **classifies and records**; it never determines applicable law or acts autonomously
//! (ADR-0001). The runtime is AI-free and deterministic — time is passed in, never read.

pub mod consume;
pub mod emit;
pub mod lexcodex;
pub mod lifecycle;
pub mod router;
pub mod scope;

pub use consume::{consume_envelope, ConsumeConfig};
pub use emit::{emit_leg_event, preflight, to_legal_attestation, EmissionBlocked};
pub use lexcodex::{
    build_artifact_ref, confirm_artifact_ref, evaluate_late_payment, verify_pinned_bundle,
    LexCodexError,
};
pub use lifecycle::{LifecycleRefusal, LifecycleView};
pub use router::{inputs_from_lbry, ExecutionRouter, RefusalReason, RouteOutcome};
pub use scope::{RegisteredWorkflow, TrHoldReason, TrRoute, TrScopeTable};
