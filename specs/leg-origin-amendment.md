# Arrow Protocol — LEG Origin Amendment

**Draft Specification v0.2** — reconciled to Arrow Protocol v1.6.0 (2026-06-14)

| Field | Value |
|---|---|
| Document | `specs/leg-origin-amendment.md` |
| Status | **Draft** |
| Extends | `docs/protocol/arrow-protocol.md`, `docs/protocol/validation-rules.md` |
| Related | Arrow `registries/verification_regimes.json`, Arrow `registries/compliance_extensions.json`, Arrow `docs/architecture/legal-policy-primitives-v1.3.1.md` |
| Protocol version impact | Promoted in v1.5-draft without changing the canonical message string. |
| Canonical message string | **Unchanged.** No new field. |
| New validation prefix | `LEG-004` onward (`LEG-001` through `LEG-003` are existing rules). |
| New pipeline step | Step 5b — LEG Action Derivation Gate, after FB-001. |

> **v0.2 reconciliation (Arrow v1.6.0).** Field 4 is **Action** (was Policy; only Origin `PCY` keeps the display name "Policy"). LEG family renumbered to match Arrow v1.6.0: old LEG-006 (subtype presence) → **MSG-006** (schema constraint); old LEG-007 (range gate) → **LEG-006** (Action Derivation Gate); old LEG-008 (SUPRA force-BL) → **LEG-007**; old LEG-009 (LEG_TEXT/PREC warning) → **LEG-008**. Reason code `LEG_POLICY_RANGE_VIOLATION` → **`LEG_ACTION_RANGE_VIOLATION`**. The Finality=0 row is governed protocol-wide by **SEM-013**. Filename version bump (→ `-v0.2.md`) and reference updates are deferred to the v1.6 housekeeping pass.

---

## 1. Purpose and Scope

The pre-v1.5 `origin = LEG` treatment correctly identified a legal system as an asserting domain and established that a `jurisdiction` field is required. It did not, however, specify:

- **What kind of legal authority** is making the assertion (a court order is structurally different from a statute or a UN resolution).
- **What Action outcomes** a LEG-origin event is permitted to produce given that authority type and finality.
- **What Confidence floor** applies to LEG-origin events, given that an unverified or untraceable legal signal is semantically inert as a policy trigger.
- **Whether Action is claimed or derived** for LEG-origin events — the deeper architectural question this amendment begins to answer.

This amendment addresses all four. It does not change the canonical six-field string, does not modify any existing rule, and does not affect non-LEG events.

---

## 2. Architectural Statement — Claim vs. Output

### 2.1 The current model

Before the v1.5 LEG gate, the Action field (position 4 of the canonical string `Class:Origin:State:Action:Finality:Confidence`) was **claimed by the submitter** and **validated by the pipeline**. The pipeline checked that the claimed Action was semantically consistent with all other fields and with external trust flags. It did not recompute Action from scratch; it accepted or rejected a claim.

Three existing rules already behave as **forced outputs** rather than claim validations:

| Rule | Trigger | Effect |
|---|---|---|
| SEM-016 | `chain_trust_flag == 0` | Forces Action to `BL`; non-overridable |
| BRG-001 | `bridge_trust_flag == 0` | Forces Action to `BL`; non-overridable |
| SAN-001 | `sanctions_aml.result = hit` | Forces Action to `BL` or `HL`; non-overridable |

In each case, an external datum overrides the submitted claim. The canonical string's Action field is accepted as output of this forcing function, not as the original submitter's claim. The multi-lane compilation model — where `arrow_code` is always the *output* of all lanes passing, not a claim that is then validated — is the natural terminus of this pattern.

### 2.2 What this amendment adds

This amendment introduces the **LEG Action Derivation Gate** (Step 5b in the v1.5 validation pipeline, after FB-001). For `origin = LEG` events, the pipeline derives a **permitted action range** from `leg_subtype + Finality + Confidence` and validates that the submitted Action falls within it. A submission outside the derived range is either forced to the most restrictive permitted value or rejected outright, depending on the violation type.

This makes LEG-origin Action **partially derived**, not purely asserted. The submitted Action is still present in the canonical string, but its validity is now constrained by a structural derivation, not only by syntax and pre-existing SEM rules.

This is the first formal range-based derivation in the protocol. Trust-flag overrides (SEM-016, BRG-001, SAN-001) are binary; the LEG gate is range-based, because different subtypes and finality states permit different policy sets. The canonical string is unchanged; only the validation logic is new.

### 2.3 What remains future work

This amendment deliberately scopes to LEG. Extension of the derivation model to other origins, and a formal multi-lane compilation architecture in which `arrow_code` is always an output rather than an asserted claim, are named here as the protocol's north star but are not built in this pass.

---

## 3. `leg_subtype` — New Required Field for LEG-Origin Events

### 3.1 Field definition

A new required field `leg_subtype` is added to the Arrow Event Object when `origin = LEG`.

| Attribute | Value |
|---|---|
| Field name | `leg_subtype` |
| Required when | `origin = LEG` |
| Type | `string` — controlled vocabulary (see §3.2) |
| Position | Event Object required field (not in the canonical string) |
| Validation | MSG-004 analog: value MUST exist in the `leg_subtype` vocabulary (MSG-006) |

`leg_subtype` is an Event Object field, not a seventh field in the canonical string. It operates the same way as `chain_id` or `bridge_id` — an Event Object field that triggers additional pipeline validation.

### 3.2 Controlled vocabulary — five subtypes

| Code | Name | Authority type | Coercive backing |
|---|---|---|---|
| `JUD` | Judicial Order | Court, tribunal, judge, arbitral body with judicial authority, administrative court | Direct state force: contempt, asset seizure, imprisonment, enforcement officers |
| `ADM` | Administrative / Regulatory Order | Regulatory agency, ministry, executive enforcement body, independent supervisory authority | Regulatory sanctions, license revocation, fines; state force ultimately available |
| `SUPRA` | Supranational Instrument | UN Security Council resolution, EU binding regulation or directive, FATF blacklist, multilateral enforcement body | Varies: UNSC has enforcement mechanisms; EU has treaty-based enforcement; FATF has systemic exclusion force |
| `LEG_TEXT` | Legislative / Statutory Norm | Legislature, parliament, executive decree, regulatory rulemaking | Indirect: the norm sets the framework; execution requires a JUD or ADM order applying it |
| `PREC` | Legal Precedent | Court of record (binding in common law systems), arbitral award with precedential effect | Binding force in common-law systems; persuasive authority elsewhere; no direct enforcement mechanism |

**Design rationale.** Five is the ceiling. `JUD` and `ADM` carry direct coercive force and produce executing action. `SUPRA` is the cross-border and international law entry point — it absorbs UNCITRAL, Lex Mercatoria, EU directives, and UN sanctions without requiring country-level resolution inside Arrow. `LEG_TEXT` captures the norm itself — important as contextual framing but never sufficient alone to authorize or block. `PREC` acknowledges the common law world without overstating its reach in civil-law jurisdictions.

**Note on religious and customary orders.** Under the v1.5 LEG architecture, religious courts (Sharia courts, rabbinical courts, ecclesiastical courts) with state-delegated authority in personal-status matters fall under `JUD` in the jurisdiction where they exercise that authority. A non-state-recognized religious or customary adjudication body falls under `LEG_TEXT` or is out of scope for Arrow; its output belongs in LexCodex/LexExe, not in a direct LEG event.

---

## 4. Action Derivation — The LEG Gate

### 4.1 Derivation table

The following table defines the permitted Action range for each `leg_subtype` and `Finality` combination. The submitter claims a Action; the pipeline confirms the claim falls within the permitted range. Claims outside the range are handled per §4.2.

| `leg_subtype` | `Finality` | Permitted Action values | Notes |
|---|---|---|---|
| `JUD` | `1` | `PR`, `BL`, `TR`, `CL` | Final judicial order. Direct execution permitted. |
| `JUD` | `0` | `HL`, `RW` | Order exists but is interim, contested, or under appeal. No execution. |
| `ADM` | `1` | `PR`, `BL`, `HL`, `CL` | Final regulatory action. `TR` excluded — administrative orders do not trigger autonomous execution. |
| `ADM` | `0` | `HL`, `RW` | Regulatory action is provisional, under challenge, or in comment period. |
| `SUPRA` | `1` + `reason_code = SA` | `BL` only | Supranational sanctions instrument. Non-overridable. Mirrors SAN-001 and BRG-001 structure. |
| `SUPRA` | `1` + other reason codes | `PR`, `HL` | Supranational authorization or non-sanctions binding instrument. `TR`, `BL`, `CL` excluded by default. |
| `SUPRA` | `0` | `HL`, `RW` | Supranational instrument is non-final, a recommendation, or in transposition. |
| `LEG_TEXT` | any | `HL`, `RW` | The norm itself; cannot alone authorize or block. Must be paired with `JUD` or `ADM` in `related_events` to produce an executing action. |
| `PREC` | any | `RW` | Informational and directional only. Signals law's direction without executing it. |

### 4.2 Violation handling

| Violation type | Pipeline behavior |
|---|---|
| Claimed Action is in a **stricter subset** of permitted range (e.g., `BL` claimed where `{PR, BL, TR, CL}` permitted) | **Accept.** The claim is within range; extra restriction is valid. |
| Claimed Action is **outside** permitted range but a more restrictive permitted value exists (e.g., `PR` claimed where only `{HL, RW}` permitted) | **Reject** with `LEG-006` (`LEG_ACTION_RANGE_VIOLATION`). Do not silently downgrade — the submitter's intent must be explicitly re-stated. |
| `SUPRA` + `reason_code = SA` + `Finality = 1` + any Action other than `BL` | **Force BL.** Non-overridable. Same structural position as BRG-001 and SAN-001. Record violation `LEG-007`. |
| `leg_subtype` absent when `origin = LEG` | **Reject** with `MSG-006`. |

**Design note on silent downgrade.** The draft considered silently downgrading a `PR` claim to `HL` when only `{HL, RW}` is permitted, paralleling how SAN-001 forces BL. The decision was to reject instead, for three reasons: (1) LEG-origin events involve legal authority — a system that silently changes PR to HL on a judicial order is misrepresenting the submitter's legal claim, which has audit implications; (2) the submitter may have submitted `PR` because they misread finality — rejection forces them to re-evaluate; (3) only `SUPRA + SA` uses the force-override model, because sanctions instruments have the same unambiguous override logic as trust flags. All other LEG violations are explicit rejections.

---

## 5. Confidence Model for LEG-Origin Events

### 5.1 The floor problem

A legal signal at Confidence E is semantically inert. It means the assertion is unverified to the degree that no weight can be placed on it for policy purposes. This is unlike chain-origin events where E is a valid (if weak) classification of a genuine observation. For LEG-origin events, E means: we cannot confirm that a legal authority issued this instruction. An unconfirmable legal claim cannot validly trigger any policy response — not even a block, because a false block based on a fabricated legal claim is itself a policy failure.

This is a policy constraint, not a categorical impossibility in the physical sense. An implementation might receive a Confidence E LEG event and want to forward it for human review. The protocol accommodates this through `RW` (review warranted). What it prohibits is any LEG-E event producing an executing action.

This constraint is expressed as a new rule (LEG-004), distinct from SEM-009. SEM-009 covers `Confidence E` + `Action PR or TR` for all origins. LEG-004 is LEG-specific and extends the prohibition to `BL`, `HL`, and `CL` as well, leaving only `RW` as a valid Action for Confidence E LEG events.

### 5.2 Confidence mapping by subtype

The table below shows the effective Confidence range for each subtype. A manual
or institutionally reviewed legal attestation caps at **C** under
`legal_attestation`. A court, arbitration, or appellate panel is a
**`legal_authority_panel`** fact on the LEG authority/finality axis; it may be
relevant to legal effect, appeal posture, and finality, but it does not by
itself raise Confidence.

Confidence **B** requires a legal-oracle proof path:
`structured_legal_attestation` for a single Sibyl/LBRY-style structured proof, or
`multiple_legal_attestation` for independent multi-Sibyl corroboration. B is a
ceiling, not a default.

| `leg_subtype` | Minimum Confidence | Maximum Confidence (manual legal attestation) | Maximum Confidence (structured or multiple legal attestation) | Notes |
|---|---|---|---|---|
| `JUD` | C | C | B | A 3-judge or multi-arbitrator panel is `legal_authority_panel`; B still requires structured or multiple proof. |
| `ADM` | C | C | B | B requires structured or multiple proof, not only a multi-officer agency act. |
| `SUPRA` | C | C | B | B requires structured or multiple proof, not only a multi-signatory instrument. |
| `LEG_TEXT` | D | C | C | D permitted — informational use only; Action MUST be HL or RW |
| `PREC` | D | C | C | D permitted — persuasive authority contexts; Action MUST be RW |

**Confidence A is not available for any live LEG subtype** under current verification regimes. `legal_native_attestation` is reserved as a hypothetical or experimental regime for a legal system that emits protocol-verifiable legal acts directly into a Sibyl or equivalent cryptographic layer. No current Origin depends on it.

### 5.3 The `structured_legal_attestation` verification regime

This amendment formalizes `structured_legal_attestation` as a legal-oracle proof path.

**Purpose:** Makes a single-source legal attestation **machine-verifiable** — structured metadata plus a cryptographic anchor that permits independent checking of the attestation's authenticity. It may reach a **B ceiling** when the artifact requirements and accountability requirements are satisfied. The ceiling is not the default.

**Required artifacts for `structured_legal_attestation`:**

1. **Structured attestation record** — not a document reference alone; a machine-readable structured record identifying: issuing authority (with a registered identity reference, ideally a DID), case or decision identifier, legal basis (subtype-appropriate: case number, regulation code, resolution number), scope of effect, effective date, expiry date (if applicable), and jurisdiction in Arrow format.
2. **Cryptographic anchor** — at minimum, a SHA-256 or keccak256 hash of the source legal document or decision; at preferred level, a digital signature from the issuing authority's registered public key, or an OpenTimestamps / on-chain calldata commitment anchoring the document hash at a specific block.
3. **Oracle identity reference** — an `asserted_by` field resolving to a registered legal oracle identity (DID or equivalent), accountable for the structured attestation.

| Artifacts present | Maximum Confidence (single source) |
|---|---|
| (1) + (2) with digital signature + (3) | B ceiling |
| (1) + (2) with hash anchor only + (3) | C |
| (1) + (3) only, no cryptographic anchor | C (same as `legal_attestation`) |
| (1) only, no anchor, no oracle identity | D |
| None — unstructured reference | E — rejected per LEG-004 unless Action = RW |

`multiple_legal_attestation` is the independent multi-Sibyl evidence path. It also has a **B ceiling** for now. It may graduate later only if a future RFC defines the operational, cryptographic, and governance conditions under which a multi-Sibyl network provides stronger guarantees.

**Regime entry (draft):**

```json
{
  "code": "structured_legal_attestation",
  "name": "Structured Legal Oracle",
  "description": "A legal oracle or registered legal authority attests to a legal fact, order, or instrument through a structured machine-readable record with cryptographic anchoring. Distinct from legal_attestation in that it requires structured metadata and a cryptographic artifact, enabling machine verification of the attestation's provenance. Single oracle: maximum Confidence B ceiling, not default. Confidence A: future legal_native_attestation or equivalent RFC.",
  "max_confidence": "B",
  "classification_bound": false,
  "classification_bound_notes": "Legal content attestation, not on-chain output classification.",
  "required_artifacts": [
    "Structured attestation record with authority DID, decision identifier, legal basis, scope, effective date",
    "Cryptographic anchor: digital signature from issuing authority key, OR SHA-256/keccak256 document hash with chain or OTS commitment",
    "asserted_by resolving to a registered legal oracle identity"
  ],
  "typical_origins": ["LEG"],
  "added_version": "0.1",
  "status": "draft"
}
```

---

## 6. Validation Rules (`LEG-004` onward)

Existing rules `LEG-001`, `LEG-002`, and `LEG-003` are unchanged.

| Rule | Severity | Constraint |
|---|---|---|
| **LEG-004** | Fatal | If `origin = LEG` and `Confidence = E`, Action MUST be `RW`. All other Action values (`PR`, `TR`, `BL`, `HL`, `CL`) are prohibited. An unverifiable legal signal cannot authorize, block, hold, or close. Implementations MAY escalate this to a full rejection at the deployment policy layer (Step 13). |
| **LEG-005** | Fatal | If `origin = LEG` and `Confidence = D`, `leg_subtype` MUST be `LEG_TEXT` or `PREC`. Confidence D is not permitted for `JUD`, `ADM`, or `SUPRA` subtypes. A judicial or administrative order that cannot be verified to Confidence C cannot produce an executing action. |
| **MSG-006** | Fatal | If `origin = LEG`, `leg_subtype` MUST be present and MUST be one of `{JUD, ADM, SUPRA, LEG_TEXT, PREC}`. Absence or unknown value MUST be rejected. |
| **LEG-006** | Fatal | If `origin = LEG`, the submitted Action MUST fall within the permitted range for the `leg_subtype` + `Finality` combination defined in §4.1. A claim outside the permitted range MUST be rejected with reason code `LEG_ACTION_RANGE_VIOLATION`. Silent downgrade is prohibited. |
| **LEG-007** | Fatal (force) | If `origin = LEG`, `leg_subtype = SUPRA`, `Finality = 1`, and `reason_code = SA`, Action MUST be `BL` regardless of all other fields. Non-overridable. Same structural position as BRG-001 and SAN-001. Any other Action claim is rejected and `BL` is applied. |
| **LEG-008** | Warning | If `origin = LEG`, `leg_subtype = LEG_TEXT` or `PREC`, and `Finality = 1`, the validator SHOULD emit a warning: a legislative norm or precedent with `Finality = 1` is unusual. A `LEG_TEXT` event flagged as final likely represents a superseding enactment and SHOULD reference `related_events` identifying the prior norm it supersedes. |

### 6.1 Relationship to existing rules

| Existing rule | Interaction |
|---|---|
| **SEM-009** | `Confidence E MUST NOT pair with PR or TR` — still applies to all origins. LEG-004 is the LEG-specific extension: E + BL, HL, and CL are also prohibited for LEG. LEG-004 supersedes SEM-009 for LEG-origin events in the E-confidence case (more restrictive). |
| **SEM-015** | Confidence ceiling per verification regime — still applies. Manual legal claims under `legal_attestation` sit at the **C** ceiling; `structured_legal_attestation` and `multiple_legal_attestation` have a **B** ceiling. A is not available for any live LEG subtype. |
| **LEG-002** | Jurisdiction consistency in `extensions.legal_instrument` — unchanged. LEG-006 is independent of LEG-002; core LEG derivation runs at Step 5b. |
| **LEG-003** | `enforceability_status = void/disputed` blocks PR/TR — unchanged. LEG-006 subsumes this for subtype-level policy derivation, but LEG-003 still applies to `extensions.legal_instrument` at Step 14. They are not redundant: LEG-006 operates at the subtype+finality level; LEG-003 operates at the instrument enforceability level. |
| **SAN-001** | Sanctions hit forces BL — unchanged. LEG-007 is the LEG-origin analog for SUPRA+SA: same structural position, origin-scoped rather than extension-scoped. Both run and the more restrictive output governs (per CMX-002 analog). |
| **CMX-001** | Extensions cannot override `arrow_code` — unchanged. LEG-006 and LEG-007 operate before Step 14; they determine the valid action range before the Compliance Module runs, and CMX-001 continues to prevent extension-level override after. |

---

## 7. Pipeline Amendment

Step 5b is inserted into the v1.5 validation pipeline immediately after Step 5a (FB-001) and before Step 6 (SEM-017 Classification Binding). It runs only when `origin = LEG`.

```
Step 1    Parse canonical message string
          → Reject if MSG-001 to MSG-005 fail

Step 2    Resolve Origin to registry entry
          → Reject if origin unregistered

Step 3    Check chain_trust_flag (SEM-016)
          → Force Action BL if 0; reject PR/TR/HL/RW/CL

Step 3a   If bridge_id present: resolve Bridge Registry entry
          → Reject if bridge_id unregistered

Step 3b   If bridge_trust_flag == 0: apply BRG-001
          → Force Action BL; reject all non-BL policies

Step 4    Verify verification_regime is declared
          → Reject if null

Step 5    Apply semantic constraints SEM-001 to SEM-014
          → Reject if any constraint fails

── NEW ──────────────────────────────────────────────────────────────────────

Step 5b   If origin = LEG: apply LEG Action Derivation Gate
          → MSG-006: reject if leg_subtype absent or unregistered
          → LEG-004: reject if Confidence = E and Action ≠ RW
          → LEG-005: reject if Confidence = D and subtype ∉ {LEG_TEXT, PREC}
          → LEG-007: force Action BL if subtype = SUPRA + reason_code = SA + Finality = 1
          → LEG-006: derive permitted Action range from subtype + Finality;
                     reject if submitted Action outside range
          → LEG-008: emit warning if subtype ∈ {LEG_TEXT, PREC} and Finality = 1

─────────────────────────────────────────────────────────────────────────────

Step 6    Apply SEM-017 — Proof Classification Binding

Step 7    Apply MSG-005 — Classification Binding Method Registry Membership

Step 8    Apply SEM-019 — Economic Consistency Binding

Step 9    Apply BRG-003 and BRG-006

Step 10   Apply SEM-015 — Regime / Confidence Ceiling

Step 11   Multi-source: compute numeric confidence; evaluate divergence

Step 12   Apply SEM-009 — Protocol confidence floor

Step 13   Pass to deployment policy layer

Step 14   If extensions.* present: apply Compliance Module
          (TRV-001–008, SAN-001–003, LEG-002–003, CMX-001–002)
```

Step 5b runs before the confidence ceiling check (Step 10) because LEG-005 sets a subtype-specific confidence floor that constrains what the ceiling check receives. It also runs before the Compliance Module (Step 14) because the derived action range must be established before extension blocks are evaluated.

---

## 8. Event Object Amendments

One new required field when `origin = LEG`:

| Field | Type | Required | Notes |
|---|---|---|---|
| `leg_subtype` | string (controlled) | Yes, when `origin = LEG` | One of `{JUD, ADM, SUPRA, LEG_TEXT, PREC}`. |

The following existing fields gain specific obligations under this amendment:

| Field | Existing? | LEG-specific obligation |
|---|---|---|
| `jurisdiction` | Yes (already required for LEG) | Unchanged. |
| `reason_code` | Yes | For `leg_subtype = SUPRA`: `reason_code = SA` triggers LEG-007 (BL force). Other SUPRA reason codes determine the action range per §4.1. |
| `proof_type` | Yes | For `structured_legal_attestation` regime: `proof_type` SHOULD be `notarial_attestation` or an equivalent structured legal proof type. |
| `proof_ref` | Yes | For `structured_legal_attestation` regime: MUST be a verifiable reference to the structured attestation record. |
| `asserted_by` | Recommended | For `structured_legal_attestation` regime: MUST be present and MUST resolve to a registered legal oracle identity (LEG-003 analog). |

Example Event Object fragment — final judicial order, structured oracle:

```json
{
  "arrow_code": "2:LEG:FN:PR:1:C",
  "origin": "LEG",
  "leg_subtype": "JUD",
  "jurisdiction": "MX-FED",
  "reason_code": "CO",
  "proof_type": "notarial_attestation",
  "proof_ref": "doc://lexexe/order/2026-MX-FED-0481",
  "asserted_by": "did:arrow:legal-oracle:cdmx-notaria-42",
  "verification_regime": "structured_legal_attestation",
  "finality": 1,
  "state": "FN",
  "confidence": "C",
  "extensions": {
    "legal_instrument": {
      "schema_version": "1.0",
      "instrument_type": "court_order",
      "governing_law": "MX-FED",
      "jurisdiction": "MX-FED",
      "enforceability_status": "enforceable",
      "execution_date": "2026-05-24"
    }
  }
}
```

Pipeline result: Step 5b — `JUD + Finality=1 → {PR, BL, TR, CL}`; `PR` claimed → within range → pass. `Confidence C + structured_legal_attestation` (single source) → below the B ceiling → pass (Step 10). LEG-004/005 → not triggered (C is at the JUD floor; not E or D). LEG-006 → pass. A fully qualified structured or multiple attestation could claim `B`.

Example — supranational sanctions, non-overridable:

```json
{
  "arrow_code": "3:LEG:FN:BL:1:C",
  "origin": "LEG",
  "leg_subtype": "SUPRA",
  "jurisdiction": "UN",
  "reason_code": "SA",
  "verification_regime": "legal_attestation",
  "finality": 1,
  "state": "FN",
  "confidence": "C"
}
```

Pipeline result: Step 5b — LEG-007 fires (`SUPRA + SA + Finality=1 → force BL`). `BL` is submitted — matches forced output → pass (no rejection needed; forced and claimed coincide). Note: if `PR` were submitted, LEG-007 would force `BL` and reject the `PR` claim.

Example — legislative norm (contextual framing):

```json
{
  "arrow_code": "1:LEG:FN:RW:0:C",
  "origin": "LEG",
  "leg_subtype": "LEG_TEXT",
  "jurisdiction": "MX",
  "reason_code": "LR",
  "verification_regime": "legal_attestation",
  "finality": 0,
  "state": "PD",
  "confidence": "C"
}
```

Pipeline result: `LEG_TEXT + any Finality → {HL, RW}`. `RW` claimed → within range → pass. Action `RW` means: downstream systems should review before acting. Used to flag that an event touches on a regulated domain without asserting an order.

---

## 9. Interaction Notes

**POLICY.** The LEG Action Derivation Gate (Step 5b) is the first protocol-level mechanism that constrains the Action field by a derivation from content fields rather than purely from field-to-field semantic rules. For LEG events, Action is no longer purely asserted — it is verified against a derived range. This is the intended direction of the protocol and does not contradict CMX-001; CMX-001 governs extension blocks at Step 14, whereas the derivation gate operates at Step 5b on the base event.

**FINALITY.** `Finality` remains the Protocol's binary lifecycle flag. LEG-006 uses it as a derivation input, not as a substitute. A `JUD` event with `Finality = 0` does not mean the judicial order is fake — it means the order's legal effect is not yet final (e.g., subject to appeal, interim only, or pending service). This is orthogonal to the event's `Finality = 0` meaning in the Arrow lifecycle sense; the two happen to be aligned here by design, but the semantics originate independently.

**CONFIDENCE.** The confidence floor for LEG (LEG-004/005) and the confidence ceiling (SEM-015) together form a bounded confidence window for each subtype. For `JUD`: the manual-attestation window is `[C, C]`; `structured_legal_attestation` or `multiple_legal_attestation` widens it to `[C, B]` when the required artifacts and accountability conditions are met. For `LEG_TEXT`/`PREC`: the window is `[D, C]`. A `legal_authority_panel` may matter to authority and finality, but it does not by itself widen the confidence window. Confidence is still a deployment policy decision within the window (SEM-011 is unchanged); the protocol only sets the bounds.

**IDENTITY / DID.** The `structured_legal_attestation` regime formally requires `asserted_by` to resolve to a registered oracle identity. This is the first LEG-specific hook into the IDENTITY layer. Resolution and verification remain fully delegated to the IDENTITY layer; the LEG amendment only states that the field is required and must resolve.

**BRIDGE.** LEG events crossing domains (e.g., a Mexican court order affecting an Ethereum-anchored asset) may carry both `leg_subtype` and `bridge_id`. BRG-001 runs before Step 5b; if `bridge_trust_flag == 0`, BL is forced and the LEG gate is not reached. If the bridge is trusted, Step 5b runs on the LEG fields and derives the action range independently of the bridge result. The more restrictive of the two governs per CMX-002 analog.

**LEXCODEX / LEXEXE.** The LEG amendment deliberately holds the Arrow-LEG surface to five subtypes and tight policy constraints. All jurisdiction-specific legal reasoning, instrument lifecycle logic (the Prime Verbs: Create, Modify, Transmit, Extinguish), domain-specific statutory interpretation, and Obligation/Right classification live in LexCodex and LexExe. A LexExe engine evaluates a legal context, determines the appropriate `leg_subtype`, confidence regime, and policy claim, and emits the Arrow event. Arrow validates the output; it does not replicate the reasoning.

---

## 10. Non-Goals and Anti-Patterns

- **LEG is not a legal advice system.** `leg_subtype = JUD` does not mean Arrow has determined a court order is valid. It means a submitter has claimed one exists, and Arrow has validated that the claim is structurally consistent with what a final judicial order should produce.
- **Do not submit LEG_TEXT or PREC expecting executing action.** These subtypes are for contextual framing — flagging that an event touches on a legislative domain or that precedent exists. Actual execution always requires a `JUD` or `ADM` event in `related_events`.
- **Do not use SUPRA to abstract away a country-level order.** A Mexican CNBV administrative order is `ADM` with `jurisdiction = MX`, not `SUPRA`. `SUPRA` is for genuinely supranational instruments: UN Security Council resolutions, EU binding acts, multilateral enforcement mechanisms.
- **E-confidence LEG events should not be routed to production.** LEG-004 permits `RW` at E-confidence as a safety valve for human review queues; it is not intended to carry legal weight. If a system is regularly producing E-confidence LEG events, the attestation pipeline is broken.
- **Silent downgrade is prohibited.** A rejected LEG-006 event must be re-submitted with a corrected Action claim. Systems that silently mutate `arrow_code` are in violation of CMX-001 and the foundational principle that `arrow_code` is the Semantic Engine's authoritative output.
- **`leg_subtype` is not a legal classification of the underlying legal system.** That is the LEGAL_SYSTEM primitive (LexCodex domain). `leg_subtype` classifies only the authority type of the LEG-origin assertion, not the legal tradition, family, or order it comes from.

---

## 11. New Reason Codes (Proposed)

For promotion, three new reason codes are proposed for addition to `reason_codes.json`:

| Code | Applies to | Meaning |
|---|---|---|
| `LEG_ACTION_RANGE_VIOLATION` | Error class for LEG-006 rejections | Submitted Action is outside the permitted range for the declared `leg_subtype` and `Finality`. |
| `LEG_SUPRA_SANCTIONS_FORCE` | Annotation for LEG-007 forced BL events | `SUPRA + SA + Finality=1` forced Action to BL. Non-overridable. |
| `LEG_SUBTYPE_ABSENT` | Error class for MSG-006 rejections | `origin = LEG` event submitted without `leg_subtype` field. |

---

## 12. Implementation Order

1. **Lock vocabulary and derivation table** — five subtypes, action ranges per §4.1, confidence mapping per §5.2. *(This draft.)*
2. **Register legal-oracle proof paths** — supersede the earlier structured-oracle term with `structured_legal_attestation`; model independent multi-Sibyl evidence as `multiple_legal_attestation`; keep `legal_authority_panel` on the LEG authority/finality axis.
3. **Define `leg_subtype` controlled vocabulary** — add to `compliance_extensions.json` at promotion (consistent with how `enforceability_status` and `instrument_type` are registered there).
4. **Draft validation rule implementations** (LEG-004 through LEG-008) — pseudo-code or Rust analogs matching the `pipeline.rs` pattern.
5. **Internal review** — focus on the derivation table edge cases (SUPRA + non-SA + Finality=1; LEG_TEXT + Finality=1 warning; D-confidence ADM edge).
6. **Promotion:** completed in the v1.5-draft package by documenting `leg_subtype` and `leg_jurisdiction`, adding Step 5b and LEG-004 through LEG-013, amending `verification_regimes.json`, `compliance_extensions.json`, and `reason_codes.json`, and wiring the first implementation pass in `pipeline.rs`.
7. **LexExe interface spec** — separate document defining how a LexExe engine emits Arrow LEG events, which subtypes it produces under which conditions, and how it populates `asserted_by` and `proof_ref`. Not an Arrow protocol document — a LexCodex/LexExe interface spec.

---

## Appendix A — Derivation Table (Machine-Readable Summary)

```json
{
  "leg_action_derivation": {
    "JUD":      { "finality_1": ["PR","BL","TR","CL"], "finality_0": ["HL","RW"] },
    "ADM":      { "finality_1": ["PR","BL","HL","CL"], "finality_0": ["HL","RW"] },
    "SUPRA_SA": { "finality_1": ["BL"],                "finality_0": ["HL","RW"] },
    "SUPRA":    { "finality_1": ["PR","HL"],            "finality_0": ["HL","RW"] },
    "LEG_TEXT": { "any":        ["HL","RW"] },
    "PREC":     { "any":        ["RW"] }
  },
  "leg_confidence_floor": {
    "JUD":      "C",
    "ADM":      "C",
    "SUPRA":    "C",
    "LEG_TEXT": "D",
    "PREC":     "D"
  },
  "leg_confidence_ceiling_manual_attestation": {
    "_note": "legal_attestation caps manual or institutionally reviewed LEG assertions here.",
    "JUD":      "C",
    "ADM":      "C",
    "SUPRA":    "C",
    "LEG_TEXT": "C",
    "PREC":     "C"
  },
  "leg_confidence_ceiling_structured_or_multiple": {
    "_note": "structured_legal_attestation or multiple_legal_attestation; B is a ceiling, not a default.",
    "JUD":      "B",
    "ADM":      "B",
    "SUPRA":    "B",
    "LEG_TEXT": "C",
    "PREC":     "C"
  }
}
```

---

*End of draft v0.2 (reconciled to Arrow Protocol v1.6.0).*
