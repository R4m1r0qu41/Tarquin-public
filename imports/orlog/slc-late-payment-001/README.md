# slc-late-payment-001 — Late Payment Clause

Status: draft (v0.1.0) · Taxonomy: `tax-commercial.payment.late-payment` · MC-004

The first pilot graduated from `labs/late-payment-pilot.md`: a bounded late-payment
clause. A payer owes a principal by a due date; after an optional grace period, a late
charge accrues (annual simple-interest rate or fixed per-day amount), subject to a cap.
The v0.1.0 logic processes a payment-in-full event only; partial-payment proration is
explicitly out of scope. It computes the charge and resulting obligation
deterministically from declared facts.

## Shape

Per `docs/slc-artifact-conventions.md`: `model/model.cto` (Concerto data model),
`text/grammar.tem.md` (TemplateMark with lex-linked binding clauses),
`logic/logic.ts` (pure TypeScript, no I/O, no clock, no model calls),
`fixtures/` (valid / invalid / disputed), `tests/logic.test.ts` (node:test).

Primitives used: `prim-grace-period`, `prim-late-fee-interest`, `prim-remedy-cap`.
Logic is intentionally self-contained (no imports): sealed artifacts do not share
mutable code; primitives are conventions the logic implements, not libraries it links.

## Conventions fixed by this artifact

- Dates are ISO `YYYY-MM-DD`, compared as UTC day numbers. No clock access — the payment
  date is always an input.
- Interest is simple, ACT/365-fixed; rounding is half-up to 2 decimals, applied once at
  the end of each computed amount.
- `accrualBasis` is explicit: `FROM_DUE_DATE` or `FROM_GRACE_EXPIRY`.

## Review posture

`requires_human_review: true`. The computation is mechanical, but the charge's
enforceability (reasonable liquidation vs. void penalty, `lex-us-ny-ucc-2-718`) is a
legal judgment even with the New York enactment pinned. The `disputed` fixture
demonstrates exactly this split: computation agreed, entitlement contested.
