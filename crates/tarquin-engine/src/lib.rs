//! # tarquin-engine
//!
//! The Tarquin **native engine** (Phase 4.1): a deliberately tiny, deterministic **Rule IR**.
//! It is the single place legal decision logic is authored once and compiled to both targets —
//! the off-chain (AP-TS) path and the on-chain Solidity `primitives/` — so the two stay in
//! parity (ADR-0001 Decision 2; `drafts/engine/tarquin-native-engine.md`).
//!
//! The evaluator is **total, deterministic, and pure**: same inputs → same [`Decision`] on every
//! run and (by construction) on every target. No clock, no I/O, no inference. Fail-closed: the
//! default when no rule yields is the most restrictive coherent action (`HL`).
//!
//! v0.1 expresses exactly the two canonical rules the stack already specifies:
//! - **SUPRA + SA + Finality=1 → `BL`**, non-overridable (Arrow LEG-007 / adapter OA-010);
//! - the **conjunctive `PR` gate** (adapter OA-011 / consumer LC-006): `PR` only when every
//!   condition holds, else `HL`.
//!
//! Each rule carries a `parity_ref` naming its on-chain counterpart in `primitives/` — the hook
//! the future parity harness (PR-003 / EN-O4) binds.

pub mod codegen;
pub mod eval;
pub mod ir;
pub mod rules;

pub use codegen::{governing_order, parse_corpus, ruleset_hash, CodegenError, CorpusCase};
pub use eval::{evaluate, Decision};
pub use ir::{Action, Condition, Inputs, Rule};
pub use rules::canonical_ruleset;
