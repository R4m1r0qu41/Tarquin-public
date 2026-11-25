# Arrow Imports

This directory is reserved for Arrow-side pins or fixtures that Tarquin needs to consume back
from Arrow.

Current pinned Arrow import:

```text
PIN.json
  -> Dev/Arrow/exports/rule-surface.v1.6.0.json
  -> sha256:347299b13271b846485d309fc7ff334810f92233c0a0130415381e9b74d9dcbb
```

The primary Tarquin-to-Arrow contract currently lives in:

```text
exports/arrow/leg-shared-surface.v0.2.json
exports/arrow/leg-shared-surface.v0.2.sha256
exports/arrow/leg-shared-surface.v0.2.md
```

Expected loop:

```text
Tarquin exports shared LEG surface
  -> Arrow imports and pins the hash
  -> Arrow publishes any accepted/pinned return metadata here when Tarquin needs it
  -> Tarquin pins Arrow return metadata in imports/arrow/PIN.json
```

Do not store generated dependency trees or vendored Arrow source here. Tarquin continues to use
Arrow crates through Cargo path dependencies.
