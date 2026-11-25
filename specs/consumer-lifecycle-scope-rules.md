# Consumer Lifecycle & Scope Rules (LC-013 … LC-016)

Status: live
Owner: Tarquin (Lex.exe consumer boundary)
Source: Arrow RFC-010 Part 4, ratified and implemented gate-side 2026-07-07
(Arrow rules TRG-001 / LFC-007 / LFC-008 live at Arrow commit `967a13a`,
pinned in `imports/arrow/PIN.json`). This document is the Tarquin-normative
home for the consumer-side companions; rule IDs use the `LC` family
(registered in Arrow's `registries/reserved_tokens.json` under NAM-002).

These rules are **specifications for the deterministic runtime** — they
describe checks Lex.exe and the consumer boundary MUST perform. LC-013 and
LC-015 extend the existing hold/ratify discipline; LC-014 and LC-016 are new
obligations that become implementable now that their Arrow gates are live.
Implementation status is tracked per rule below; nothing here authorizes
AI in any runtime path (ADR-0001).

## LC-013 — Lifecycle-Head Check (execution-time re-verification)

Before executing any ratified event, the consumer MUST re-verify against
current lifecycle context that no Class 1 State RV reversal and no Class 4
(CONTROL) supersession has landed in that lifecycle since ratification. If
one has, execution is refused and the item returns to the review queue.

- Depends on: Arrow LFC-007 (live).
- Why: ratification is a decision about the lifecycle as it stood; the
  append-only ledger may have moved. Single-shot execution (e2e_pr_path)
  protects against double-fire; LC-013 protects against firing on a
  superseded world-state.
- Status: **implemented** — `tarquin-lex-exe/src/lifecycle.rs` (`superseded_since_ratification`), enforced in `ExecutionRouter::route` ahead of every dispatch; the caller supplies the ratification watermark in `LifecycleView`.

## LC-014 — TR Routing Scope

On a validated TR event, the consumer MUST resolve
`action_target.target_ref` against its own registered workflow scope table
and route to that workflow — never execute directly (extends LC-002, which
keeps TR non-executable at the consumer). An unresolvable or out-of-scope
`target_ref` produces a hold + review item, never a proceed.

- Depends on: Arrow TRG-001 (live). The gate guarantees envelope shape;
  LC-014 owns resolution — this split is the architect's ratified 2026-07-07
  division of labor.
- Status: **mechanism implemented** — `tarquin-lex-exe/src/scope.rs` (`TrScopeTable::resolve`: route or hold, no execute variant exists by construction). The scope table ships empty; entries are added per deployment when the first TR-consuming workflow lands.

## LC-015 — Post-Closure Consumer Stance

A consumer holding state for a lifecycle that receives a validated Class 6
closure MUST transition that lifecycle to terminal locally, refuse further
execution for it, and accept only LFC-007's two reopening shapes — which
route to human review, never to auto-execution.

- Depends on: Arrow LFC-007 (live).
- Status: **implemented** — `lifecycle.rs` (`lifecycle_closed`), enforced in `ExecutionRouter::route`; a closed lifecycle refuses even a ratified, Sibyl-active, first-execution PR.

## LC-016 — Closure Staleness (passive monitoring)

At reduction and at execution time, the consumer MUST treat a lifecycle
whose closure horizon has passed without a Class 6 closure as **stale**:
degrade confidence in reduction outputs, refuse new execution, and raise a
review item. The horizon is `closure_eta` where the settled event declares
one (LFC-008 makes this mandatory under bounded disciplines) or the
registry-derived bound (`reorg_safety_confirmations` /
`challenge_window_seconds`) otherwise.

- Monitoring is **passive**: staleness is computed at touchpoints from
  explicit inputs (event-carried etas + the invocation-supplied time), never
  by an ambient clock or poller. Deterministic runtime discipline holds.
- Security framing (architect, 2026-07-07): an event that fails to close on
  schedule is possibly under attack — a dormant vulnerability being
  exploited, a withheld settlement. Staleness is an attack indicator, and
  the fail-closed reaction is the defensive posture.
- Horizons: deterministic disciplines are on the order of **hours**;
  `legal_exhaustion` lifecycles are **weeks or longer** and are never
  presumed overdue by wall-clock alone.
- Status: **implemented at the execution path** — `lifecycle.rs`
  (`closure_staleness`, unix_ts horizons; block-count horizons await a
  chain-height input), enforced in `ExecutionRouter::route`. The reduction
  path can call the same pure predicate; wiring it into docverify reduction
  reports is follow-on work. This satisfies the replay-staleness invariant
  Level-3 model-checking waits on.

## TR/CL Primitive Gate — LIFTED CONDITIONALLY

The AGENTS.md gate is satisfied as of 2026-07-07: TRG-001/LFC-007/LFC-008
are live and pinned on the Arrow side, and LC-013/014/015/016 are
implemented above (LC-016's reduction-path wiring is follow-on but the
execution gate — the part the AGENTS.md gate exists to protect — enforces
all four). **TR/CL primitive work may begin.** Dual-target parity
discipline applies from the first primitive; LC-014's scope table only
accepts workflows whose primitives carry certified parity bindings.
