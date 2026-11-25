# Kim v. DPRK / Arbitrum — Source Record

Source documents and reporting backing the OA-011 design reference in
`lex-exe/leg-origin-adapter.md` and the live-instance analysis in
`drafts/sibyl/external-source-verification-and-template-classification.md`.
Collected/updated 2026-06-11. Relates to Orlog commission MC-005.

Facts are **tiered by provenance** — this is the whole discipline of the project, applied to
its own sources. Do not collapse the tiers.

## Tier 1 — Court-verified (read directly from filed documents)

| File | sha256 | What it establishes |
|---|---|---|
| `2026-05-01-sdny-25-mc-00527-MMG-doc-34.pdf` | `6f71364259c041ee1ed94536c320580962d790fc9763bbd0f90e8304080a24c5` | **ORDER granting alternative service** (Garnett, J.), Case Nos. 25-MC-00527; 26-MC-00033; 26-MC-00094 (S.D.N.Y.), filed 2026-05-01. DPRK judgment creditors collecting against crypto assets; Arbitrum Security Council emergency freeze of hack-impacted assets; **N.Y. C.P.L.R. § 5222 restraining notice** on Arbitrum DAO as garnishee; service granted via governance forum + Security Council + counsel (per *Ooki DAO*, *Lido DAO*). The order states the hack transaction occurred **"On April 19, 2026."** **No turnover ordered; no verification standard articulated.** Dkt. 33 (the emergency motion describing the DAO structure) is referenced but not in hand. |
| `2014-12-23-dc-cir-13-7147-opinion-tatel.pdf` | `f726ec732fa55ec31a4d2d6a0ad53b8683c3fd8a0729c0097cdda9fb43c979bf` | Underlying FSIA terrorism-exception judgment, *Han Kim v. DPRK*, D.C. Cir. No. 13-7147 (2014, Tatel, J.), from D.D.C. 1:09-cv-00648. The judgment now being enforced. |

## Tier 2 — Press-reported (CoinDesk summary 2026-05; NOT court-verified)

Useful for context and for understanding the *shape* of the dispute; must not be treated as
adjudicated fact. Key items:

- **Exploit mechanism:** attackers used **unbacked rsETH as collateral on Aave** to borrow
  ~$230M ETH; **30,765 ETH (~$71M)** were frozen by the Arbitrum Security Council. Reporting
  dates the exploit **April 18, 2026**; the court order says **April 19** — *date discrepancy
  to resolve against Dkt. 33 / the complaint.*
- **Parties:** plaintiffs' counsel **Charles Gerstein**, representing families holding
  **~$877M in unpaid terrorism judgments** against North Korea.
- **The contested legal-title question (the crux):** Gerstein argues the exploit was
  **fraud, not theft** — meaning the attackers obtained **legal title** to the ETH by
  deceiving Aave's lending markets with worthless collateral, which would make the ETH
  "North Korean property" reachable by the judgment creditors. **Aave moved to vacate** the
  restraining notice, arguing the assets belong to innocent users and that delay risks
  cascading DeFi liquidations. *(This is the "Aave motion" — a motion to vacate, sought but
  not yet resolved in documents reviewed.)* Ownership is genuinely contested between
  sophisticated parties in active litigation.
- **Governance track:** a non-binding **Snapshot** sentiment check closed ~2026-05-08 with
  **>90% support** to release the 30,765 ETH to a recovery effort. A real transfer would
  require a separate on-chain **Constitutional AIP** and faces Arbitrum's **~8-day L2→L1
  withdrawal delay**. Draft proposal included **indemnification** for the Arbitrum
  Foundation, Offchain Labs, Security Council, and delegates. *(CoinDesk correction
  2026-05-09: the measure was a Snapshot, not a binding AIP.)*
- **Recovery coalition (resolves the KelpDAO/Aave question):** the frozen ETH is earmarked
  for an industry recovery effort led by **Aave, KelpDAO, LayerZero, EtherFi, Compound**.
  So KelpDAO/Aave are the **recovery coalition / litigants**, *not* defendants or garnishees
  in Doc 34. The earlier OA-011 parenthetical that lumped "Arbitrum DAO / Kelp DAO / Aave" as
  the case caption was wrong; the garnishee in Doc 34 is **Arbitrum DAO**.

## Tier 3 — Claimed by a third-party aggregator (unverified, no cryptographic anchor)

- `courtrules.app` (e.g. `/courts/sdny/margaret-m-garnett`) aggregates judge rules/standing
  orders. **Verified what it actually claims (2026-06-11):** it *"check[s] court websites
  continuously… reflects [changes] within days"*, with *"human-in-the-loop review"*, rule→PDF
  page traceability, and timestamped before/after diffs; enforcement data within 24–48h.
  **It does NOT claim hourly verification against court (docket) servers, and provides no
  cryptographic signatures, hashes, or last-verified timestamps.** Source-traceability +
  freshness, not source-binding. See the design note for where this fits the Sibyl ladder.
- `MMG Garnett Civil Case Management Plan (Fillable)- 2025.05.05.pdf` — Judge Garnett's
  standard fillable CMP form (numbered paragraphs, checkboxes, fill-in blanks, Courtroom 906).
  A **deterministic template-classification target**, not a litigation document. See design note.

## Citation status for OA-011

**Docket-verified; substance-read; attribution corrected** (committed 2026-06-11, `06ef16b`).
Doc 34 is a service order in the **restraint phase** and articulates **no** verification
standard for executing an on-chain asset action. OA-011's conjunctive conditions stand on
protocol-design merit, calibrated to the **turnover phase this docket is heading toward**, not
to a holding already issued. The fraud-vs-theft title dispute is the canonical illustration of
why a contested-ownership matter must stay **FN+0 / HL** and must never auto-`PR`.

## Open under MC-005

1. Obtain **Dkt. 33** (emergency motion) and the **Aave motion to vacate** + any ruling on it;
   a turnover order is the only document that could state an actual verification standard.
2. Resolve the **April 18 vs 19** exploit-date discrepancy against primary filings.
3. Confirm the recovery-coalition membership and the Snapshot/AIP timeline against on-chain
   governance records (PCY-origin events) rather than press.
