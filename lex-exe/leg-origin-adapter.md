# LEG Origin Adapter — Interface Specification

**Version:** v0.2 — reconciled to Arrow Protocol v1.6.0  
**Status:** Draft  
**Date:** 2026-06-07 (rev. 2026-06-10)  
**Track:** Tarquin / Lex.exe  
**Document:** `lex-exe/leg-origin-adapter.md`  
**Depends on:** `arrow-protocol` v1.6.0; `specs/leg-origin-amendment.md`;
`specs/legal-system-foundation.md`; `specs/leg-jurisdiction-addendum.md`

> **v0.2 revision (2026-06-14) — reconciled to Arrow Protocol v1.6.0.** Completes the field-4
> rewrite the v0.1.1 note anticipated: all "Policy" / "policy proposal" wording is now **Action** /
> **Action proposal** (only Origin `PCY` keeps the display name "Policy"). LEG numbering matches
> Arrow v1.6.0: derivation gate **LEG-006**, SUPRA sanctions force-BL **LEG-007**, subtype schema
> **MSG-006**; jurisdiction **LEG-009 / LEG-010**; the subnational-forum hook is **LEG-013-EXT**
> (Compliance Module, Step 14). Arrow rule IDs are permanent from v1.6.0; the shared surface pins
> `(rule_id, semantic_slug)` pairs so this class of drift cannot recur.

---

## §1 Scope

This document defines the obligations of Lex.exe when emitting an Arrow LEG-origin event via
the `arrow-origin-leg` adapter. It is the **producer contract** for the Tarquin → Arrow
binding.

This document does not govern:

- Arrow's internal action derivation (defined in `specs/leg-origin-amendment.md`, Step 5b core LEG-004–LEG-008 + MSG-006; jurisdiction LEG-009/LEG-010; LEG-012-EXT/LEG-013-EXT at Step 14)
- Lex.exe's internal legal reasoning or instrument lifecycle
- The `arrow-consume-leg` consumer (covered in a companion spec)
- The Sibyl attestation protocol (covered in the Sibyl Layer specification)

**Two independent gates; the more restrictive governs.** Lex.exe enforces the Tarquin-side
gate (Lex.Codex `permits()` + the rules in this spec). Arrow enforces its own derivation gate
downstream. Neither gate can override the other; conflicts resolve to the more restrictive
outcome. Lex.exe's action proposal is an input to Arrow, not a mandate.

---

## §2 Definitions

| Term | Definition |
|---|---|
| **Lex.exe** | The Tarquin execution engine; sole authorized emitter via this adapter |
| **LEG event** | An Arrow event with Origin = `LEG` |
| **LBRY record** | A Libri IX structured attestation record produced by a registered Sibyl; the payload that achieves `structured_legal_attestation` verification regime |
| **Sibyl** | A registered legal-oracle identity — a specialized attestation oracle whose Decentralized Identifier (DID) appears in `asserted_by` |
| **Sibyl registry** | The Tarquin registry of recognized Sibyl identities; designed as a profile of the RFC-007 Part B recognized-authority record |
| **Emission** | The act of Lex.exe submitting a classified event to `arrow-origin-leg` |
| **Action proposal** | Lex.exe's proposed Action field value; subject to Arrow Step 5b derivation gate |
| **FN+0** | A legal event in a finalized-but-challengeable state: under appeal, within a clawback window, or subject to active review; represented as Finality = 0 in the classification string |

---

## §3 Prerequisites

All of the following conditions MUST be satisfied before Lex.exe emits. Failure of any
prerequisite is a hard stop; Lex.exe MUST NOT emit and MUST log the blocking condition.

**OA-001 — Lex.Codex `permits()` gate.**  
Lex.Codex MUST NOT have returned `false` from its `permits()` check for the proposed event.
This gate is upstream and non-overridable by Lex.exe. A `false` result terminates emission
immediately; the veto reason is logged.

**OA-002 — Valid LBRY record exists.**  
A LBRY record satisfying all requirements in §7 MUST exist and be retrievable before
emission. Lex.exe MUST NOT construct a `proof_ref` that does not resolve to a current,
valid record.

**OA-003 — Sibyl is registered.**  
The DID in the LBRY record's `attested_by` field MUST be present in the Sibyl registry at
the time of emission. An unregistered Sibyl fails Arrow rules ORG-001/ORG-002 at Step 2;
OA-003 enforces this check on the Tarquin side before submission.

**OA-004 — Target chain is registered (if applicable).**  
If the LEG event references or targets an on-chain asset or state, the relevant chain MUST
be present in Arrow's `chain_registry.json` (Arrow rule CHN-001). Lex.exe MUST resolve the
chain code before constructing the event and MUST NOT emit with an unregistered chain code.

**OA-005 — Human reviewer is assigned.**  
The anticipated Arrow disposition must have a designated human reviewer assigned before
emission, consistent with the `arrow-consume-leg` fail-closed pattern. A LEG event MUST NOT
be emitted into a consumer with no designated reviewer.

---

## §4 Emission Protocol

Lex.exe follows this sequence on every emission. The steps are strictly ordered; no step
may be skipped.

1. Evaluate Lex.Codex `permits()` — abort if false (OA-001).
2. Retrieve and validate the LBRY record (OA-002, §7).
3. Verify Sibyl registration (OA-003).
4. Resolve target chain code if applicable (OA-004).
5. Confirm human-reviewer assignment (OA-005).
6. Determine the six-field classification string per §5.
7. Populate the LEG envelope per §6.
8. Determine the action proposal per §8.
9. Submit to `arrow-origin-leg`.
10. Handle the response per §9.

---

## §5 Classification String

The Arrow six-field string is `Class:Origin:State:Action:Finality:Confidence`.

| Field | Lex.exe obligation |
|---|---|
| **Class** | Determined by the nature of the legal event (e.g., a legal hold or obligation = Class 3 CONDITION; a compliance determination may map to a different class — consult the Arrow class taxonomy). Valid Class values and their LEG-subtype compatibility are defined in Arrow's validation rules (see OI-001 re: LEG-006 completion). Lex.exe MUST NOT propose a Class value that has no valid combination with the chosen `leg_subtype`. |
| **Origin** | Fixed: `LEG`. No other value is permissible from this adapter. |
| **State** | Set to reflect the current legal disposition. Any decision under appeal, subject to challenge, or within an applicable review window MUST be represented as FN with Finality = 0. Do not represent a challengeable order as settled. |
| **Action** | Proposed by Lex.exe per §8; subject to Arrow Step 5b derivation gate. The derived value governs. |
| **Finality** | `0` = settled but challengeable (FN+0); `1` = irreversible under the applicable legal order. |
| **Confidence** | MUST NOT be proposed as `A` (see OA-006). `structured_legal_attestation` has a `B` ceiling per Arrow's `confidence_subprofiles.json`, but B is not the default. Propose `B` only when the LBRY record fully satisfies §7 and the Sibyl registry declares B; propose `C` or `D` for weaker records. Independent multi-Sibyl evidence uses `multiple_legal_attestation`. Arrow rule LEG-004 defines the floor. |

**OA-006 — Confidence A prohibition.**  
Lex.exe MUST NOT propose Confidence `A` for any LEG-origin event. Confidence `A` is reserved
for the `native_canonical` regime. An emission proposing `A` is a malformed event and will be
rejected at Arrow Step 2.

---

## §6 LEG Envelope

The LEG envelope carries the metadata that accompanies the classification string. The
canonical full field set is defined in `specs/leg-origin-amendment.md`. The fields Lex.exe is
directly responsible for populating correctly are:

| Field | Source | Requirement |
|---|---|---|
| `leg_subtype` | Lex.exe determination | MUST be one of: `JUD` \| `ADM` \| `SUPRA` \| `LEG_TEXT` \| `PREC` |
| `leg_jurisdiction` | Lex.exe determination | MUST conform to the jurisdiction object schema in `specs/leg-jurisdiction-addendum.md`; at least one of `country` or `supra` is required; `local` lookup performed where applicable per Arrow rule LEG-013-EXT |
| `asserted_by` | From LBRY record | MUST be the DID of the Sibyl that produced the LBRY record; MUST match `attested_by` in the record; MUST be registered per OA-003 |
| `proof_ref` | Reference to LBRY record | MUST resolve to a retrievable LBRY record satisfying §7 |
| `verification_regime` | Derived from LBRY record quality | MUST accurately reflect the actual observation method; set to `structured_legal_attestation` only when the LBRY record meets all §7 criteria; `institutional_attestation` is the honest fallback for a record with a named issuer but lacking cryptographic anchoring (see OA-007) |

**OA-007 — No false verification regime.**  
Lex.exe MUST NOT set `verification_regime` to `structured_legal_attestation` unless the LBRY
record satisfies all §7 requirements. Upcasting regime to achieve a higher Confidence ceiling
is a protocol violation. The regime field must reflect the actual observation method, not the
theoretical best-case for the issuer.

---

## §7 LBRY Record Requirements

A LBRY record achieves `structured_legal_attestation` regime — with a Confidence `B` ceiling,
not default — when it satisfies all
of the following:

| Requirement | Description |
|---|---|
| **Structured content** | A machine-readable representation of the attested legal fact: instrument reference, `leg_subtype`, jurisdiction, effective date, and operative clauses as applicable |
| **Source reference** | A resolvable pointer to the authoritative source (e.g., court docket entry, official gazette, regulatory register); must be independently verifiable |
| **Sibyl DID** | The registered DID of the attesting Sibyl; must appear in `attested_by` and be present in the Sibyl registry |
| **Cryptographic anchor** | A hash of the structured content and source reference, with a timestamp anchor; Sibyl signing-key signature is strongly recommended |
| **Timestamp** | ISO 8601 timestamp of attestation; MUST NOT be in the future at the time of emission |

A record with a named issuer but no cryptographic anchoring achieves `institutional_attestation`
(also a Confidence `C` ceiling; the anchor distinguishes the *method*, not the single-source
ceiling). A record with neither achieves only lower regimes; Lex.exe MUST evaluate whether
emission serves any purpose at that confidence level given Arrow LEG-004's floor constraint on
LEG-origin events. For `structured_legal_attestation`, Confidence `B` is reachable only when
the structured proof artifacts and Sibyl registry entry justify it and the LBRY signature
verifies. Independent multi-Sibyl evidence uses `multiple_legal_attestation`.

**OA-014 — Confidence B requires verified LBRY signature.**
Lex.exe MUST NOT emit Confidence `B` for a `structured_legal_attestation` LBRY record unless
the record's signature verifies against an active attestation key on the registered Sibyl
identity. A registry entry declaring a `B` ceiling is necessary but not sufficient: the concrete
record must prove its signed structured content at emission time. If signature resolution or
verification fails, Lex.exe MUST either emit at a lower justified Confidence or block emission.

---

## §8 Action Proposal Rules

Lex.exe proposes the Action field value. Arrow's Step 5b gate may derive a different, more
restrictive value; Arrow's derived value governs. Lex.exe's obligations here are:

**OA-008 — Proposals must be within the LEG-006 derivation range.**  
Lex.exe MUST NOT propose a Action value that the LEG-006 derivation table does not permit
for the given `leg_subtype` + Finality + `verification_regime` combination. An out-of-range
proposal will be rejected; OA-008 catches it before submission. (See OI-001: LEG-006
completion is a remaining `arrow-core` priority; Lex.exe must track this table as it
stabilizes.)

**OA-009 — FN+0 default is HL.**  
For any event with Finality = 0, Lex.exe MUST propose Action `HL` (hold for human review)
unless OA-010 or OA-011 applies. `HL` is the safe default for any event whose legal finality
is not established. Escalation from `HL` requires explicit justification.

**OA-010 — SUPRA + SA forces BL.**  
For `leg_subtype = SUPRA` with sanctions applicability (the `SA` reason code defined in Arrow rule
LEG-007), Lex.exe MUST propose Action `BL`. No other proposal is valid for this combination.
This mirrors Arrow LEG-007 as a Tarquin-side enforcement to prevent an unnecessary submission.

**OA-011 — PR elevation conditions are conjunctive.**  
Lex.exe MAY propose Action `PR` (proceed / execute) only when ALL of the following conditions
are satisfied:

1. `verification_regime` is `structured_legal_attestation` — the LBRY record satisfies §7 fully.
2. `proof_ref` resolves to a LBRY record containing attested, explicit authorization for the
   proposed action — not merely a general order or enabling instrument.
3. The Sibyl DID in `asserted_by` is resolvable and currently registered in the Sibyl
   registry.
4. The applicable legal order is not under active appeal or challenge: Finality = 1.
5. Lex.Codex `permits()` has explicitly authorized `PR` for this specific event — not merely
   returned non-false on a default check.

If any one condition is unmet, Lex.exe MUST NOT propose `PR`. It MUST propose `HL` and route
to human review.

> **Design reference (substance-verified 2026-06-11; see `sources/kim-v-dprk/SOURCES.md`):**
> *Kim et al. v. DPRK*, No. 1:25-mc-00527-MMG and related cases 26-MC-00033 / 26-MC-00094
> (S.D.N.Y., Garnett, J.) is the canonical **live instance** of the lifecycle this adapter
> models: DPRK judgment creditors collecting against cryptocurrency assets, with (i) an
> Arbitrum Security Council **emergency freeze** of hack-impacted assets (2026-04, the
> control-plane event Arrow's ARB chain-registry advisory independently records), (ii) a
> N.Y. C.P.L.R. § 5222 **restraining notice** on Arbitrum DAO as garnishee — the judicial
> restraint phase, i.e. a non-executing disposition while turnover remains unadjudicated
> (the FN+0 → `HL` discipline of OA-009), and (iii) procedural lifecycle events (Doc 34,
> 2026-05-01, granting alternative service on the DAO via its governance forum and Security
> Council). **No turnover has been ordered in the documents reviewed, and Doc 34 articulates
> no verification standard for executing an on-chain asset action.** OA-011's conjunctive
> conditions therefore stand on protocol-design merit, calibrated to the turnover phase this
> docket is heading toward — not to a holding it has already issued. The underlying judgment
> is *Han Kim v. DPRK*, 774 F.3d-era D.C. Cir. No. 13-7147 (2014). KelpDAO/Aave are not
> named in the reviewed filings and are dropped from this citation pending MC-005.

---

## §9 Error Handling

**Arrow rejection (structural or validation failure):**

- Lex.exe MUST log the rejection with the Arrow error code and a copy of the submitted event.
- The event is quarantined; no retry without human review and a documented root-cause.
- Any OA-001 through OA-005 failure that was not caught in the pre-submission sequence
  indicates a Lex.exe implementation defect and MUST trigger an alert.

**Arrow action downgrade (Arrow's derived action is more restrictive than proposed):**

- Lex.exe MUST log the original proposal and the Arrow-derived value.
- The consumer receives the derived (more restrictive) action; this is correct behavior.
- **OA-012 — No silent re-escalation.** Lex.exe MUST NOT re-emit the same event with the
  originally proposed action after a downgrade, without explicit human authorization and a
  documented resolution of the condition that caused the downgrade. Escalating around a
  downgrade is equivalent to a protocol violation.

**Arrow acceptance:**

The event passes to `arrow-consume-leg`. Execution is governed by the consumer specification.
Lex.exe takes no further action on the strength of the emitted event.

---

## §10 Consumer Handoff

The `arrow-consume-leg` consumer is fail-closed; receipts are quarantined; a human-ratification
gate governs all execution. Full specification is in the companion LEG Consumer Spec.

Lex.exe's obligations at handoff:

- The accepted event and its LBRY record MUST remain retrievable for the consumer.
- **OA-013 — No autonomous execution.** Lex.exe MUST NOT take any on-chain or off-chain
  action on the strength of the emitted event before `arrow-consume-leg` has received the
  Arrow disposition and a human ratifier has confirmed. There is no autonomous path. The
  runtime is AI-free at this stage.

---

## §11 Open Items

| ID | Item | Dependency |
|---|---|---|
| OI-001 | Complete LEG-006 derivation table in `arrow-core` — the full `leg_subtype` × Finality × regime → valid Action range matrix; OA-008 depends on it | Remaining `arrow-core` coding priority |
| OI-002 | Sibyl registry v0.1 schema — profile of RFC-007 Part B authority record; OA-003 depends on it | Drafted in `drafts/sibyl/sibyl-registry.md`; implementation pending |
| OI-003 | Class-to-subtype compatibility matrix — valid Class values for each `leg_subtype`; referenced in §5 but defined in Arrow validation rules pending LEG-006 completion | OI-001 |
| OI-004 | `EMG` origin events — emergency-authority emissions; not covered by this adapter in its current scope | RFC-007 Part B graduation |
| OI-005 | Multi-Sibyl consensus path toward Confidence `A` — future RFC; not in scope here; do not represent as a current capability | Separate RFC required |
| OI-006 | LEG Consumer Spec — companion document defining `arrow-consume-leg` behavior, quarantine model, and ratification workflow | Next spec artifact after this one |

---

## §12 References

| Document | Role |
|---|---|
| `specs/leg-origin-amendment.md` | Arrow-side LEG rules (LEG-001–LEG-010, MSG-006, LEG-012-EXT/LEG-013-EXT), Step 5b Action Derivation Gate, canonical field set, LEG-006/LEG-007 derivation; authoritative for Arrow-facing obligations |
| `specs/legal-system-foundation.md` | LEGAL_SYSTEM classification (LSY-001–007); `legal_systems.json` registry |
| `specs/leg-jurisdiction-addendum.md` | `leg_jurisdiction` object schema; subnational registry hook (LEG-013-EXT) |
| `decisions/ADR-0001-stack-boundaries-v2.md` | Live repo topology, supply chain, Sibyl layer design, dual-target constraint, Confidence ceiling |
| `validation-rules-v1.6.0.md` | Arrow validation rules (SEM-xxx, CHN-xxx, LEG-xxx); LEG-006 derivation table (pending completion — OI-001) |
| `arrow-consume-leg` crate | Fail-closed consumer counterpart to this adapter |
| `drafts/sibyl/sibyl-registry.md` | Tarquin Sibyl registry profile of RFC-007 Part B recognized authority |
| `SBL-9/lbry-legal-attestation.md` | LBRY structured attestation record used as `proof_ref` |
| *Kim et al. v. DPRK*, 1:25-mc-00527-MMG (S.D.N.Y.) + related | Live-instance reference for the restraint/turnover lifecycle (§8, OA-011) — substance-verified 2026-06-11; no turnover/verification-standard holding yet; see `sources/kim-v-dprk/SOURCES.md` |
