# LEG Consumer (`arrow-consume-leg`) — Interface Specification

**Version:** v0.1
**Status:** Draft
**Date:** 2026-06-07
**Track:** Tarquin / Lex.exe
**Document:** `lex-exe/leg-consumer.md`
**Companion to:** `lex-exe/leg-origin-adapter.md` (this closes OI-006)
**Depends on:** `arrow-protocol` v1.6.0; `specs/leg-origin-amendment.md`;
`decisions/ADR-0001-stack-boundaries-v2.md`

---

## §1 Scope

This document defines the obligations of the `arrow-consume-leg` consumer: the **fail-closed,
human-ratification gate** that sits between an Arrow LEG disposition and any Tarquin execution
(on-chain Solidity primitive or off-chain action).

The Origin Adapter (`leg-origin-adapter.md`) is the **producer** contract — how Lex.exe
emits. This is the **consumer** contract — how Tarquin acts (or refuses to act) on what comes
back. Together they bracket the Arrow boundary.

This document does **not** govern:

- Arrow's internal Step 5b action derivation (Arrow-side).
- Lex.exe's emission obligations (the Origin Adapter spec).
- The Sibyl attestation protocol (Sibyl Layer spec / `drafts/sibyl/`).

**Core stance — fail-closed.** Absence of an explicit, ratified, in-range `PR` is a refusal,
not a pause that times out into action. There is no autonomous execution path anywhere in this
consumer (ADR-0001; OA-013).

---

## §2 What "produce the consumer" requires

To stand up `arrow-consume-leg`, the following must exist. Items marked **(have)** exist in
the repo today; **(need)** are the gap this spec scopes.

| # | Component | Status |
|---|---|---|
| C-1 | Disposition intake — receive the Arrow-validated LEG disposition + its envelope | **need** (this spec) |
| C-2 | Quarantine store — every disposition is held inert on receipt | **need** (this spec; on-chain stub in `primitives/src/core/LegConsumerGate.sol`) |
| C-3 | Re-validation of the disposition against the source LBRY record + Sibyl registry | **need**; depends on Sibyl registry impl (OI-002) and `SBL-9/lbry-legal-attestation.md` **(have, draft)** |
| C-4 | Human-ratification gate — identity, authority, dual-control policy | **need** (this spec defines the contract; identity layer is external) |
| C-5 | Execution router — dispatch to a Solidity primitive or an off-chain action | **need**; on-chain surface stubbed in `primitives/` **(have, stub)** |
| C-6 | Audit log — append-only record of every disposition, decision, and ratifier | **need** (this spec) |
| C-7 | Action/disposition type model shared with the adapter | **have** (`primitives/src/core/Types.sol`) |

The minimum first build is **C-1, C-2, C-4, C-6** (intake → quarantine → ratify → audit) with
a no-op execution router. That alone is a safe, demonstrable fail-closed gate.

---

## §3 Lifecycle

```text
Arrow disposition
  -> [C-1] intake
  -> [C-3] re-validate (LBRY resolves? Sibyl still registered+active? disposition unmutated?)
  -> [C-2] QUARANTINE  (inert; default terminal disposition is "held")
  -> [C-4] human ratification gate
        |-- reject -> REJECTED (terminal, audited)
        |-- no decision -> stays QUARANTINED forever (never auto-executes)
        `-- ratify (only if all §5 conditions hold) -> RATIFIED
  -> [C-5] execute (Solidity primitive permits() AND/OR off-chain action)
  -> EXECUTED (audited)
```

States: `QUARANTINED → {REJECTED | RATIFIED} ; RATIFIED → EXECUTED`. There is no edge from
`QUARANTINED` to `EXECUTED` that does not pass through `RATIFIED`.

---

## §4 Intake & Quarantine Rules

**LC-001 — Quarantine on receipt (Fatal).**
Every disposition MUST be stored in the quarantine state before any other processing. A
disposition that cannot be stored MUST be dropped and alerted; it MUST NOT be acted on.

**LC-002 — No execution on a non-PR disposition (Fatal).**
Only Action `PR` is executable. `BL`, `HL`, `RW`, `TR`, and `UNSET` are terminal-on-quarantine
for execution purposes (`HL`/`RW` may route to rework queues; they never reach `_execute`).

**LC-003 — Re-validate, do not trust the envelope (Fatal).**
Before a disposition is eligible for ratification, the consumer MUST independently re-confirm:
(a) `proof_ref` resolves to a current, valid LBRY record (§7 of the adapter spec);
(b) the `asserted_by` Sibyl DID is still **registered and active** at ratification time, not
merely at emission time;
(c) the disposition content has not mutated since quarantine (hash check);
(d) the disposition's Confidence is not `A` (OA-006 mirror).
Any failure routes to `REJECTED`.

**LC-004 — Disposition downgrade is binding (Fatal).**
If Arrow derived a more restrictive Action than Lex.exe proposed, the consumer acts on the
**derived** value only. The consumer MUST NOT consult Lex.exe's original proposal to escalate
(mirror of OA-012, no silent re-escalation).

---

## §5 Human-Ratification Gate

**LC-005 — Ratifier authority (Fatal).**
A ratifier MUST be a registered identity authorized for the disposition's `leg_subtype`,
jurisdiction, and execution domain. Authorization resolution is delegated to the identity
layer; this consumer only enforces that an authorized ratifier acted.

**LC-006 — Conjunctive PR execution conditions (Fatal).**
The consumer MUST NOT execute a `PR` disposition unless **all** of the following hold at
ratification time — the consumer-side mirror of OA-011:

1. `verification_regime == structured_legal_attestation` (LBRY satisfies §7 fully).
2. `proof_ref` contains attested, **instrument-specific** authorization for the proposed
   action — not a general order.
3. The `asserted_by` Sibyl DID is currently registered and active.
4. `Finality == 1` (not under appeal/challenge).
5. The executing primitive's on-chain `permits()` returns true (Lex.Codex gate).
6. An authorized human ratifier has explicitly confirmed (LC-005).

If any condition fails, the disposition MUST NOT execute. It routes to `HL` (rework) or
`REJECTED`. *(This intentionally restates OA-011 at the consumer; the producer-side check is
advisory to Arrow, the consumer-side check is the last line of defence.)*

**LC-007 — Dual control SHOULD apply to high-impact actions (Warning→Fatal by policy).**
For irreversible or high-value executions, deployments SHOULD require two distinct authorized
ratifiers. A deployment MAY promote this to Fatal in its policy profile.

**LC-008 — Ratification does not expire into action (Fatal).**
A quarantined disposition that is never ratified remains quarantined indefinitely. Wall-clock
elapse MUST NOT convert a hold into an execution (aligns with RFC-007 DPL-004: no silent
lifecycle mutation on time alone).

---

## §6 Execution Router

**LC-009 — Bind to a parity-tested primitive (Fatal).**
An on-chain execution MUST dispatch to a primitive whose `parityRef()` binds it to its
off-chain engine counterpart. A primitive with no passing parity test MUST NOT be reachable
from the router (ADR-0001 dual-target parity; PR-003).

**LC-010 — Single-shot execution (Fatal).**
A disposition transitions `RATIFIED → EXECUTED` exactly once. Replay (same `arrow_event_id`)
MUST be rejected.

**LC-011 — Off-chain actions carry the same gate (Fatal).**
An off-chain execution path (notice, filing, custody instruction) is bound by LC-001 through
LC-010 identically. "Off-chain" is not an exception to fail-closed.

---

## §7 Audit

**LC-012 — Append-only audit (Fatal).**
Every state transition MUST be recorded append-only with: `arrow_event_id`, `proof_ref`,
`asserted_by`, the disposition at each transition, the ratifier identity, timestamp, and the
decision rationale for rejects/holds. The audit record is the evidentiary artifact for the
action and MUST be retained for the applicable legal retention period.

---

## §8 On-chain ⇄ off-chain mapping

| This spec | On-chain (`primitives/`) |
|---|---|
| C-1 intake / LC-001 | `IArrowLegConsumer.receiveDisposition` → `Quarantined` event |
| C-4 ratify / LC-006 | `IArrowLegConsumer.ratify` (reverts unless Action == PR, Confidence != A) |
| reject / LC-003 | `IArrowLegConsumer.reject` → terminal |
| C-5 execute / LC-009 | `LegConsumerGate._execute` → `ILegalPrimitive.permits` |
| isExecutable | `false` by default; true only after EXECUTED |

The on-chain `LegConsumerGate` is a **stub**: access control (`_isRatifier`) and the effect
(`_execute`) are abstract. Concrete deployments supply both and disclose their upgrade model
per RFC-007 INT-003.

---

## §9 Open Items

| ID | Item | Dependency |
|---|---|---|
| LCO-001 | Identity layer for ratifier authorization (LC-005) | external IDENTITY/DID layer |
| LCO-002 | Sibyl registry runtime for LC-003(b) | OI-002 (`drafts/sibyl/sibyl-registry.md`) |
| LCO-003 | LBRY `proof_ref` → on-chain commitment encoding | PR-002 |
| LCO-004 | Rework/`HL` queue semantics (where held dispositions go) | this spec, future minor |
| LCO-005 | `TR` (compelled transfer) handling — distinct from `PR` | RFC-007 §13 Q3 |
| LCO-006 | Dual-control policy profile format (LC-007) | deployment profile schema |

---

## §10 References

| Document | Role |
|---|---|
| `lex-exe/leg-origin-adapter.md` | Producer counterpart; OA-001–013 |
| `SBL-9/lbry-legal-attestation.md` | LBRY record re-validated in LC-003 |
| `drafts/sibyl/sibyl-registry.md` | Sibyl active-status check (LC-003b) |
| `primitives/src/interfaces/IArrowLegConsumer.sol` | On-chain consumer surface |
| `primitives/src/core/LegConsumerGate.sol` | On-chain fail-closed gate stub |
| `decisions/ADR-0001-stack-boundaries-v2.md` | Fail-closed + AI-free runtime mandate |
| RFC-007 §3 (DPL-004) | No silent lifecycle mutation on wall-clock (LC-008) |
