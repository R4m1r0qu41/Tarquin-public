# ADR-TARQ-001 — Adopt Arrow Governance (ADR-ARRW-001)

**Status:** live
**Date:** 2026-06-19
**Scope:** Tarquin repository only.
**Governing authority:** Arrow [ADR-ARRW-001](../../Arrow/docs/architecture/decisions/ADR-ARRW-001-protocol-authority-and-baseline-liveness.md) — Protocol Authority, Baseline Liveness, and ADR Governance.
**Ratified by:** Soft-reboot human-ratification decisions 1–5.

---

## 1. Context

Arrow ADR-ARRW-001 establishes Arrow as the normative protocol authority and the single
owner of the stack-wide liveness/archive discipline and the repository-specific ADR scheme.
It supersedes the *stack-governing scope* of Tarquin [ADR-0002](ADR-0002-v1.6-reboot-and-liveness-v1.md),
which previously hosted that policy here. This adoption ADR records Tarquin's repository-local
application; it does not restate or fork the governing decision.

## 2. Decision

| # | Tarquin-local adoption |
|---|---|
| 1 | Tarquin accepts Arrow ADR-ARRW-001 as the governing cross-repository authority. Tarquin does **not** independently version Arrow protocol semantics; it declares Arrow compatibility in `arrow-compatibility.json` and pins Arrow's rule surface in `imports/arrow/PIN.json` (BASE-001 / ADR-0003). |
| 2 | **ADR-0002 supersession.** ADR-0002's `STATUS:` is set to `superseded` with `superseded-by:` pointing to ADR-ARRW-001. Only the header/supersession note is added; the decision body is preserved verbatim as the v1.6 soft-reboot historical record (decision 1 — do not rewrite Tarquin history). |
| 3 | **ADR identifier scheme (ADR-001).** New Tarquin ADRs use `ADR-TARQ-NNN`; this is the first. The pre-existing `ADR-0001`/`ADR-0002`/`ADR-0003` are preserved under their original identifiers as historical records and are **not** renumbered (ADR-ARRW-001 ADR-003). |
| 4 | **Active naming transition (decisions 3, 5).** The upstream foundry's active name is **Orlog**. Tarquin's import path is renamed `imports/contract-labs/` → `imports/orlog/`, with `PIN.json` (`source_repo`, `pinned_path`), the import README, and all Rust source/test references updated as one coordinated change. The pinned bundle's MC-006 `artifact_hash` is unchanged (the hash is over bundle-relative paths), so the pin and SLC-handoff tests remain green. The former name "Contract Labs" survives only in historical ADR bodies, the `PIN.json`/README former-name disclosures, and provenance. |
| 5 | **Document-filename versioning retired (decision 4 / DOCVER-001).** Active Tarquin documents drop rolling `*-v0.1.md` suffixes for stable names tracked through Git history (`specs/`, `lex-exe/`, `SBL-9/`, and active `drafts/`). All active references — Rust doc-comments, Solidity comments, `exports/arrow/leg-shared-surface.v0.2.json` `generated_from` (sidecar `.sha256` recomputed), `Cargo.toml`, READMEs, and cross-doc links — are updated. Excluded: `archive/` snapshots (ARC-001 keeps era markers), the archived `fable-5-*-review-*-v0.1.md` artifacts, and historical ADR Source lists (preserved as written). Interface-artifact and schema-compatibility versions (e.g. `leg-shared-surface.v0.2`) are **not** documents and keep their versions. |

## 3. Preserved-as-history references

Tarquin ADR-0001/0002/0003 are decision records dated 2026-06-07/06-14. Their bodies — including
references to the former name "Contract Labs" and to pre-retirement filenames such as
`imports/contract-labs/PIN.json` and `specs/*-v0.1.md` — are **left intact** as accurate
provenance for their era (ADR-ARRW-001 ARC-001/ADR-003). The renamed targets remain resolvable
through Git history. New and active material uses the current names.

## 4. Consequences

- One governing source (Arrow ADR-ARRW-001); Tarquin references it and cannot drift it.
- No protocol semantics, registries, fixtures, pinned hashes, or test outcomes change as a
  result of this ADR; the rename and filename retirement were reference-only and verified by
  the existing pin, SLC-handoff, and parity tests.
- A residual stylistic cost: live ADR-0003 and Accepted ADR-0001 now cite former-era paths
  in their Source lists. This is accepted as history-preservation rather than rewritten,
  consistent with decision 1.
