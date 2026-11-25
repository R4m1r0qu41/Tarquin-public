---
id: concept-restricted-securities
jurisdiction: us
source: concept
locator: restricted-securities
title: Restricted securities
authority_type: concept
official_url: https://www.ecfr.gov/current/title-17/chapter-II/part-230/section-230.502
current_as_of: 2026-06-08
status: verified
cross_refs:
  - lex-us-17cfr-230-502
  - lex-us-17cfr-230-905
  - lex-us-15usc-77b
source_url_hashed: https://www.ecfr.gov/api/versioner/v1/full/2026-06-08/title-17.xml?section=230.502
source_hash: sha256:326362c227a3b72f473acef7e3df454e63a76ebdaa1fdef7f79b0b117169e89e
lex_codex_status: synced@lex-grounding-001
notes: Full definition is 17 CFR 230.144(a)(3) - TODO add lex-us-17cfr-230-144 when a resale artifact needs holding-period computation.
---

# concept: Restricted securities

Securities acquired in exempt transactions that cannot be freely resold: Reg D
securities via [[lex-us-17cfr-230-502]] (d), and domestic-issuer equity sold under Reg S
via [[lex-us-17cfr-230-905]]. Resale requires registration or an exemption; restricted
status survives private transfers. The canonical definition is § 230.144(a)(3) (node
TODO).

**Effect.** The legal reason transfer-restriction clauses exist and must bind
transferees.

**Artifact treatment.** `prim-restricted-transfer` expresses the restriction; legends
and stop-transfer instructions are recorded as declared facts; Rule 144 holding-period
math is computable but deferred until § 230.144 is mapped.
