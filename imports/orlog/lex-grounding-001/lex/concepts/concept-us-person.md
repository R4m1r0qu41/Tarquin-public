---
id: concept-us-person
jurisdiction: us
source: concept
locator: us-person
title: U.S. person (Regulation S)
authority_type: concept
official_url: https://www.ecfr.gov/current/title-17/chapter-II/part-230/section-230.902
current_as_of: 2026-06-08
status: verified
cross_refs:
  - lex-us-17cfr-230-902
  - lex-us-17cfr-230-903
  - lex-us-17cfr-230-904
source_url_hashed: https://www.ecfr.gov/api/versioner/v1/full/2026-06-08/title-17.xml?section=230.902
source_hash: sha256:143fcacf5eedb4c65ae3feb336ef74b5146b0f427e7c17a9f4000f57bd92388f
lex_codex_status: synced@lex-grounding-001
notes: Regulation S definition only - other regimes (tax, OFAC, CFTC) define "U.S. person" differently; never reuse this concept outside Reg S artifacts.
---

# concept: U.S. person (Regulation S)

Defined at [[lex-us-17cfr-230-902]] (k): natural persons resident in the United States;
partnerships/corporations organized under US law; certain estates, trusts, and agency
accounts; and foreign entities formed by a U.S. person principally for investing in
unregistered securities — with the (k)(2) exclusions.

**Effect.** The buyer-status axis of every Reg S condition: offshore transaction
([[concept-offshore-transaction]]), Category 3 purchaser certification, resale
restrictions.

**Artifact treatment.** A representation by the purchaser ("is not a U.S. person"),
recorded with the residency/organization facts supporting it. Status edge cases
(dual-resident persons, multi-jurisdiction entities) stay human.
