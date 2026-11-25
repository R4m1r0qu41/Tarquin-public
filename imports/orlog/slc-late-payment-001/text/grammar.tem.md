# Late Payment Clause — TemplateMark

> Binding clauses carry an explicit `[basis: <lex-node-id>]` tag. The bindings table at
> the end is machine-checked against `manifest.json.source_refs`.

## Clause 1 — Payment obligation

{{payer.name}} shall pay {{payee.name}} the principal amount of {{principal}}
{{currency}} (the "Outstanding Amount") on or before {{dueDate}} (the "Due Date").

## Clause 2 — Grace period

No late charge accrues if payment is made within {{graceDays}} calendar days after the
Due Date. If payment is made after that period, the late charge accrues
{{#if accrualBasisFromDueDate}}from the Due Date{{else}}from the expiry of the grace
period{{/if}}. [prim-grace-period]

## Clause 3 — Late charge (binding)

If payment is not made within the grace period, a late charge accrues on the
Outstanding Amount at {{#if annualRate}}{{rateOrAmount}}% per annum, as simple interest
on an actual/365 (fixed) basis{{else}}{{rateOrAmount}} {{currency}} per day{{/if}}, for
each chargeable day. The parties agree this charge is a reasonable liquidation of the
damages anticipated from late payment, the actual amount being difficult of proof.
[basis: lex-us-ny-ucc-2-718] [prim-late-fee-interest]

## Clause 4 — Cap on remedy (binding)

The aggregate late charge shall not exceed
{{#if capAbsolute}}{{capValue}} {{currency}}{{else}}{{capValue}}% of the Outstanding
Amount{{/if}}, and that limit is the exclusive measure of the late-charge remedy.
[basis: lex-us-ny-ucc-2-719] [prim-remedy-cap]

---

## Clause bindings

| Clause | Binding | lex node |
|---|---|---|
| 1 — Payment obligation | structural (defines amounts/dates) | — |
| 2 — Grace period | contract-defined term | concept-grace-period (via prim-grace-period) |
| 3 — Late charge | **binding** | lex-us-ny-ucc-2-718 |
| 4 — Cap on remedy | **binding** | lex-us-ny-ucc-2-719 |
