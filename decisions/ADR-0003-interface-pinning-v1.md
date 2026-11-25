# ADR-0003 — Interface Pinning Across the Three Legs

**Status:** live
**Date:** 2026-06-14
**Amended:** 2026-07-06 — corrected the pin path after the Contract Labs → Orlog rename (`imports/contract-labs/` → `imports/orlog/`), and marked the Arrow rule-surface seam CLOSED: `Arrow/exports/rule-surface.v1.6.0.json` exists and is pinned by `imports/arrow/PIN.json` (first pinned 2026-06-25 at Arrow commit `28da84d`; current pin `8c828fb`).
**Scope:** Stack-level — Arrow Protocol, Tarquin, Contract Labs. Sibling to [ADR-0001](ADR-0001-stack-boundaries-v2.md) and [ADR-0002](ADR-0002-v1.6-reboot-and-liveness-v1.md).

---

## 1. Context

The LEG renumbering drift had a precise mechanism: **Tarquin referenced Arrow's rules by paraphrasing rule numbers in prose** (e.g. "LEG-007 derivation gate" written into a sentence). When Arrow renumbered `LEG-007 → LEG-006` in v1.5.1, **nothing failed** — Tarquin's prose silently lagged for two version cycles until a human noticed.

The stack already has the *right primitive* in two places:
- `Tarquin/imports/orlog/PIN.json` (path corrected 2026-07-06; original text cited the pre-rename `imports/contract-labs/` path) — pins each SLC bundle to a source commit + MC-006 content hash; W3 verified zero drift.
- `Tarquin/exports/arrow/leg-shared-surface.v0.2.json` — pins `(rule_id, semantic_slug)` pairs for the LEG surface.

This ADR makes that pattern **the standard** for every cross-repo seam, and fixes the one direction that was missing (Arrow's rule-ID surface).

## 2. Decision

| Rule | Statement |
|---|---|
| **IFP-001** | Each repo **publishes** its outward-facing contract as a versioned, **content-hashed** interface artifact under `exports/` (or equivalent). The artifact — not prose — is the source of truth for the seam. |
| **IFP-002** | The artifact pins **`(id, semantic_slug)` pairs** for every cross-repo identifier (rule IDs, reason codes, field-4 codes, field names). A mismatch between an `id` and its `semantic_slug` on either side is a failure. |
| **IFP-003** | A **consumer pins the producer's artifact by content hash** (the `PIN.json` pattern): `{ artifact, version, source_commit, hash }`. |
| **IFP-004** | **CI recomputes the hash and compares to the pin; mismatch = build failure** — loud, not silent. This is the **W3** (pin conformance) pattern generalized; **W6** audits semantic coherence on top. |
| **IFP-005** | Cross-repo references **in prose MUST cite a pinned `id`/`slug`** from the interface artifact, never paraphrase a number inline. (This rule alone would have caught the LEG renumbering.) |

## 3. The seam map (who publishes, who pins)

| Seam | Producer (publishes) | Consumer (pins) | Artifact | Status |
|---|---|---|---|---|
| **Arrow rule/registry surface → Tarquin** | **Arrow** | Tarquin | `Arrow/exports/rule-surface.v1.6.0.json` — rule IDs, reason codes, field-4 (Action) codes, LEG numbering, each with `semantic_slug` + hash | **Closed 2026-06-25** — pinned by `imports/arrow/PIN.json`; re-pinned 2026-07-06 at Arrow `8c828fb` (SEM-015/FB-001 pins added) |
| **Tarquin LEG obligations → Arrow** | Tarquin | Arrow | `Tarquin/exports/arrow/leg-shared-surface.v0.2.json` (+ `.sha256`) | Exists; conformant after the v1.6 reconciliation |
| **Orlog SLC artifacts → Tarquin** | Orlog (formerly Contract Labs) | Tarquin | `Tarquin/imports/orlog/PIN.json` (MC-006 hashes) | Exists; W3-verified clean |

**Key fix (IFP / the missing direction — closed):** Arrow now publishes `exports/rule-surface.v1.6.0.json` and Tarquin pins it in `imports/arrow/PIN.json` (since 2026-06-25). The loop this ADR called for is wired; what remains open is the CI leg of IFP-004 (automated recompute-and-compare on every build) tracked by the three-repo workflow replacement of Tarquin PR #1.

## 4. Minimal artifact schema

```json
{
  "interface": "arrow-rule-surface",
  "version": "1.6.0",
  "baseline": "arrow-v1.6.0",
  "pairs": [
    { "id": "LEG-006", "semantic_slug": "leg-action-derivation-gate" },
    { "id": "LEG-007", "semantic_slug": "leg-supra-sanctions-force-bl" },
    { "id": "MSG-006", "semantic_slug": "leg-subtype-schema-constraint" },
    { "id": "LEG_ACTION_RANGE_VIOLATION", "semantic_slug": "leg-action-range-violation" }
  ],
  "content_hash": "sha256:…"
}
```

Consumer pin (the `PIN.json` shape, generalized):

```json
{ "artifact": "arrow-rule-surface", "version": "1.6.0",
  "source_commit": "<sha>", "hash": "sha256:…" }
```

## 5. Consequences

- **Positive:** cross-repo identifier drift becomes a **build failure**, not a discovery weeks later. Reconciliation work like the v1.5→v1.6 LEG pass becomes a one-line pin bump + a CI green/red, instead of a manual archaeology session. Pairs with ADR-0002: flattened live specs + pinned seams = a system that stays coherent by construction.
- **Cost:** Arrow gains a small `exports/` publishing step (generate the rule-surface artifact from its registries — a natural extension of `validate-registries.js`). Each consumer gains a pin file and a CI check (the W3 pattern, already proven).
- **Wiring:** generate Arrow's `rule-surface` from `registries/*.json` so it can't drift from the registries; check it with W3/W6 in CI.
