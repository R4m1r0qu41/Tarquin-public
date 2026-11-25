---
id: concept-distribution-compliance-period
jurisdiction: us
source: concept
locator: distribution-compliance-period
title: Distribution compliance period (Regulation S)
authority_type: concept
official_url: https://www.ecfr.gov/current/title-17/chapter-II/part-230/section-230.902
current_as_of: 2026-06-08
status: verified
cross_refs:
  - lex-us-17cfr-230-902
  - lex-us-17cfr-230-903
source_url_hashed: https://www.ecfr.gov/api/versioner/v1/full/2026-06-08/title-17.xml?section=230.902
source_hash: sha256:143fcacf5eedb4c65ae3feb336ef74b5146b0f427e7c17a9f4000f57bd92388f
lex_codex_status: synced@lex-grounding-001
notes: The one fully computable element of the Reg S artifact - pure date arithmetic once the start date and category are declared facts.
---

# concept: Distribution compliance period

Defined at [[lex-us-17cfr-230-902]] (f): begins at the **later of** the date securities
are first offered to persons other than distributors or the closing date, and runs for
the period [[lex-us-17cfr-230-903]] sets per category — **40 days** (Category 2; and
Category 3 debt) or **1 year** (Category 3 equity); Category 1 has none. During it,
offers/sales to U.S. persons (other than per the rules) break the safe harbor.

**Artifact treatment.** The only mechanically computable Reg S element: given declared
`start_date` and `category`/`security_kind`, the end date and "is date X inside the
period" are pure functions. `slc-reg-s-offshore-restriction-001` computes exactly this
and nothing more.
