# ADR-0001: Tarquin, Accord, and Arrow Boundaries

**Status:** live  
**Date:** 2026-06-07  
**Deciders:** CipherLayer

---

## Context

Tarquin is the CipherLayer legal stack: a runtime, primitive library, and oracle layer that
binds to Arrow Protocol for external state-transition semantics. It must (1) produce
machine-readable Smart Legal Contracts (SLCs) without rebuilding a mature contract-template
substrate from scratch, (2) execute legal logic deterministically with no AI at runtime, and
(3) emit and consume Arrow LEG events through two designated interfaces — the LEG Origin
Adapter (Lex.exe as emitter) and the `arrow-consume-leg` consumer (fail-closed,
human-ratification gate).

Tarquin is fed by a separate upstream foundry, **Contract Labs** (`Dev/Contract Labs`), which
vendors the Accord Project (AP) markdown and template stack (`vendor/accord-markdown`) and
produces machine-readable SLC artifacts — the raw material Lex.Codex curates into an audited
primitive library. Contract Labs does not bind to Arrow directly; Tarquin is the only layer
that touches the Arrow LEG interface.

The supply chain is:

> **Contract Labs** (author / represent) → **Lex.Codex** (curate, audit, compile to Solidity
> and off-chain) → **Lex.exe** (orchestrate / execute) → **Sibyl layer / SBL-9** (attest and
> package) → **Arrow LEG gate** (semantic / policy validation) → **Consumer** (act,
> fail-closed, human ratifies).

## Decision

Adopt AP's mature contract-template substrate — Concerto, TemplateMark, Cicero, and the
TypeScript-in-template-archive logic direction — as the off-chain leg of Lex.Codex and
Lex.exe, without building those layers from scratch. Tarquin retains native ownership of
legal-source management, rule extraction, the Solidity on-chain primitive library, the Sibyl
oracle layer (SBL-9 / SBYL), the Sibyl identity registry, and all Arrow interop. If the AP
TypeScript direction proves insufficient for Arrow-specific or determinism requirements in
early passes, a Tarquin-native logic engine is introduced incrementally — not from scratch.

Three decisions locked with this ADR:

1. **Lex.exe logic path:** AP TypeScript-in-template-archive direction, not Ergo (see
   Constraints). Diverge to a Tarquin-native engine only if early passes surface a concrete
   gap.
2. **Solidity primitives:** Tarquin-native, housed in `primitives/` within the Tarquin repo.
   AP does not target Solidity; this is Tarquin's own engineering surface.
3. **Sibyl registry:** Start now, designed as a profile of the RFC-007 Part B
   recognized-authority record to allow lossless schema convergence when that RFC graduates.

## Boundaries

| Area | Decision |
|---|---|
| **arrow-protocol** | External semantic/policy contract that Tarquin targets; owns the LEG interface, Step 5b gate, LEG-004–014, `structured_legal_attestation`, `leg_jurisdiction`, and LEGAL_SYSTEM (LSY) semantics |
| **Contract Labs** (`Dev/Contract Labs`) | Upstream foundry repo; authors and represents SLCs using the AP stack; does not bind to Arrow |
| **Lex.Codex** | Tarquin-native source-law and legal-knowledge layer; curates Contract Labs output into an audited primitive library |
| **Solidity primitives** | Tarquin-native; housed in `primitives/`; dual-target parity with the off-chain path is the primary engineering risk of the stack |
| **Structured rules** | Tarquin-native legal primitive model |
| **Concerto** | Adopt as typed schema / data-model layer |
| **TemplateMark** | Adopt as legal wording and model-binding layer |
| **Template Engine / AgreementMark** | Adopt as rendering and document-generation bridge |
| **Cicero** | Retain as compatibility and validation target |
| **Ergo** | **Rejected** — AP maintainers are replacing it with a TypeScript-based engine (see Constraints); do not build Lex.exe logic on Ergo |
| **TypeScript logic (AP direction)** | Off-chain execution path for Lex.exe and template logic; does not produce Solidity; kept strictly separate from the on-chain Solidity path |
| **APAI** | Accord Project Agent Interface (AP's Model Context Protocol–based agent interface for drafting / agreement management); monitor for AI-assisted-drafting alignment; does not affect runtime — runtime remains AI-free |
| **SBL-9 / SBYL (Sibyl layer)** | Tarquin-native oracle and attestation layer; each registered legal-oracle identity is a **Sibyl** (a specialized attestation oracle); a Sibyl's output is an LBRY / Libri IX structured attestation record (cryptographic anchor + Sibyl Decentralized Identifier, DID); LBRY record = `proof_ref` target in the emitted Arrow LEG event; Sibyl DID = `asserted_by`; verification regime = `structured_legal_attestation`; **single-Sibyl ceiling = Confidence B** (per Arrow's live `confidence_subprofiles.json`; ceiling, not default); independent multi-Sibyl corroboration uses `multiple_legal_attestation` |
| **Sibyl registry** | Tarquin-native; start now; schema is a profile of the RFC-007 Part B recognized-authority record; single shared schema with the Arrow track — converges losslessly when Part B is specified |
| **Arrow** | Native interop layer; interface is: (1) LEG Origin Adapter — Lex.exe emits Arrow LEG events, deriving `leg_subtype`, verification regime, and policy claim; (2) `arrow-consume-leg` — fail-closed consumer, receipts quarantined, human-ratification gate; Arrow validates Tarquin output, never replicates legal reasoning |

## End-to-End Dataflow

```text
Real-world legal fact
  -> Sibyl attestation
  -> LBRY record
       structured attestation
       cryptographic anchor
       Sibyl DID
  -> Lex.exe ingestion
  -> Lex.Codex parameter mapping
       built from Contract Labs SLC templates
  -> leg_subtype / regime / policy claim
  -> Arrow LEG event
       Lex.exe = Origin Adapter
       LBRY = proof_ref
       Sibyl = asserted_by
       regime = structured_legal_attestation
  -> Arrow LEG gate
  -> arrow-consume-leg
       fail-closed
       human ratification
  -> execution
       Solidity primitive on-chain, or off-chain action
```

Layer rule: Contract Labs never talks to Arrow; Arrow never reasons about contracts. Lex.exe and the Sibyl layer are the only boundary that touches both the machine-readable contract supply chain and the Arrow LEG interface.

## Constraints

- **Ergo is deprecated upstream.** As of the AP 2025 planning cycle, maintainers concluded
  that peripheral repositories with low participation will be archived, and announced a new
  contract-logic engine to replace Ergo, with TypeScript as the supported logic language.
  Building Lex.exe logic on Ergo is therefore (d) not viable — explicitly rejected.

- **AP does not target Solidity.** AP's execution path is off-chain (JavaScript / TypeScript).
  The Solidity / on-chain half of dual-target compilation is wholly Tarquin's engineering
  surface and the primary technical risk of the stack. Budget and audit attention concentrate
  here.

- **Confidence A is unavailable to live LEG-origin events** under current Arrow regimes.
  `structured_legal_attestation` has a **Confidence B ceiling** for a fully structured single
  Sibyl/LBRY proof, but B is not automatic; the Sibyl registry can still declare C or D for weaker
  artifacts or operating posture. Independent multi-Sibyl corroboration uses
  `multiple_legal_attestation` and also has a B ceiling for now. Confidence A remains a future RFC
  item tied to `legal_native_attestation` or equivalent legal-system-native proof — do not represent
  A as available from today's Sibyl stack.

- **AP repo pruning in progress.** Verify that the packages inside `vendor/accord-markdown`
  are in AP's designated core set (not slated for archive per the AP Repository Status Wiki).
  As a frozen vendor snapshot this is not blocking, but determines whether upstream patches are
  ever pulled.

- **Tarquin is not Arrow.** Arrow's concern is verified external state transition; Tarquin's
  concern is legal computation. Tarquin must not absorb relayer, validator, or transport
  infrastructure — those belong to the Arrow / AMP (Arbitrary Message Passing) layer.

## Consequences

- Tarquin moves quickly on the off-chain SLC layer by reusing AP where it is mature (Concerto,
  TemplateMark, Cicero, TypeScript templates) without committing to AP's governance trajectory
  at the legal-reasoning or interop layers.
- The Solidity primitive library (`primitives/`) and the off-chain AP logic path must be kept
  in deterministic parity. This is the primary build risk; it is where the deterministic test
  suite discipline earns its keep.
- The Sibyl registry is the single shared schema artifact with the Arrow track. Any changes to
  RFC-007 Part B must be evaluated against the Sibyl registry before either side hardens its
  schema.
- AP component adoption is substrate only. Tarquin retains legal-reasoning and Arrow-interop
  layers unconditionally. Accord remains a substrate; Tarquin remains the legal computation
  and state-coordination framework.

## Action Items

1. [ ] Verify `vendor/accord-markdown` packages against AP Repository Status Wiki — confirm
       each is in the core (not archive) set
2. [ ] Confirm per-package license (expected Apache-2.0 per Linux Foundation governance)
3. [x] Create `primitives/` folder in Tarquin; stub initial Solidity structure
       (`primitives/` — README, `Types.sol`, interfaces, `LegConsumerGate.sol`, `LegalHold.sol`)
4. [x] Draft Sibyl registry schema as a profile of RFC-007 Part B authority record
5. [x] Draft LEG Origin Adapter interface spec — how Lex.exe emits Arrow LEG events (the
       producer contract for the Arrow binding); this is the immediate next spec artifact

## Sources

- Ricardian Contracts 2026 research note: `archive/research/Ricardian Contracts 2026.md`
- Ricardian Contracts 2026 shared chat: https://chatgpt.com/share/6a25d83c-14d8-83e8-8809-a3dc59452c81
- Arrow Protocol/Tarquin LEG drafts: `specs/leg-origin-amendment-v0.1.md`,
  `specs/legal-system-foundation-v0.1.md`,
  `specs/leg-jurisdiction-addendum-v0.1.md`
- Arrow RFC graduation review (external, `arrow-protocol` repo):
  `rfc/rfc-readiness/rfc-graduation-2026-06-07.md`
- Arrow RFCs Tarquin depends on (external, `arrow-protocol` repo):
  `rfc/drafts/RFC-006-Robust-Chain-Registry-v0_4.md`,
  `rfc/drafts/RFC-007-Declared-Policy-and-Integration-Strength-v0_2.md`
- AP 2025 planning and APAI announcement: https://accordproject.org/news/announcing-accord-project-agent-interface/
- Repo topology: `drafts/architecture/repo-topology.md`
- Sibyl registry profile: `drafts/sibyl/sibyl-registry-v0.1.md`
- LBRY LegalAttestation: `SBL-9/lbry-legal-attestation-v0.1.md`
