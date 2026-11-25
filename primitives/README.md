# Tarquin Solidity Primitives

**Status:** live (stub / scaffolding)
**Owner:** Lex.Codex (curation) + Lex.exe (orchestration)
**ADR:** `decisions/ADR-0001-stack-boundaries-v2.md` — Decision 2 (Solidity primitives are Tarquin-native)

## What this is

The on-chain leg of the Tarquin legal runtime. Accord Project (AP) does **not** target
Solidity; its execution path is off-chain TypeScript. The Solidity / on-chain half of
dual-target compilation is therefore wholly Tarquin's engineering surface — and per ADR-0001
it is the **primary technical risk of the stack**. Audit and test budget concentrate here.

A *primitive* is an audited, minimal, single-purpose on-chain legal building block (a legal
hold, an escrow release condition, a sanctions block) that a compiled Smart Legal Contract
(SLC) composes. Primitives are curated by Lex.Codex from Orlog SLC artifacts and must
stay in **deterministic parity** with the off-chain engine (see
`drafts/engine/tarquin-native-engine.md`).

## Non-negotiable invariants

1. **AI-free at runtime.** No primitive may call out to an oracle that performs inference.
   Inputs are structured attestations (LBRY records) and on-chain state only.
2. **Fail-closed.** The default disposition of every gate is `HL` (hold for human review) or
   `BL` (block). `PR` (proceed) is reachable only when every conjunctive condition is met
   (mirrors LEG Origin Adapter §8 / OA-011).
3. **No autonomous execution.** Nothing executes on the strength of a LEG event before the
   `arrow-consume-leg` consumer has the Arrow disposition **and** a human ratifier has
   confirmed. See `lex-exe/leg-consumer.md`.
4. **Dual-target parity.** Every primitive has a named off-chain counterpart. A primitive
   without a parity test against the off-chain engine MUST NOT be promoted out of `drafts`.

## Layout

```
primitives/
  README.md                         this file
  foundry.toml                      build config (toolchain install pending — see Open Items)
  src/
    interfaces/
      ILegalPrimitive.sol           base interface for all SLC primitives
      IArrowLegConsumer.sol         on-chain consumer surface (fail-closed gate)
    core/
      Types.sol                     six-field classification enums (Action, Finality, ...)
      LegConsumerGate.sol           abstract fail-closed + human-ratification gate
    primitives/
      LegalHold.sol                 example Class 3 CONDITION primitive — mirrors engine default/HL (never PR)
      EscrowRelease.sol             example PR-gated primitive — mirrors engine rule ENG-PR-ELEVATION
  test/                             parity + unit tests (pending)
```

Each `.sol` primitive corresponds to a rule in the native engine (`crates/tarquin-engine`):
`LegalHold` ↔ the fail-closed restriction path (never `PR`); `EscrowRelease` ↔ `ENG-PR-ELEVATION`
(permits only on a `PR` disposition). The engine's `parity_ref` names the counterpart; the parity
harness (PR-003) will assert both targets return identical dispositions on a golden corpus.

## Mapping to Arrow's six-field model

The on-chain types in `src/core/Types.sol` mirror the Arrow classification string
`Class:Origin:State:Action:Finality:Confidence`. The primitives consume the *derived*
disposition from `arrow-consume-leg`; they never re-derive policy on-chain (Arrow's Step 5b
gate and Lex.exe own derivation). The gate stores only what it must to enforce fail-closed
behaviour.

## Open Items

| ID | Item |
|---|---|
| PR-001 | ✅ Done — Foundry toolchain pinned (solc 0.8.28). `forge build` + `forge test` green (5 tests). |
| PR-002 | Define the LBRY `proof_ref` → on-chain proof-commitment encoding (hash anchoring). |
| PR-003 | Parity-test harness binding each `.sol` primitive to its off-chain engine counterpart. |
| PR-004 | Decide upgrade model and disclose it per RFC-007 INT-003 (`integration_strength`). |
| PR-005 | Class-to-subtype compatibility matrix depends on Arrow LEG-007 completion (OI-001 / OI-003). |
