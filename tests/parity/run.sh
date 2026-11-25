#!/usr/bin/env bash
# The full cross-target parity harness (PR-003 / EN-O4). Run from the workspace root.
# Any failing leg — or any drift between the Rule IR and the committed generated
# artifacts — is a build failure.
set -euo pipefail

echo "── [1/4] drift check: committed artifacts match the Rule IR"
cargo run -q -p tarquin-engine --bin tarquin-engine-gen -- --check

echo "── [2/4] Rust: golden corpus on the reference evaluator"
cargo test -q -p tarquin-engine

echo "── [3/4] Solidity: golden corpus on the generated RuleEngineV01 (forge)"
forge test --root primitives

echo "── [4/4] TypeScript: golden corpus on the generated ruleEngine.ts (node)"
node exports/engine/runParity.ts tests/parity/corpus.json

echo "parity: all targets agree on the golden corpus"
