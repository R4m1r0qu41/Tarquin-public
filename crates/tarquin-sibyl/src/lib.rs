//! # tarquin-sibyl
//!
//! The Tarquin **Sibyl layer** (SBL-9): the recognized legal-oracle registry and the LBRY
//! structured-attestation record. A *Sibyl* is a registered legal-oracle identity; its output
//! is an *LBRY record* that Lex.exe uses as the `proof_ref` for an Arrow LEG event.
//!
//! This crate is the first Rust component of Tarquin (see
//! `drafts/architecture/development-plan.md`, Phase 3). It is intentionally narrow and
//! deterministic — **no inference at runtime** — and reuses `arrow-identity` for DID/Ed25519
//! and `arrow-core` for shared LEG types.
//!
//! Specs of record:
//! - `drafts/sibyl/sibyl-registry.md` + `drafts/sibyl/sibyl-registry.schema.json`
//! - `SBL-9/lbry-legal-attestation.md`
//! - `drafts/sibyl/sibyl-legal-oracle.md`

pub mod docverify;
pub mod error;
pub mod lbry;
pub mod pipeline;
pub mod registry;
pub mod validate;

pub use docverify::{
    reduce, ClaimedFinding, DimensionClaim, DocumentVerificationFinding, EvidenceKind, EvidenceRef,
    SemanticState,
};
pub use error::SibylError;
pub use lbry::{LbryRecord, LbrySignature, StructuredFact};
pub use pipeline::{anchor_source, attest, extract, AttestParams, DocketOrder, DocketSource};
pub use registry::{
    AttestationScope, AuthorityProfile, AuthorityType, Confidence, Key, KeyAlgorithm, KeyPurpose,
    KeyStatus, LegSubtype, RegistryMetadata, SibylEntry, SibylRegistry, SibylStatus,
    SourceAuthorityBasis, VerificationRegime,
};
pub use validate::{Finding, Severity};
