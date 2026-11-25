# Tarquin Architecture Decision Records

Go-forward identifier scheme: `ADR-TARQ-NNN` (per Arrow ADR-ARRW-001 §2.4). Cross-repository
governance is owned by Arrow; Tarquin records repository-local adoption that references the
governing Arrow ADR rather than copying it.

## Active

| ADR | Title | Status | Date |
|---|---|---|---|
| [ADR-TARQ-001](ADR-TARQ-001-adopt-arrow-governance.md) | Adopt Arrow Governance (ADR-ARRW-001) | live | 2026-06-19 |
| [ADR-0001](ADR-0001-stack-boundaries-v2.md) | Tarquin, Accord, and Arrow Boundaries | live | 2026-06-07 |
| [ADR-0003](ADR-0003-interface-pinning-v1.md) | Interface Pinning Across the Three Legs | live | 2026-06-14 |

## Superseded

| ADR | Title | Status | Superseded by |
|---|---|---|---|
| [ADR-0002](ADR-0002-v1.6-reboot-and-liveness-v1.md) | v1.6 Soft Reboot: Liveness Convention & Archive Discipline | superseded (governance scope) | Arrow ADR-ARRW-001 |

## Notes

The pre-existing `ADR-000N` records are preserved under their original identifiers as
historical decision records and are **not** renumbered into the `ADR-TARQ-NNN` scheme
(ADR-ARRW-001 ADR-003). Their bodies retain era-correct terminology (e.g. the former name
"Contract Labs" and pre-retirement filenames); current material uses the active names.
