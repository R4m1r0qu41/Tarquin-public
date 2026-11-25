# Provenance — slc-late-payment-001

| Item | Value |
|---|---|
| Commission | MC-004 |
| Drafted | 2026-06-10, AI-assisted drafting (Claude Fable 5) per MC-003 drafting loop; all shipped content deterministic, no runtime model calls |
| Source agreement | None (synthetic clause; pilot defined in `labs/late-payment-pilot.md`) |
| Legal basis | `lex-us-ny-ucc-2-718` (liquidated damages), `lex-us-ny-ucc-2-719` (remedy cap) — verified against NYSenate.gov; enforceability analysis remains human |
| Jurisdiction posture | US; UCC Article 2 frame assumes a sale-of-goods context. Service-contract use needs a common-law liquidated-damages node (unmapped). MX (pena convencional) unmapped. |
| Verification gates | Gate 1 (Concerto validation) not run — no AP toolchain in repo; model authored to Concerto 3.x syntax. Gate 2/3: TypeScript logic + fixtures pass under `node --test` (Node 26 type stripping). |
| Known gaps | Day-count fixed to ACT/365F; no compounding; no partial-payment proration (a partial payment before due date does not reduce the base — declared limitation, see README). |
