# Tarquin / Arrow LEG Shared Surface v0.1

Status: draft export.

This file is the human-readable companion to `leg-shared-surface.v0.1.json`.
Arrow should import the JSON artifact at build time and pin its SHA-256.

## Current Codes

| Family | Codes |
|---|---|
| Action | `BL`, `HL`, `RW`, `TR`, `CL`, `PR` |
| Legacy Policy alias | `BL`, `HL`, `RW`, `TR`, `CL`, `PR` |
| LEG subtype | `JUD`, `ADM`, `SUPRA`, `LEG_TEXT`, `PREC` |
| Verification regime | `legal_attestation`, `structured_legal_attestation`, `multiple_legal_attestation` |
| Live LEG confidence | `B`, `C`, `D`, `E` |
| Unavailable live LEG confidence | `A` |

## Terminology Transition

Field 4 is **Action**. Existing Arrow field names/types that say `Policy` remain compatibility
aliases for the v0.1 transition window.

| Surface | Current posture |
|---|---|
| Field 4 | `Action`; legacy alias `Policy` |
| Class 4 | `CONTROL` / `CTR`; legacy name `POLICY` |
| Origin `PCY` | Unchanged governance/policy source domain |
| State `TL` | Timelocked State only; not an Action |
| Actions `TR` / `CL` | RFC-scoped before primitive implementation |

## Current Engine Rules

| Rule | Result | Binding |
|---|---|---|
| `ENG-SUPRA-SA-BL` | forced `BL` | `primitives: sanctions force-block (LEG-007)` |
| `ENG-PR-ELEVATION` | `PR` | `primitives: PR-gated escrow release (OA-011/LC-006)` |
| `ENG-DEFAULT-HL` | `HL` | engine fail-closed default |

## Current Primitive Bindings

| Primitive | Role | Status |
|---|---|---|
| `LegalHold` | Restriction primitive | Implemented stub, parity tested |
| `EscrowRelease` | PR-gated release primitive | Implemented stub, parity tested |
| `LegConsumerGate` | Human ratification gate | Abstract deployment stub |
| `RuleEngineV01` | Generated Solidity rule engine | Generated from Rule IR |

## Action Primitive Layer

| Action primitive | Base Action | Current implementation |
|---|---|---|
| Block | `BL` | Covered by `LegalHold` / force-block rules |
| Hold | `HL` | Covered by `LegalHold` |
| Rework | `RW` | Covered by review/cure handling; no separate primitive yet |
| Proceed | `PR` | Covered by `EscrowRelease` |
| Trigger | `TR` | RFC-scoped before implementation |
| Closed | `CL` | RFC-scoped before implementation |

`TR` must not be treated as generic `PR`. The current Tarquin position is that `TR` means Trigger,
with transfer semantics carried by Class `5` when applicable. No real `TransferOrder` primitive
should ship before RFC-009 scopes the object, target, authority, finality, and ratification rules.

`CL` must not be treated as a generic lifecycle note. RFC-009 must decide whether Action `CL` and
Class `6` work in tandem, whether Class `6` is narrowed, or whether Class `6` is deprecated.

## LEG-007 Status

The LEG-007 derivation matrix is incomplete.

Required final shape:

```text
leg_subtype x Finality x verification_regime -> permitted Action set
```

Until that matrix is complete and imported by Arrow, real on-chain `PR` remains gated.

## Import Contract

Arrow should:

1. Read `exports/arrow/leg-shared-surface.v0.1.json`.
2. Verify `exports/arrow/leg-shared-surface.v0.1.sha256`.
3. Generate Arrow-side validation fixtures or registry rows.
4. Fail CI if generated Arrow files drift from the pinned Tarquin artifact.
