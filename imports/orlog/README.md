# Orlog Pinned Imports

Pinned, hash-verified copies of Orlog SLC bundles (Orlog was formerly named Contract Labs;
the `source_commit` recorded in `PIN.json` predates the rename but resolves in the same
repository history). This directory is the Tarquin side of the MC-006 handoff contract:

```text
Orlog slc/<artifact-id>/   (authoring; sealed by artifact_hash)
  -> imports/orlog/<artifact-id>/   (pinned copy at a named commit)
  -> hash verification (recompute MC-006 artifact_hash; reject drift)
  -> Lex.Codex recomputation (fixture case recompute; reject result drift)
  -> Lex.exe / LBRY related_slc_artifacts reference
```

Rules:

- Bundles are **copies at a pinned commit** (recorded in `PIN.json`), never live path
  dependencies. Orlog does not emit Arrow events; Tarquin does not author SLCs.
- A bundle MUST NOT be used unless its recomputed `artifact_hash` matches both the bundle's
  own `manifest.json` and the pin entry. Verification is implemented in
  `tarquin-lex-exe::lexcodex` and exercised by `tests/e2e_slc_handoff.rs`.
- Downstream references use `{artifact_id, version, artifact_hash, fixture_case,
  computed_result_hash}` (the `related_slc_artifacts` schema), never prose case names.
