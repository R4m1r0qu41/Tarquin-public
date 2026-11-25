# Arrow Protocol — LEGAL_SYSTEM Primitive

**Draft Specification v0.1**

| Field | Value |
|---|---|
| Document | `specs/legal-system-foundation.md` |
| Status | **Draft** — not promoted, not wired into the live validation pipeline |
| Extends | Arrow `docs/architecture/legal-policy-primitives-v1.3.1.md` |
| Companion files | `registries/drafts/legal_systems.json`, `registries/drafts/legal_systems.schema.json` |
| Protocol version impact | None this pass. Promotion targets a future minor version (provisionally v1.5). |
| Canonical message string | **Unchanged.** LEGAL_SYSTEM does not add a seventh field. |
| Validation prefix | `LSY-` |

---

## 1. Status and Scope

LEGAL_SYSTEM is a semantic classification primitive. It gives Arrow events, policies, attestations, modules, and bridge-governance flows a durable, machine-readable way to reference *the type of legal order or legal tradition relevant to a context*.

It is delivered in two parts, per the agreed structural model:

1. A standalone reference registry — `legal_systems.json` — holding classified legal-system entries that existing references (`jurisdiction`, `extensions.legal_module`, `legal_instrument.governing_law`) can resolve into.
2. A thin, optional event extension block — `extensions.legal_system` — for events or modules that need to assert legal-order context directly. This block is **proposed** in this draft and is not a live protocol field.

**This primitive classifies. It does not determine.** Arrow does not establish what law applies to a matter, does not adjudicate enforceability, and does not produce legal advice. It records *asserted or verified* legal-system metadata. Jurisdiction-specific interpretation remains delegated to LexCodex, Lex.exe, or external legal-policy systems, consistent with `legal-policy-primitives-v1.3.1.md`.

**Out of scope:** statutory text, judicial reasoning, jurisdiction-specific procedure, enforceability determination, choice-of-law resolution, and any ranking of legal orders by legitimacy.

## 2. Relationship to the Existing Arrow Legal Surface

Arrow already carries legal-context structure. LEGAL_SYSTEM slots beside it and does not replace any of it.

| Existing element | What it does | Relationship to LEGAL_SYSTEM |
|---|---|---|
| `origin = LEG` | Marks an event as asserted by a legal system; requires `jurisdiction`. | Unchanged. A `LEG` event MAY additionally carry `extensions.legal_system`. |
| `jurisdiction` field | Records *where / which forum* — ISO 3166, UN M.49, or `PROTOCOL`. | Kept separate and orthogonal. See §4 and LSY-005. |
| Legal reason codes (CO, EO, LR, RA, RV, SA, LI, LS, AR, NT, GD) | Classify a legal *act or instrument event*. | Unchanged. LEGAL_SYSTEM classifies the *order*, not the act. |
| `extensions.legal_instrument` | Classifies a specific legal *document* and its enforceability. | Unchanged. May, in a future version, gain an optional `legal_system_ref`. Not built this pass. |
| `extensions.legal_module` | Names a jurisdiction-specific interpretation module (e.g. `lexcodex-us-ny-v2`). | A module MAY declare which legal system it implements via a `legal_system_ref`. |
| Compliance Module rules (LEG-002/003, CMX-001/002) | Govern legal-instrument consistency and extension precedence. | LSY rules mirror their structure and never contradict them. |

The distinction LEGAL_SYSTEM adds: **`jurisdiction` answers "where"; LEGAL_SYSTEM answers "what kind of legal order."** A `jurisdiction` value alone — `MX`, `US-NY`, `EU`, `PROTOCOL` — does not tell a consumer whether the operative order is codified civil law, common law, a religious order, a customary order, or protocol-code governance. LEGAL_SYSTEM supplies that axis.

## 3. Design Principles

1. **Classify, never determine.** Every entry records an asserted or verified description, marked as such by `classification_status`.
2. **No country-to-system identity.** A jurisdiction may host multiple legal systems; a legal system may span multiple jurisdictions. The model never hardcodes one to the other.
3. **Hybridity is first-class.** Systems may carry multiple legal families and reference secondary legal systems. A validator must not flatten a hybrid system to a single tradition.
4. **No implicit hierarchy.** No `legal_order_type` is subordinate to another by default. Precedence is expressed only through explicit `recognition_status` or explicit `secondary_legal_systems` ordering.
5. **Separation of axes.** Legal tradition, source of authority, and enforcement mode are distinct fields. A system's tradition does not imply its enforcement mode.
6. **Minimum viable, cleanly extensible.** Closed enums where the vocabulary is stable; an extensible string list (`subfamilies`) where it is not; a `metadata` escape hatch for the rest.
7. **Registry-governed and append-only in spirit.** Classifications change by new registry version, not by in-place mutation.

## 4. Taxonomy — The Pyramid Model

The taxonomy is layered. Level 0 is the set of classification *fields* carried by every entry. Levels 1–4 are the *values* those fields draw on.

### 4.1 Level 0 — Meta-classification (entry fields)

Every `legal_system` entry is classified along these axes: a single **legal order type** (Level 1); one or more **legal families** (Level 2) with one named primary; optional **subfamilies** (Level 3); optional cross-cutting **system characteristics**; one or more **source-of-authority** values; one or more **enforcement-mode** values; a single **recognition status**; a single **jurisdictional scope**; optional **domain scope** (Level 4); and a single **classification status** recording the epistemic standing of the record itself. Field-level definitions are in §6; the full value vocabularies are in Appendix A.

### 4.2 Level 1 — Legal Order Type

The macro order. Exactly one value per entry. Eight values: `state_law`, `religious_law`, `customary_indigenous_law`, `supranational_international_law`, `transnational_private_ordering`, `protocol_code_governance`, `informal_community_ordering`, `hybrid_order`.

`hybrid_order` is reserved for cases where even the *macro* order is genuinely mixed with no clear primary. A sovereign state with several legal traditions inside it (Nigeria) remains `state_law` with multiple `legal_families` — it is not `hybrid_order`.

### 4.3 Level 2 — Legal Families

Legal traditions. One or more per entry, with one `primary_legal_family`. Sixteen values, covering the classical families (civil, common, Islamic, canon, Jewish, Hindu, customary, indigenous, socialist), the supra/transnational families (international public, supranational, transnational commercial), and the digital families (protocol governance, DAO governance, smart-contract ordering, platform governance). See Appendix A.

### 4.4 Level 3 — Subfamilies

Implementation patterns within a family. Optional, extensible, lowercase snake_case. The schema does not close this list because it grows. Recommended starter values: `french_civil`, `german_civil`, `nordic_law`, `iberian_latin_american_civil`, `roman_dutch`, `socialist_civil`, `english_common`, `american_common`, `commonwealth_common`, `lex_mercatoria`, `maritime_admiralty`, `aviation_law`, `lex_sportiva`, `token_governance`, `multisig_governance`, `optimistic_governance`.

### 4.5 Cross-Cutting — System Characteristics

Structural features that are *not* legal families and recur across them: constitutional-review strength, parliamentary sovereignty, transformative constitutionalism, administrative-regulatory dominance, the several forms of legal pluralism, party-state control, military dominance, managed authoritarian legality, and postcolonial layering. These resolve the long-standing ambiguity in the source taxonomy where "constitutional", "administrative", and "pluralism" were repeatedly described as "not really a family." They are kept on a dedicated axis. See Appendix A.

### 4.6 Level 4 — Domain Scope

The applicable domain(s): general, or specific domains such as `financial_regulation`, `family_personal_status`, `digital_assets`, `dao_governance`, `cross_border_settlement`, `bridge_governance`, `on_chain_attestation`. Absence is read as `general`.

### 4.7 The five classification axes, summarized

| Axis | Field(s) | Cardinality | Answers |
|---|---|---|---|
| Order type | `legal_order_type` | one | What kind of legal order is this? |
| Tradition | `legal_families`, `primary_legal_family`, `subfamilies` | many / one / many | What legal DNA does it carry? |
| Structure | `system_characteristics` | many | What cross-cutting structural features does it have? |
| Authority | `source_of_authority` | many | From what does its bindingness derive? |
| Enforcement | `enforcement_mode` | many | How are its norms enforced? |

Recognition, jurisdictional scope, and domain scope frame these axes; `classification_status` and `confidence` describe the trustworthiness of the record.

## 5. Naming Conventions and IDs

**`legal_system_id`** follows `LS-<scope-tag>-<order-tag>-<NN>`, uppercase, e.g. `LS-MX-STATE-01`, `LS-DAO-GENERIC-01`, `LS-ARROW-PROTOCOL-01`.

The `scope-tag` is a **mnemonic only**. It carries no normative jurisdiction binding (LSY-005). For state systems it is convenient to use the ISO country code; for non-state systems it is a label (`DAO`, `ARROW`). Operative jurisdictions are recorded only in `related_jurisdictions`, and that field is a non-exclusive hint, not a mapping.

**Validation rules** use the `LSY-` prefix with a three-digit number: `LSY-001` … `LSY-007`.

**Vocabulary codes** are lowercase snake_case (`civil_law`, `protocol_code_governance`, `customary_adjudication`).

## 6. Registry Entry Format

The registry file is `{ "registry_metadata": {...}, "entries": [...] }`. The `entries` array shape is compatible with the `arrow-core` registry loader; `registry_metadata` is an ignored sibling key under the loader's deserialization and is safe at promotion time.

| Field | Req. | Type | Definition |
|---|---|---|---|
| `legal_system_id` | yes | string | Stable ID, `LS-…` convention. |
| `name` | yes | string | Human-readable name. |
| `description` | yes | string | Neutral description. No legal advice, no enforceability claim. |
| `legal_order_type` | yes | enum | Level 1 macro order (one). |
| `legal_families` | yes | enum[] | Level 2 traditions (≥1). |
| `primary_legal_family` | yes | enum | Dominant tradition. **Must** be a member of `legal_families` (LSY-002). |
| `subfamilies` | no | string[] | Level 3 patterns, extensible. |
| `system_characteristics` | no | enum[] | Cross-cutting structural features. |
| `source_of_authority` | yes | enum[] | Authority sources (≥1). |
| `enforcement_mode` | yes | enum[] | Enforcement modes (≥1). |
| `recognition_status` | yes | enum | The **only** field expressing precedence/subordination (LSY-006). |
| `jurisdictional_scope` | yes | enum | Territorial / protocol scope of the order. |
| `domain_scope` | no | enum[] | Level 4 domains. Absent ⇒ `general`. |
| `classification_status` | yes | enum | `descriptive` \| `asserted` \| `verified` \| `disputed` \| `normative`. |
| `confidence` | no | enum | `A`–`E`. Bounded by `verification_regime` (LSY-003). |
| `verification_regime` | no | string | Existing Arrow regime code. Required when `classification_status = verified`. |
| `asserted_by` | no | string | IDENTITY / DID reference of the asserting party. |
| `verified_by` | no | string | IDENTITY / DID reference of the verifier. Required when `verified`. |
| `related_jurisdictions` | no | string[] | Non-exclusive operative-context hint (LSY-005). |
| `secondary_legal_systems` | no | string[] | Ordered `legal_system_id` references; order = asserted precedence only. |
| `source_refs` | no | string[] | Non-authoritative supporting references. |
| `notes` | no | string | Free text. |
| `metadata` | no | object | Escape hatch. Must not override schema constraints. |
| `status` | yes | enum | `active` \| `draft` \| `deprecated`. |
| `added_version` | yes | string | Registry version of first appearance. |

### 6.1 `classification_status` — the epistemic axis

| Value | Meaning | Arrow handling |
|---|---|---|
| `descriptive` | Neutral description; no normative claim. | Default for well-settled state systems. |
| `asserted` | A party asserts this classification. | `asserted_by` SHOULD be populated. |
| `verified` | Classification verified by a referenced verifier. | `verified_by` and `verification_regime` MUST be populated (LSY-003). |
| `disputed` | The classification itself is contested. | `notes` MUST describe the dispute. Biases policy toward `HL`/`RW`. |
| `normative` | The classification is used to make a prescriptive claim. | Permitted but given **no special weight**; `asserted_by` MUST be populated. |

## 7. The `extensions.legal_system` Block (Proposed)

A thin, optional event extension, parallel to `extensions.legal_instrument`. **Proposed in this draft — not a live protocol field.** Promotion adds it to `compliance_extensions.json` and the Compliance Module as a separate, deliberate step.

```json
"extensions": {
  "legal_system": {
    "schema_version": "0.1",
    "legal_system_ref": "LS-MX-STATE-01",
    "secondary_legal_system_refs": ["LS-MX-INDIGENOUS-01"],
    "assertion_context": "asserted",
    "asserted_by": "did:example:asserting-party",
    "applicable_domain": "digital_assets",
    "notes": "Optional free text."
  }
}
```

`legal_system_ref` is the only required field beyond `schema_version`. The block is an **input to the Semantic Engine only**. It cannot change `arrow_code` (CMX-001) and cannot, by itself, authorize action (LSY-004). The canonical six-field string is untouched.

## 8. Example Entries

Ten seed entries ship in `legal_systems.json`. Highlights:

- **`LS-MX-STATE-01` — Mexico.** `state_law`, `civil_law` (Iberian / Latin American subfamily), strong constitutional review, administrative-regulatory dominance, state legal pluralism. References `LS-MX-INDIGENOUS-01` as a secondary rather than absorbing it — and asserts no precedence between them.
- **`LS-MX-INDIGENOUS-01` — Mexican indigenous normative systems.** `customary_indigenous_law`, families `indigenous_law` + `customary_law`, `recognition_status = constitutionally_recognized`. Recognition is stated at the level of the federal constitutional framework, with no article cited and no community competences enumerated.
- **`LS-US-STATE-01` — United States.** `state_law`, `common_law` (American subfamily), strong constitutional review. References `LS-US-TRIBAL-01` (`semi_autonomous`) and `LS-US-LA-CIVIL-01` (`federated_constituent`, civil-law primary) as secondaries. State-level common-law variation is treated as implementation variation, not separate entries.
- **`LS-EU-SUPRA-01` — European Union.** `supranational_international_law`, `supranational_law`, `recognition_status = supranational_recognized`. Coexists with Member State orders; a Member State context references both this and the relevant national entry.
- **`LS-SA-RELIGIOUS-01` — Saudi Arabia.** `religious_law`, `islamic_law`, with a modern statutory and administrative-regulatory overlay carried by `system_characteristics` and `source_of_authority` rather than by reclassifying the order.
- **`LS-NG-HYBRID-01` — Nigeria.** `state_law` with three families — `common_law` (primary), `customary_law`, `islamic_law` — plus `personal_status_pluralism` and `postcolonial_layered`. The worked multi-family example: hybridity expressed without `hybrid_order` and without flattening to one tradition.
- **`LS-DAO-GENERIC-01` — a DAO.** `protocol_code_governance`, `recognition_status = non_state_self_constituted`, `classification_status = asserted`. Generic template; a specific DAO registers its own entry. State-law recognition, where it exists, is recorded separately and never implied.
- **`LS-ARROW-PROTOCOL-01` — Arrow Protocol itself.** `protocol_code_governance`, `non_state_self_constituted`, `enforcement_mode = self_enforcement_or_compliance`. Explicitly **not** a state legal system and **not** a legal authority — the entry exists only so protocol-scoped events (`origin = PCY`, `jurisdiction = PROTOCOL`) can carry a self-classification.

## 9. Validation Rules (`LSY-`)

Seven rules. Each is logically necessary; the set is deliberately minimal and is not wired into the live pipeline in this draft. At promotion, LSY rules sit at the Compliance Module step (Step 10) alongside the LEG and CMX families.

| Rule | Severity | Constraint |
|---|---|---|
| **LSY-001** | Fatal | Any `legal_system_ref` or `secondary_legal_system_refs` value MUST resolve to an entry in `legal_systems.json` with `status = active`. Unresolved references MUST be rejected. |
| **LSY-002** | Fatal (structural) | `legal_families` MUST contain ≥1 value; `primary_legal_family` MUST be a member of `legal_families`. All enumerated fields MUST use registered vocabulary values. |
| **LSY-003** | Fatal | If `classification_status = verified`, `verified_by` (a resolvable IDENTITY/DID reference) and `verification_regime` MUST be present. If `asserted` or `normative`, `asserted_by` MUST be present. If `disputed`, `notes` MUST describe the dispute. `confidence` MUST NOT exceed the ceiling permitted by `verification_regime` — the LEGAL_SYSTEM analogue of SEM-015. |
| **LSY-004** | Fatal (semantic) | A LEGAL_SYSTEM classification — a registry entry or an `extensions.legal_system` block — MUST NOT, by itself, cause the Semantic Engine to emit `PR` or `TR`. It is descriptive context only. Mirrors CMX-001 and SEM-009; `arrow_code` remains the Semantic Engine's governing output. |
| **LSY-005** | Warning | Validators MUST NOT infer a `legal_system` from a `jurisdiction` value alone, nor a `jurisdiction` from a `legal_system` alone. `related_jurisdictions` is a non-exclusive hint. Multiple legal systems MAY share a jurisdiction; one legal system MAY span many jurisdictions. |
| **LSY-006** | Fatal (semantic) | No `legal_order_type` or `legal_family` is subordinate to any other by default. Precedence or subordination MAY be expressed ONLY through an explicit `recognition_status` value or an explicit `secondary_legal_systems` ordering — never inferred from order type or family. |
| **LSY-007** | Warning | If `legal_order_type = hybrid_order`, or `legal_families` contains more than one value, the plurality MUST be reflected in `secondary_legal_systems`, `system_characteristics`, or `notes`. A validator MUST NOT flatten a hybrid system to a single family. |

## 10. Integration Notes

**POLICY.** A LEGAL_SYSTEM classification is contextual *input* to the Semantic Engine, never its output. `arrow_code` remains governing (CMX-001). LSY-004 forbids a classification alone from producing `PR`/`TR`. A `recognition_status` of `contested`/`unrecognized`, or a `classification_status` of `disputed`, SHOULD bias the engine toward `HL` or `RW` — never toward `PR`.

**FINALITY.** A classification is metadata and carries no finality. It MUST NOT be used to assert `Finality = 1`. Classifications evolve; that change is handled by a new registry version, not by mutation, and not by a finality flag. Legal-system classification and event finality are orthogonal.

**CONFIDENCE.** The optional `confidence` field reuses the `A`–`E` scale and is bounded by `verification_regime` (LSY-003, mirroring SEM-015). Most state-system entries are `descriptive` with no confidence claim. A Confidence `E` classification MUST NOT be used to authorize action — the SEM-009 analogue inside LSY-004. The `legal` confidence subprofile is the natural ceiling source if/when LEGAL_SYSTEM is wired into `confidence_subprofiles.json`.

**IDENTITY / DID.** `asserted_by` and `verified_by` are IDENTITY/DID references. Resolution and verification are delegated entirely to the IDENTITY layer; `legal_systems.json` neither mints nor verifies identities. A legal-system entry is not itself an identity.

**BRIDGE.** A bridge spanning two chains may span two legal-order contexts; `extensions.legal_system` MAY annotate bridge-governance events. It is informational only and MUST NOT override `bridge_trust_flag` — BRG-001 is non-overridable and a LEGAL_SYSTEM classification is never a substitute for bridge trust.

**EVENT.** `extensions.legal_system` is the event-level hook — optional, parallel to `extensions.legal_instrument`. An `origin = LEG` event MAY carry it; a `legal_module` MAY assert it. It never alters the canonical six-field string.

**VALIDATION RULES.** LSY-001…007 are defined here as a draft and are not in the live pipeline. At promotion they join Step 10 beside the LEG and CMX families and are documented in the validation-rules specification.

**REGISTRIES.** `legal_systems.json` is a new registry. While a draft it lives under `registries/drafts/` and is not loaded by `arrow-core`. Promotion adds an `index.json` entry, a `validate-registries.js` validation function, and a loader binding. A future, separate version step MAY add an optional `legal_system_ref` to `extensions.legal_module` and `extensions.legal_instrument` — flagged here, not built now.

## 11. Edge Cases and Anti-Patterns

- **Country ≠ legal system.** Never treat a `jurisdiction` value as resolving to exactly one legal system, or vice versa (LSY-005). The relationship is many-to-many.
- **`jurisdiction = PROTOCOL` is not exclusively DAO/Arrow.** It is the natural pairing for `protocol_code_governance`, but the pairing is not exclusive in either direction.
- **No inferred hierarchy.** Recording a state system and a customary system in the same context does not make the customary system subordinate. Subordination is expressed only through `recognition_status` or explicit `secondary_legal_systems` ordering (LSY-006).
- **Disputed classifications.** When the classification itself is contested, set `classification_status = disputed` and describe it — never silently pick a side. For politically contested *orders*, prefer `recognition_status = contested` over the registry taking a position.
- **Self-reference humility.** Arrow's self-entry uses `non_state_self_constituted` + `descriptive` and must never imply legal authority or the power to determine law.
- **Extension override.** `extensions.legal_system` must never override `arrow_code` (CMX-001) or `bridge_trust_flag` (BRG-001).
- **Not a legal-advice tool.** The registry classifies asserted/verified metadata. It must not be used, or marketed, as determining applicable law or providing advice.
- **Mutation anti-pattern.** A change in `recognition_status` or family composition is a new registry version, not an in-place edit — consistent with Arrow's append-only philosophy.
- **Over-granularity anti-pattern.** Do not register a separate entry per sub-jurisdiction unless it carries a genuinely distinct legal order. Common-law state-to-state variation is implementation variation, not a new entry; Louisiana is a new entry because its primary family differs.
- **`hybrid_order` overuse.** Reserve the `hybrid_order` value for genuinely mixed *macro* orders. Multi-tradition sovereign states stay `state_law` with multiple `legal_families`.

## 12. Implementation Order

1. **Lock the taxonomy** — enums and field definitions in this document. *(This draft.)*
2. **Commit `legal_systems.schema.json`** under `registries/drafts/`. *(This draft.)*
3. **Commit `legal_systems.json`** under `registries/drafts/` with the ten seed entries. *(This draft.)*
4. **Draft-validate** entries against the schema and the LSY-001/002 invariants. No live pipeline wiring.
5. **Internal review** of taxonomy coverage, example correctness, and the LSY rule set.
6. **Promotion (separate, deliberate step):** move files out of `/drafts`; add the `index.json` entry; add a `validate-registries.js` function; add the LSY rules to the validation-rules specification; add `extensions.legal_system` to `compliance_extensions.json` and the Compliance Module; decide and apply the version bump (provisionally v1.5).
7. **Future extension:** optional `legal_system_ref` cross-links from `extensions.legal_module` and `extensions.legal_instrument`; LexCodex module-to-system binding.

## 13. File Placement

| File | Path | State |
|---|---|---|
| This specification | `specs/legal-system-foundation.md` | Draft |
| Registry | `registries/drafts/legal_systems.json` | Draft |
| Schema | `registries/drafts/legal_systems.schema.json` | Draft |

No live protocol file is modified by this draft. `legal-policy-primitives-v1.3.1.md`, `compliance_extensions.json`, `index.json`, and the validation-rules specification are referenced but untouched; their amendment is the promotion step in §12.6.

---

## Appendix A — Controlled Vocabulary Reference

**Level 1 — `legal_order_type`** (one per entry): `state_law`, `religious_law`, `customary_indigenous_law`, `supranational_international_law`, `transnational_private_ordering`, `protocol_code_governance`, `informal_community_ordering`, `hybrid_order`.

**Level 2 — `legal_family`** (one or more; one primary): `civil_law`, `common_law`, `islamic_law`, `canon_law`, `jewish_law`, `hindu_law`, `customary_law`, `indigenous_law`, `socialist_law`, `international_public_law`, `supranational_law`, `transnational_commercial_law`, `protocol_governance`, `dao_governance`, `smart_contract_ordering`, `platform_governance`.

**Level 3 — `subfamilies`** (extensible, lowercase snake_case): recommended starters include `french_civil`, `german_civil`, `nordic_law`, `iberian_latin_american_civil`, `roman_dutch`, `socialist_civil`, `english_common`, `american_common`, `commonwealth_common`, `lex_mercatoria`, `maritime_admiralty`, `aviation_law`, `lex_sportiva`, `token_governance`, `multisig_governance`, `optimistic_governance`.

**Cross-cutting — `system_characteristics`**: `constitutional_review_strong`, `constitutional_review_weak`, `parliamentary_sovereignty`, `transformative_constitutionalism`, `administrative_regulatory_dominant`, `legal_pluralism_state`, `legal_pluralism_deep`, `personal_status_pluralism`, `party_state_control`, `military_dominant`, `authoritarian_managed`, `postcolonial_layered`.

**`source_of_authority`**: `state_sovereignty`, `constitutional_mandate`, `legislative_enactment`, `judicial_precedent`, `religious_doctrine`, `customary_consensus`, `treaty_or_convention`, `supranational_delegation`, `protocol_consensus`, `contractual_agreement`, `community_recognition`, `executive_or_administrative_act`.

**`enforcement_mode`**: `state_courts`, `administrative_enforcement`, `religious_courts`, `customary_adjudication`, `arbitration`, `supranational_tribunal`, `on_chain_execution`, `code_is_enforcement`, `social_or_reputational_sanction`, `self_enforcement_or_compliance`, `none_declared`.

**`recognition_status`**: `sovereign`, `supranational_recognized`, `treaty_recognized`, `federated_constituent`, `constitutionally_recognized`, `autonomous`, `semi_autonomous`, `non_state_self_constituted`, `contested`, `unrecognized`, `descriptive_only`.

**`jurisdictional_scope`**: `national`, `subnational`, `supranational`, `international`, `transnational`, `extraterritorial`, `protocol_scope`, `community_scope`.

**Level 4 — `domain_scope`**: `general`, `constitutional`, `administrative_regulatory`, `financial_regulation`, `capital_markets`, `contract`, `property`, `land_tenure`, `family_personal_status`, `inheritance_succession`, `criminal`, `commercial`, `digital_assets`, `dao_governance`, `protocol_governance`, `dispute_resolution`, `cross_border_settlement`, `bridge_governance`, `on_chain_attestation`.

**`classification_status`**: `descriptive`, `asserted`, `verified`, `disputed`, `normative`.

**`confidence`**: `A`, `B`, `C`, `D`, `E` (reused from the core Confidence scale).

---

*End of draft v0.1.*
