// Parity harness, TS side (PR-003 / EN-O4). NOT generated — this is the runner.
//
// Replays the golden corpus against the generated `ruleEngine.ts` and exits non-zero on any
// divergence. Run from the workspace root with Node >= 22.6 (native type stripping):
//
//   node exports/engine/runParity.ts [tests/parity/corpus.json]

import { readFileSync } from "node:fs";
import { evaluate, RULESET_SHA256, type Decision, type Inputs } from "./ruleEngine.ts";

interface CorpusCase {
  name: string;
  inputs: Inputs;
  expected: Decision;
}

const corpusPath = process.argv[2] ?? "tests/parity/corpus.json";
const corpus: CorpusCase[] = JSON.parse(readFileSync(corpusPath, "utf8"));

let failures = 0;
for (const c of corpus) {
  const got = evaluate(c.inputs);
  const ok =
    got.action === c.expected.action &&
    got.forced === c.expected.forced &&
    got.rule === c.expected.rule;
  if (!ok) {
    failures += 1;
    console.error(
      `DIVERGENCE ${c.name}: expected ${JSON.stringify(c.expected)}, got ${JSON.stringify(got)}`,
    );
  }
}

if (failures > 0) {
  console.error(`ts parity: ${failures}/${corpus.length} cases diverged (ruleset ${RULESET_SHA256})`);
  process.exit(1);
}
console.log(`ts parity: ${corpus.length}/${corpus.length} cases ok (ruleset ${RULESET_SHA256})`);
