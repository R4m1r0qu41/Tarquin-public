---
id: lex-us-ny-cplr-5004
jurisdiction: us-ny
source: ny-cplr
locator: 5004
title: Rate of interest
authority_type: statute
official_url: https://www.nysenate.gov/legislation/laws/CVP/5004
current_as_of: 2026-06-25
status: verified
defines:
  - statutory rate of interest
  - consumer debt
cross_refs:
  - lex-us-ny-gol-5-501
  - concept-liquidated-damages
source_url_hashed: https://www.nysenate.gov/legislation/laws/CVP/5004
source_hash: sha256:b6568e96cb9c41fe710b5b8b889a52a461d4486f25a3172a47b2a5589ee1a22a
lex_codex_status: synced@lex-grounding-001
notes: Verified point-in-time snapshot of NYSenate.gov Civil Practice Law & Rules (CVP) 5004; most recent revision shown 2022-05-06. Default rate 9% per annum; a 2021 amendment sets 2% per annum for actions arising out of a consumer debt where a natural person is a defendant. "Otherwise provided by statute" preserves rate ceilings such as GOL 5-501 / Banking Law 14-a.
---

# New York CPLR § 5004 — Rate of interest

## Summary

CPLR § 5004(a) fixes the statutory rate of interest at nine per centum per annum,
"except where otherwise provided by statute." A 2021 amendment (most recent revision
2022-05-06) sets a reduced rate of two per centum per annum for interest in an action
arising out of a *consumer debt* where a natural person is the defendant, on judgments
or accrued claims entered on or after the amendment's effective date. Subdivision (b)
defines "consumer debt" as an obligation of a natural person to pay money arising out of
a transaction primarily for personal, family, or household purposes.

## Why it matters here

This is the New York default ("legal") interest rate that applies to an overdue money
obligation when the agreement is silent on a rate. The late-payment pilot
(`slc-late-payment-001`) lets parties declare an explicit rate; where they do not, this
node is the gap-filling reference, and the 9% default plus the 2% consumer carve-out
bound the reasonableness analysis for any stated late charge. The pilot computes the
declared charge deterministically; selection of the governing rate remains a human legal
judgment.

## Links

- [[lex-us-ny-gol-5-501]] — usury ceiling on an agreed rate (companion)
- [[concept-liquidated-damages]] — enforceability frame for an agreed late charge
