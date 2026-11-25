# Tarquin

Tarquin is the downstream legal runtime for CipherLayer. It consumes machine-readable contract artifacts from Orlog, curates them into Lex.Codex, evaluates them through Lex.exe, validates legal facts through Sibyl / LBRY attestations, and emits Arrow `LEG` events through the LEG Origin Adapter.

> **Orlog** was formerly named *Contract Labs*. The former name is preserved only in
> historical decision records (`decisions/ADR-0001`–`0003`), provenance, and former-name
> disclosures; all active paths, pins, and imports use `orlog`.

Orlog and Tarquin are a supply chain, not peers:

```text
Orlog SLC artifacts
  -> Tarquin / Lex.Codex
  -> Lex.exe + Sibyl / LBRY
  -> Arrow LEG event
```

## Arrow v1.6 Terminology

Tarquin follows the completed Arrow v1.6.0 Action/Class-4 migration:

| Namespace | Current posture |
|---|---|
| Field 4 | **Action**, not Policy. New prose and code use `Action` / `action`; `Policy` appears only for legacy quotes, compatibility notes, or governance-policy concepts outside field 4. |
| Class 4 | **CONTROL**, not `POLICY`. The dropped compact alias `CTR` is not used in new material. |
| Origin `PCY` | Stays `PCY`; this is the protocol/governance policy source domain. |
| State `TL` | Timelocked State only; not an Action. |
| Actions `TR` / `CL` | Tarquin primitives now permitted: the governance gate is satisfied as of 2026-07-07 (Arrow `TRG-001`/`LFC-007`/`LFC-008` live via RFC-010; consumer-side `LC-013`–`LC-016` implemented in `specs/consumer-lifecycle-scope-rules.md`). `CL` is bound to Class 6 by `SEM-005` + `SEM-020`; post-closure lifecycle enforcement is `LFC-007` (live). `TR` is gated on `TRG-001` (live). |

## Canonical Documents

The active shared legal/protocol surface is intentionally small. Document filenames are
stable; document evolution lives in metadata and Git history.

| Spec | Status | Path |
|---|---|---|
| LEG Origin Amendment | Draft; reconciled to Arrow v1.6.0 | `specs/leg-origin-amendment.md` |
| LEG Jurisdiction Addendum | Draft; reconciled to Arrow v1.6.0 | `specs/leg-jurisdiction-addendum.md` |
| LEGAL_SYSTEM Foundation | Draft | `specs/legal-system-foundation.md` |
| Consumer Lifecycle & Scope Rules (LC-013…LC-016) | Live — Arrow RFC-010 Part 4, ratified and implemented gate-side 2026-07-07 | `specs/consumer-lifecycle-scope-rules.md` |

## Runtime Contracts

| Contract | Owner | Path |
|---|---|---|
| LBRY LegalAttestation | SBL-9 / Sibyl | `SBL-9/lbry-legal-attestation.md` |
| LEG Origin Adapter | Lex.exe | `lex-exe/leg-origin-adapter.md` |
| LEG Consumer (`arrow-consume-leg`) | Lex.exe | `lex-exe/leg-consumer.md` |

## Working Areas

| Area | Path |
|---|---|
| Solidity primitives | `primitives/` |

## Repository Layout

| Directory | Purpose |
|---|---|
| `specs/` | Current shared legal/protocol draft specifications. Active document filenames are stable; versions live in document metadata and Git history. |
| `decisions/` | Live architectural decisions. |
| `lex-codex/` | Future curated primitive library and Solidity primitive workspace. |
| `lex-exe/` | Lex.exe execution and adapter/consumer contracts, plus future implementation workspace. |
| `SBL-9/` | Sibyl / LBRY attestation contracts, plus future implementation workspace. |
| `primitives/` | Tarquin-native Solidity primitive library (on-chain leg). ADR-0001 Decision 2. |
| `crates/` | Tarquin Rust workspace (off-chain runtime). `tarquin-sibyl` first; path-deps on `arrow-core` + `arrow-identity`. |
| `exports/` | Generated artifacts for downstream consumers (`engine/`: TS backend + Rule IR artifact, emitted by `tarquin-engine-gen`; `arrow/`: Tarquin-owned LEG shared surface for Arrow import/check). |
| `imports/` | Pinned or returned upstream protocol artifacts Tarquin consumes without vendoring source. |
| `tests/` | Shared fixtures and cross-target validation (`fixtures/sibyl/`, `parity/` golden corpus + harness runner). |

## Version Discipline (per Arrow ADR-ARRW-001 / DOCVER-001)

- Active documents use **stable filenames** (`kebab-case.md`, no `-vMAJOR.MINOR` suffix) and
  evolve in place; Git history is the version record (`git log -- <path>`). Software/package
  and protocol release tags stay versioned — this rule is for *documents*.
- A document's `Status:` is `live` or `superseded`; no version baseline in the marker.
- Live ADRs stay in `decisions/`; superseded ADRs and documents are retired from
  the tree.
- Work that is not yet part of the canonical surface starts in `drafts/`.
- References inside specs point to repository paths, not Obsidian vault paths.
- Arrow integration remains downstream-facing: Orlog never talks to Arrow directly.
