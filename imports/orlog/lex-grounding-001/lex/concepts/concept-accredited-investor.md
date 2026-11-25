---
id: concept-accredited-investor
jurisdiction: us
source: concept
locator: accredited-investor
title: Accredited investor
authority_type: concept
official_url: https://www.ecfr.gov/current/title-17/chapter-II/part-230/section-230.501
current_as_of: 2026-06-08
status: verified
cross_refs:
  - lex-us-17cfr-230-501
  - lex-us-17cfr-230-506
source_url_hashed: https://www.ecfr.gov/api/versioner/v1/full/2026-06-08/title-17.xml?section=230.501
source_hash: sha256:a54a63376aa3e8a20b053759ebda57d7ed371a373794351ba299292f11b08b1a
lex_codex_status: synced@lex-grounding-001
notes: US-defined term; no MX analog mapped yet.
---

# concept: Accredited investor

A person within one of the thirteen categories of [[lex-us-17cfr-230-501]] (a)(1)–(13)
— institutions, insiders, $1M-net-worth or $200K/$300K-income natural persons,
credentialed persons, knowledgeable employees, family offices — or whom the issuer
**reasonably believes** to be within one at the time of sale.

**Effect.** Gates Rule 506 ([[lex-us-17cfr-230-506]]): 506(b) caps *non*-accredited
purchasers at 35 with sophistication conditions; 506(c) requires *all* purchasers
accredited plus verification.

**Artifact treatment.** Represented as `{category, declared facts, verification method}`.
The category and thresholds are checkable; "reasonable belief" and verification
sufficiency are not — they stay flagged for human review.
