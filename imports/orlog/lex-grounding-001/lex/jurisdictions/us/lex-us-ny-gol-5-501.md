---
id: lex-us-ny-gol-5-501
jurisdiction: us-ny
source: ny-gol
locator: 5-501
title: Rate of interest; usury forbidden
authority_type: statute
official_url: https://www.nysenate.gov/legislation/laws/GOB/5-501
current_as_of: 2026-06-25
status: verified
defines:
  - rate of interest
  - usury
cross_refs:
  - lex-us-ny-cplr-5004
  - concept-liquidated-damages
source_url_hashed: https://www.nysenate.gov/legislation/laws/GOB/5-501
source_hash: sha256:dd1b2daa7b5d9baa3820f76eff1e59831d58969b8ebea586a173c669be27e3bb
lex_codex_status: synced@lex-grounding-001
notes: Verified point-in-time snapshot of NYSenate.gov General Obligations Law (GOB) 5-501. Civil usury rate is 6% per annum (5-501(1)) unless Banking Law 14-a prescribes a different rate (16% by current superintendent rule); 5-501(6) exempts loans of $250,000+ (other than those secured by a one-to-six-family owner-occupied residence) and $2,500,000+ from the civil usury ceiling. Criminal usury (Penal Law 190.40, 25%) and Banking Law 14-a are referenced by this section but not yet separate nodes.
---

# New York General Obligations Law § 5-501 — Rate of interest; usury forbidden

## Summary

GOL § 5-501(1) sets the rate of interest on a loan or forbearance of money, goods, or
things in action at six per centum per annum unless a different rate is prescribed in
Banking Law § 14-a. Subdivision (2) forbids any person or corporation from charging,
taking, or receiving interest above that prescribed rate, and provides that the amount
treated as interest includes amounts paid directly or indirectly to or for the lender's
account in consideration for the loan or forbearance. Subdivisions (5)–(6) carve out
larger loans from the civil usury ceiling.

## Why it matters here

The late-payment pilot (`slc-late-payment-001`) computes a late charge that may be
expressed as an annual simple-interest rate. This node pins the New York ceiling that
bounds the *enforceability* of such a rate: a stated rate above the lawful maximum is
usurious and unenforceable. The pilot still computes the charge deterministically; the
usury bound is part of the human enforceability judgment the artifact flags, not a
mechanized check.

## Links

- [[lex-us-ny-cplr-5004]] — statutory default interest rate (companion)
- [[concept-liquidated-damages]] — enforceability frame for an agreed late charge
