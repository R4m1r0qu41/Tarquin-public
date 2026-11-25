# Arrow Protocol — LEG Amendment Addendum: Jurisdiction Model

**Draft Specification v0.2** — reconciled to Arrow Protocol v1.6.0 (2026-06-14)

| Field | Value |
|---|---|
| Document | `specs/leg-jurisdiction-addendum.md` |
| Status | **Draft** |
| Amends | `specs/leg-origin-amendment.md` |
| Also affects | `docs/protocol/arrow-protocol.md` Part 17, `docs/protocol/validation-rules.md` LEG rules |
| Protocol version impact | Originally promoted in v1.5-draft; reconciled to v1.6.0 naming + numbering. |
| Canonical message string | **Unchanged.** |
| Validation rules | LEG-009, LEG-010 (Step 5b); LEG-012-EXT, LEG-013-EXT (Compliance Module, Step 14); LEG-014 retired |
| New registry | `registries/leg_supra_jurisdictions.json` (promoted, active) |

> **v0.2 reconciliation (Arrow v1.6.0).** Field 4 is **Action** (only Origin `PCY` keeps the display name "Policy"). Rule numbering matched to Arrow v1.6.0: jurisdiction rules `LEG-010 → LEG-009` (required) and `LEG-011 → LEG-010` (`supra` registry); core gate rules from the amendment are `MSG-006` (subtype), `LEG-006` (Action Derivation Gate), `LEG-007` (SUPRA force-BL), `LEG-008` (warning). `LEG-012 → LEG-012-EXT` and `LEG-013 → LEG-013-EXT` (moved to the Compliance Module, Step 14); **`LEG-014` RETIRED** (flat `jurisdiction` is a computed projection of `leg_jurisdiction`). The `leg_supra_jurisdictions.json` registry is promoted (no longer under `drafts/`).
>
> **⚠️ Structural delta pending (not silently rewritten).** Arrow v1.6.0 relocated the subnational forum from `leg_jurisdiction.local` to `extensions.legal_instrument.local_forum`. §3.1–3.3, the examples in §3.5, and §5 still describe the old `leg_jurisdiction.local` placement. The field-model rewrite (and the matching example/JSON updates) is a deliberate design item deferred to the v1.6 housekeeping pass. Filename bump (→ `-v0.2.md`) and reference updates are likewise deferred.

---

## 1. Problem Statement

The existing `jurisdiction` field in the Arrow Event Object is a flat string: ISO 3166-1 alpha-2, UN M.49 region code, or `PROTOCOL`. This is sufficient for routing and compliance-module consistency checks (LEG-002). It is insufficient for LEG-origin events, which require three things the flat string cannot express simultaneously:

1. **Which sovereign state** the order comes from — the existing flat string carries a country code but cannot carry subnational or supranational context alongside it.
2. **Which supranational body** issued or applies the instrument — a controlled list of entities with genuine enforcement authority, independent of or overlapping with state jurisdiction.
3. **Which subnational forum** issued the specific order — because most real-world court orders and regulatory decisions originate in local or regional bodies, and that precision matters for enforcement and recognition analysis.

The flat `jurisdiction` field answered (1) loosely and ignored (2) and (3). For LEG events, all three must be expressible. The solution is a structured `leg_jurisdiction` object.

---

## 2. Design Decisions

**Country code convention: ISO 3166-1 alpha-2 — consistent with the existing `jurisdiction` field.**

`leg_jurisdiction.country` uses ISO 3166-1 alpha-2 uppercase two-character codes (`MX`, `US`, `GB`, `DE`, `ES`, etc.). This is the same standard already used by the flat `jurisdiction` field throughout Arrow. No convention change, no deviation note needed.

**Flat `jurisdiction` field: preserved for backward compatibility.**

`leg_jurisdiction` is an *additional* structured field for LEG events. The existing flat `jurisdiction` field is not removed. For LEG events that carry `leg_jurisdiction`, the flat `jurisdiction` field MUST be populated with the value derived from `leg_jurisdiction` per the derivation rule in §3.4, so that LEG-002 and all existing rules continue to operate unchanged.

**Supranational list: closed and conservative.**

Only bodies with genuine enforcement or executive authority are registered. This is a deliberate constraint. Most international organizations are advisory. The list starts at eight entries and grows only by explicit registration with documented enforcement basis. OFAC, for example, is NOT in this list; it is a US domestic regulatory body and its orders are `ADM` + `country = us`, regardless of extraterritorial reach.

**Local codes: structurally validated, semantically deferred.**

Arrow validates the structural format of `local` (2–4 uppercase alpha characters) but does not maintain a canonical registry of subnational codes. That registry is a LexCodex responsibility — the volume is exponential (thousands of courts and regulatory districts globally). Arrow requires only that the code is present, structurally valid, and consistent with `country`.

---

## 3. `leg_jurisdiction` Object

### 3.1 Definition

A new structured required field for all `origin = LEG` events. Replaces the flat `jurisdiction` string as the operative jurisdiction specification for LEG-origin validation. The flat `jurisdiction` field continues to be populated from it per §3.4.

```json
"leg_jurisdiction": {
  "country": "MX",
  "supra": null,
  "local": "CDMX"
}
```

### 3.2 Field Specification

| Field | Type | Required | Format | Constraint |
|---|---|---|---|---|
| `country` | string | Conditional | ISO 3166-1 alpha-2, uppercase, 2 chars `[A-Z]{2}` | Required unless `supra` is present. At least one of `country` or `supra` MUST be present (LEG-009). |
| `supra` | string | Conditional | Controlled vocabulary — see §4 | Required unless `country` is present. MUST be a registered entry in `leg_supra_jurisdictions.json` (LEG-010). |
| `local` | string | Optional | 2–4 uppercase alpha `[A-Z]{2,4}` | If present, `country` MUST be present (LEG-012-EXT). Semantically validated against LexCodex subnational registry; structurally validated by pattern only at the Arrow layer (LEG-013-EXT). *(v1.6.0 relocates this to `extensions.legal_instrument.local_forum`; see reconciliation note.)* |

### 3.3 Cardinality and combination rules

All three fields may be present simultaneously. Common patterns:

| Pattern | Example | Meaning |
|---|---|---|
| `country` only | `{ "country": "MX" }` | Federal Mexican order, no subnational specification |
| `country` + `local` | `{ "country": "US", "local": "NY" }` | New York State or SDNY court order |
| `supra` only | `{ "supra": "UNSC" }` | UN Security Council resolution, no single country |
| `country` + `supra` | `{ "country": "FR", "supra": "EBA" }` | EBA decision applied in France |
| `country` + `supra` + `local` | `{ "country": "DE", "supra": "ECB", "local": "HES" }` | ECB/SSM supervisory decision for a Frankfurt institution |

`supra` + `local` without `country` is **prohibited** (LEG-012-EXT). A subnational code without a state anchor is meaningless.

### 3.4 Derivation rule for flat `jurisdiction` field

For backward compatibility with LEG-002 and all existing rules that read the flat `jurisdiction` field, the following derivation applies when `leg_jurisdiction` is present:

```
if leg_jurisdiction.country is present:
    jurisdiction = leg_jurisdiction.country  [ISO 3166-1 alpha-2]
else if leg_jurisdiction.supra is present:
    jurisdiction = leg_jurisdiction.supra    [supra code]
```

The flat `jurisdiction` field continues to be required per §3 of the Event Object specification; it MUST be populated with this derived value. *(v1.6.0: LEG-014 is retired — the flat `jurisdiction` is treated as a computed projection of `leg_jurisdiction` rather than a separately validated, independently-supplied field, so a mismatch cannot arise at runtime.)*

### 3.5 Example objects

**Mexican federal regulatory order (CNBV administrative action):**
```json
{
  "origin": "LEG",
  "leg_subtype": "ADM",
  "leg_jurisdiction": {
    "country": "MX",
    "local": "CDMX"
  },
  "jurisdiction": "MX"
}
```

**New York Southern District court order (SDNY):**
```json
{
  "origin": "LEG",
  "leg_subtype": "JUD",
  "leg_jurisdiction": {
    "country": "US",
    "local": "NY"
  },
  "jurisdiction": "US"
}
```

**EU binding regulation applied in Spain:**
```json
{
  "origin": "LEG",
  "leg_subtype": "SUPRA",
  "leg_jurisdiction": {
    "country": "ES",
    "supra": "EU"
  },
  "jurisdiction": "ES"
}
```

**UN Security Council mandatory sanctions resolution:**
```json
{
  "origin": "LEG",
  "leg_subtype": "SUPRA",
  "reason_code": "SA",
  "leg_jurisdiction": {
    "supra": "UNSC"
  },
  "jurisdiction": "UNSC"
}
```

**ECB/SSM supervisory decision, Frankfurt institution:**
```json
{
  "origin": "LEG",
  "leg_subtype": "ADM",
  "leg_jurisdiction": {
    "country": "DE",
    "supra": "ECB",
    "local": "HES"
  },
  "jurisdiction": "DE"
}
```

---

## 4. Supranational Jurisdiction Vocabulary

### 4.1 Design principle

Admission criterion: the body must have **direct enforcement authority or binding decision-making power** over regulated entities or member states — not merely advisory, recommendatory, or coordinative functions. The list is closed. Growth requires a registered entry with documented enforcement basis.

Entities explicitly excluded from this list and their correct treatment:

| Entity | Correct Arrow treatment |
|---|---|
| OFAC (US Treasury) | `leg_subtype = ADM`, `country = us` — US domestic regulatory body with extraterritorial reach |
| BIS (Bank for International Settlements) | Advisory/standard-setting; not a legal enforcement body; out of LEG scope |
| IMF | Financial assistance, not enforcement authority in the LEG sense |
| WTO | Dispute settlement authorizes retaliation, not direct enforcement; `leg_subtype = PREC` is the closest fit for WTO rulings |
| OECD | Advisory; out of scope |
| World Bank | Lending, not enforcement |

### 4.2 Registered supranational jurisdiction entries

Eight entries, v0.1 initial vocabulary.

| Code | Full name | Enforcement basis | Primary region | Notes |
|---|---|---|---|---|
| `EU` | European Union | Treaty on the Functioning of the EU (TFEU): ECJ enforcement, infringement proceedings, direct effect and supremacy of EU law, binding regulations and directives. | Europe (27 Member States) | Broadest supranational body in this list. Covers the full acquis communautaire. Specific EU agencies (EBA, ESMA, ECB) are separate entries when the specific agency is the issuing body. |
| `EBA` | European Banking Authority | EU Regulation 1093/2010: binding technical standards (BTS), legally binding mediation between NCAs, breach of Union law investigations. AML/CFT supervisory convergence under AMLA transition. | EU Member States (banking) | Binding only through BTS; direct supervisory powers over individual institutions are limited and transitional. |
| `ESMA` | European Securities and Markets Authority | EU Regulation 1095/2010: direct supervisory authority over credit rating agencies (CRAs), trade repositories, central counterparties (CCPs), and certain third-country firms. | EU Member States (capital markets) | Has direct supervisory authority over specific entity categories — not merely standard-setting. |
| `ECB` | European Central Bank | Treaty on EU (TEU) / TFEU, SSM Regulation (EU 1024/2013): Single Supervisory Mechanism (SSM) direct supervisory authority over significant credit institutions. Monetary policy decisions binding on eurozone. | Eurozone / SSM participating states | Two distinct authority modes: monetary policy (all eurozone) and banking supervision (significant institutions only). |
| `UNSC` | United Nations Security Council | UN Charter Chapter VII: mandatory measures binding on all UN member states; authorization of enforcement action. | Global | The only body in this list with global mandatory binding force. Chapter VII resolutions override domestic law in member states. UNSC sanctions are the primary use case. |
| `FATF` | Financial Action Task Force | FATF Mandate (2012 Recommendations): blacklisting/greylisting creates systemic exclusion force — correspondent banking, market access, and investment restrictions follow automatically from listing. | Global | Not a treaty body; enforcement operates through systemic exclusion and reputational sanctions rather than direct legal orders. Listed here because the practical effect on regulated entities is coercive and near-mandatory. |
| `ICC` | International Criminal Court | Rome Statute (1998): binding arrest warrants and surrender obligations on State Parties; jurisdiction over genocide, crimes against humanity, war crimes, aggression. | State Parties to the Rome Statute (124 states) | Jurisdiction is over individuals, not entities or assets. Relevant in Arrow contexts involving freezing of assets connected to ICC proceedings. |
| `ICSID` | International Centre for Settlement of Investment Disputes | ICSID Convention (1966): arbitral awards are binding on contracting states and have the force of a final judgment of a domestic court in all contracting states (Article 54). | ICSID Contracting States (160+ states) | The most enforceable form of international arbitral award. A final ICSID award supports `leg_subtype = JUD` + `supra = ICSID`. |

### 4.3 Registry file (`registries/leg_supra_jurisdictions.json`)

```json
{
  "registry_metadata": {
    "registry_name": "leg_supra_jurisdictions",
    "schema_version": "0.1",
    "status": "draft",
    "description": "Controlled vocabulary of supranational bodies with enforcement or binding executive authority, registered as valid values for leg_jurisdiction.supra in LEG-origin events. Admission criterion: direct enforcement authority or binding decision-making power over regulated entities or member states."
  },
  "entries": [
    {
      "code": "EU",
      "name": "European Union",
      "enforcement_basis": "TFEU — ECJ enforcement, infringement proceedings, direct effect, treaty supremacy",
      "enforcement_type": "treaty_enforcement",
      "geographic_scope": "eu_member_states",
      "status": "active",
      "added_version": "0.1"
    },
    {
      "code": "EBA",
      "name": "European Banking Authority",
      "enforcement_basis": "EU Regulation 1093/2010 — binding technical standards, breach-of-Union-law investigations",
      "enforcement_type": "regulatory_standard_binding",
      "geographic_scope": "eu_member_states",
      "domain": "banking",
      "status": "active",
      "added_version": "0.1"
    },
    {
      "code": "ESMA",
      "name": "European Securities and Markets Authority",
      "enforcement_basis": "EU Regulation 1095/2010 — direct supervisory authority over CRAs, trade repositories, CCPs",
      "enforcement_type": "direct_supervisory",
      "geographic_scope": "eu_member_states",
      "domain": "capital_markets",
      "status": "active",
      "added_version": "0.1"
    },
    {
      "code": "ECB",
      "name": "European Central Bank",
      "enforcement_basis": "TEU/TFEU, SSM Regulation (EU 1024/2013) — monetary policy, direct banking supervision",
      "enforcement_type": "direct_supervisory",
      "geographic_scope": "eurozone_ssm",
      "domain": "banking_monetary",
      "status": "active",
      "added_version": "0.1"
    },
    {
      "code": "UNSC",
      "name": "United Nations Security Council",
      "enforcement_basis": "UN Charter Chapter VII — mandatory measures, override of domestic law, enforcement authorization",
      "enforcement_type": "mandatory_global",
      "geographic_scope": "global_un_members",
      "status": "active",
      "added_version": "0.1",
      "notes": "Chapter VII resolutions are binding on all 193 UN member states and override domestic law. Primary use case: UNSC sanctions (reason_code = SA triggers LEG-007 BL force)."
    },
    {
      "code": "FATF",
      "name": "Financial Action Task Force",
      "enforcement_basis": "FATF Mandate — High-Risk Jurisdictions (blacklist) and Increased Monitoring (greylist) listings create systemic market exclusion; correspondent banking, capital market access, and investment restrictions follow automatically from listing",
      "enforcement_type": "systemic_exclusion",
      "geographic_scope": "global",
      "domain": "aml_cft",
      "status": "active",
      "added_version": "0.1",
      "policy_derivation": {
        "blacklist": "BL — High-Risk Jurisdictions subject to a Call for Action. Triggers LEG-007 force-BL when leg_subtype=SUPRA + reason_code=SA + Finality=1, identical to UNSC sanctions path.",
        "greylist": "HL — Jurisdictions under Increased Monitoring. Triggers HL; does not force BL. Action derived from subtype+finality table; reason_code=SA is not appropriate for greylist — use reason_code=CO (compliance obligation) or a future dedicated FATF reason code.",
        "jurisdiction_trigger": "The FATF listing status of leg_jurisdiction.country is the operative input. A FATF blacklist event with country=IR (Iran) or country=KP (North Korea) triggers the same BL-force path as a UNSC sanctions resolution. Implementations SHOULD cross-reference FATF listing status in their deployment policy layer (Step 13) for greylist jurisdictions."
      },
      "notes": "Not a treaty body. Enforcement operates through systemic exclusion. BL path is available via the existing LEG-007 + reason_code=SA mechanism. Greylist produces HL, not BL, and is not a LEG-007 trigger."
    },
    {
      "code": "ICC",
      "name": "International Criminal Court",
      "enforcement_basis": "Rome Statute (1998) — binding arrest warrants, surrender obligations on State Parties",
      "enforcement_type": "criminal_jurisdiction",
      "geographic_scope": "rome_statute_states",
      "domain": "criminal",
      "status": "active",
      "added_version": "0.1",
      "notes": "Jurisdiction over individuals, not entities. Relevant to asset freezing in ICC proceedings. A final ICC warrant supports leg_subtype = JUD."
    },
    {
      "code": "ICSID",
      "name": "International Centre for Settlement of Investment Disputes",
      "enforcement_basis": "ICSID Convention Art. 54 — final awards binding and enforceable as final domestic court judgments in all contracting states",
      "enforcement_type": "arbitral_award_binding",
      "geographic_scope": "icsid_contracting_states",
      "domain": "investment_arbitration",
      "status": "active",
      "added_version": "0.1",
      "notes": "Most enforceable form of international investment arbitration. A final ICSID award is treated as JUD-equivalent for policy derivation purposes."
    }
  ]
}
```

---

## 5. Local Code Convention

### 5.1 Purpose

The `local` field identifies the subnational forum — a state, province, region, court district, or regulatory district — from which a specific order originates. It exists because in practice, most orders are local: a CDMX court, a New York federal district, a Hessian banking supervisor, a Catalan administrative tribunal.

### 5.2 Format

`[A-Z]{2,4}` — 2 to 4 uppercase Latin alphabet characters. No numbers, no special characters, no lowercase.

Examples:

| Code | Jurisdiction | Country | Context |
|---|---|---|---|
| `NY` | New York State | US | SDNY, NY state courts |
| `CA` | California | US | CDCA, California state courts |
| `TX` | Texas | US | NDTX, Texas state courts |
| `CDMX` | Ciudad de México | MX | CDMX courts and administrative bodies |
| `JAL` | Jalisco | MX | Guadalajara courts |
| `NL` | Nuevo León | MX | Monterrey courts |
| `LON` | Greater London | GB | UK High Court sittings in London |
| `HES` | Hesse | DE | Frankfurt courts, Frankfurt banking supervisor |
| `CAT` | Catalonia | ES | Barcelona courts, Catalan autonomous bodies |
| `QC` | Quebec | CA | Quebec courts and civil-law jurisdiction |
| `ON` | Ontario | CA | Ontario courts |

### 5.3 Semantic validation: deferred to LexCodex

Arrow validates only structure (`[A-Z]{2,4}` + `country` present). It does not maintain a canonical list of valid local codes. The LexCodex subnational registry is the authoritative source for semantic validation. When a `local` value passes Arrow structural validation but is not in the LexCodex registry, LEG-013-EXT fires with severity controlled by deployment configuration: `silent`, `warning`, or `fatal`. The default recommended setting is `warning`.

---

## 6. Validation Rules (LEG-009, LEG-010; plus LEG-012-EXT / LEG-013-EXT)

The jurisdiction rules `LEG-009` and `LEG-010` run at **Step 5b** of the validation pipeline (the LEG Action Derivation Gate), after FB-001 and LEG-004 through LEG-008, before Step 6. The former `local`-forum rules now run at **Step 14** (Compliance Module) as `LEG-012-EXT` / `LEG-013-EXT`. All are fatal unless noted.

| Rule | Severity | Constraint |
|---|---|---|
| **LEG-009** | Fatal | If `origin = LEG`, `leg_jurisdiction` MUST be present. Within `leg_jurisdiction`, at least one of `country` or `supra` MUST be present. An event with `origin = LEG` and no `leg_jurisdiction` object, or a `leg_jurisdiction` with neither `country` nor `supra`, MUST be rejected. |
| **LEG-010** | Fatal | If `leg_jurisdiction.supra` is present, its value MUST be a registered entry in `leg_supra_jurisdictions.json` with `status = active`. An unregistered `supra` value MUST be rejected. |
| **LEG-012-EXT** | Fatal (Step 14, Compliance Module) | When `extensions.legal_instrument.local_forum` is present, it MUST match `[A-Z]{2,4}` and `leg_jurisdiction.country` MUST be present in the same event. A `local_forum` value without a `country` anchor MUST be rejected. *(v1.6.0: the subnational forum moved from `leg_jurisdiction.local` to `extensions.legal_instrument.local_forum` — see the reconciliation note; §3/§5 still describe the old placement pending the design rewrite.)* |
| **LEG-013-EXT** | Deployment-configurable (Step 14) | If `extensions.legal_instrument.local_forum` is present and does not resolve to a registered entry in the LexCodex subnational registry, the pipeline emits `LEG_LOCAL_UNREGISTERED`. **Severity is deployment-configurable** (silent / warning / fatal), set in the deployment policy configuration (Step 13); default recommended `warning`. The event is not protocol-level rejected at any configuration level. |
| ~~**LEG-014**~~ | **RETIRED (v1.6.0)** | Flat `jurisdiction` is a computed projection of `leg_jurisdiction` (§3.4); no separate runtime rule is required. The former `LEG_JURISDICTION_MISMATCH` reason code is retired with it. |

### 6.1 Amendment to LEG-002

LEG-002 currently reads: *"If `extensions.legal_instrument` is present, the `jurisdiction` field in the root Event Object MUST match or be a superset of `legal_instrument.jurisdiction`."*

This rule is amended for events that carry `leg_jurisdiction`:

> **LEG-002 (amended for v1.5):** If `extensions.legal_instrument` is present and the root Event Object carries `leg_jurisdiction`, the `legal_instrument.jurisdiction` field MUST be consistent with `leg_jurisdiction.country` (if present) or `leg_jurisdiction.supra` (if `country` is absent). The flat `jurisdiction` field derivation in §3.4 of the LEG Jurisdiction Addendum satisfies the match requirement for all existing LEG-002 checks.

This amendment is non-breaking: LEG-002 continues to operate as before for all non-LEG events and all LEG events that do not carry `leg_jurisdiction`.

---

## 7. Updated Step 5b Pipeline

```
Step 5b   If origin = LEG: apply LEG Action Derivation Gate
          → MSG-006: reject if leg_subtype absent or unregistered
          → LEG-004: reject if Confidence = E and Action ≠ RW
          → LEG-005: reject if Confidence = D and subtype ∉ {LEG_TEXT, PREC}
          → LEG-007: force Action BL if subtype=SUPRA + reason_code=SA + Finality=1
          → LEG-006: derive permitted Action range; reject if outside range
          → LEG-008: warn if subtype ∈ {LEG_TEXT, PREC} and Finality=1
          ── Jurisdiction (new) ──────────────────────────────────────────────
          → LEG-009: reject if leg_jurisdiction absent or missing country+supra
          → LEG-010: reject if supra not in leg_supra_jurisdictions registry
          ── (Step 14, Compliance Module) ────────────────────────────────────
          → LEG-012-EXT: reject if extensions.legal_instrument.local_forum bad format or missing country anchor
          → LEG-013-EXT: [configurable: silent/warn/fatal] if local_forum not in LexCodex registry
          (LEG-014 retired: flat jurisdiction is a computed projection of leg_jurisdiction)
```

---

## 8. Edge Cases and Anti-Patterns

- **OFAC is not `supra`.** A US OFAC SDN designation is `leg_subtype = ADM`, `country = us`, and optionally `reason_code = SA` if it is a sanctions block. Its extraterritorial reach is a feature of US domestic law. It does not become a supranational instrument merely because it affects non-US parties.

- **EU directives before transposition.** An EU directive that has not yet been transposed into domestic law is `supra = EU` + `Finality = 0` → Action `{HL, RW}` per the LEG-006 derivation table. Once transposed, the implementing domestic measure is a separate `LEG_TEXT` event (`country = de`, for example) that references the EU directive in `related_events`.

- **WTO rulings.** WTO Dispute Settlement Body rulings are not in the `supra` vocabulary. They are best expressed as `leg_subtype = PREC` with `related_events` referencing the specific dispute, or as `LEG_TEXT` with `supra` absent, because WTO rulings authorize retaliation rather than directly ordering parties. A future RFC could address this specifically.

- **FATF blacklist vs greylist.** A FATF High-Risk Jurisdiction (blacklist — currently Iran `IR`, North Korea `KP`) is treated identically to a UNSC sanctions instrument: `leg_subtype = SUPRA`, `supra = FATF`, `reason_code = SA`, `Finality = 1` → LEG-007 forces `BL`. A FATF Increased Monitoring jurisdiction (greylist) produces `HL`, not `BL`; `reason_code = SA` is not appropriate for greylist — use `CO` or a future dedicated FATF reason code. The operative input is `leg_jurisdiction.country` matching a listed jurisdiction; implementations SHOULD maintain FATF listing status at the deployment policy layer (Step 13) for greylist cross-referencing, given that greylist additions and removals occur between FATF plenary sessions.

- **`local` codes for federal courts.** The US federal district court system (SDNY, NDCA, etc.) maps to the state-level `local` code of the district: `NY` for SDNY, `CA` for NDCA. The specific court (district vs circuit vs supreme) is captured in `leg_instrument_type` in `extensions.legal_instrument`, not in `local`.

- **Compound `supra` events.** An instrument issued jointly by EBA and ESMA (e.g., joint guidelines) is expressed as a single `supra` entry with the primary issuing body named. If genuinely joint and equal, use `supra = EU` (the common parent) and note the specific agencies in `metadata`. Do not attempt to place two values in the `supra` field; the field is a single string.

---

## 9. File Placement

| File | Path | State |
|---|---|---|
| This addendum | `specs/leg-jurisdiction-addendum.md` | Draft (v0.2, reconciled to v1.6.0) |
| Supranational vocabulary registry | `registries/leg_supra_jurisdictions.json` | **Active (promoted)** |

`leg_supra_jurisdictions.json` is promoted into `registries/` (non-draft) and registered in `index.json`; `LEG-009`/`LEG-010` are live in the validation pipeline. The remaining design item is the `local` → `extensions.legal_instrument.local_forum` relocation (see reconciliation note).

---

*End of addendum v0.2 (reconciled to Arrow Protocol v1.6.0).*
