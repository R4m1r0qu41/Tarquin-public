# Tarquin → Arrow Shared LEG Surface v0.2 (human-readable)

Generated companion to `leg-shared-surface.v0.2.json` (pin: see `.sha256`). Supersedes v0.1.

## What changed from v0.1

| Item | v0.1 | v0.2 |
|---|---|---|
| Field 4 name | asserted `Action` while Arrow v1.5.2 shipped `Control` (contradiction) | **`Action`, final** — executed in Arrow v1.6.0-draft; registry `actions.json`; rules `ACT-001/002` |
| Class 4 | proposed `CONTROL` + `CTR` alias | **`CONTROL` ratified**; `CTR` alias dropped; anchored by draft `control_subtypes.json` (SEM-021 reserved) |
| `policies` legacy block | duplicated action list | removed — replaced by ALS-001 alias rule (legacy names accepted on input, never emitted, equal-or-reject) |
| LEG derivation matrix | "LEG-007, incomplete, gates real PR" | corrected to **LEG-006**, status **factored-complete**; remaining ask is a generated derived-matrix export from Arrow, not new spec work |
| Rule references | bare IDs, pre-v1.5.1 numbering in places | **pinned `(rule_id, semantic_slug)` pairs**; Arrow rule IDs permanent from v1.6.0 |
| CL semantics | open (RFC-scoped) | **bound**: SEM-005 + SEM-020 bidirectional, live in Arrow v1.6.0 (fixture F-12); LFC-007 reserved |
| TR semantics | open (RFC-scoped) | gated on TRG-001 (`action_target`), classes {2,5}, never consumer-executable |
| Follow-up events | not modeled | affirmed / reversed / discharged / post-closure model exported (`follow_up_event_model`) |

The JSON export aligns to Arrow's v1.6.0 protocol and registry artifacts. Arrow RFC-008 v0.3 is
historical design provenance, not a machine-consumed source path for this contract.

## Stable facts (unchanged)

- LEG subtypes: `JUD`, `ADM`, `SUPRA`, `LEG_TEXT`, `PREC`.
- Regimes and ceilings: `legal_attestation` C; `structured_legal_attestation` B (OA-014 signature required for B); `multiple_legal_attestation` B (SO-001 gated — must not be minted until independence evidence is defined).
- Confidence A unavailable for live LEG events; B is a ceiling, not a default.
- Primitives: `LegalHold` (BL/HL/RW), `EscrowRelease` (PR), `LegConsumerGate` (ratification stub). No TR/CL primitives until TRG-001 / LFC-007 land.
- Engine rules: `ENG-SUPRA-SA-BL` (forced BL), `ENG-PR-ELEVATION` (PR), `ENG-DEFAULT-HL` (fail-closed HL).
- `primitives/Types.sol` names the field-4 enum `Action`; no `Policy` alias remains in Tarquin runtime code.

## Import contract

Arrow pins the SHA-256 of the v0.2 JSON. CI on either side fails on: hash drift without review, generated-fixture drift, or `(rule_id, slug)` pin mismatch.
