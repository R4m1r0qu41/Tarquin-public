import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { evaluateLatePayment } from "../logic/logic.ts";

const fixturesDir = join(dirname(fileURLToPath(import.meta.url)), "..", "fixtures");
const fixture = (name: string) =>
  JSON.parse(readFileSync(join(fixturesDir, name), "utf8"));

test("valid fixtures compute the agreed obligations", () => {
  for (const c of fixture("valid.json").cases) {
    assert.deepEqual(evaluateLatePayment(c.terms, c.event), c.expected, c.name);
  }
});

test("invalid fixtures are rejected with RangeError", () => {
  for (const c of fixture("invalid.json").cases) {
    assert.throws(
      () => evaluateLatePayment(c.terms, c.event),
      (err: unknown) =>
        err instanceof RangeError && String(err).includes(c.expectErrorMatch),
      c.name
    );
  }
});

test("disputed fixtures: computation deterministic, entitlement flagged", () => {
  for (const c of fixture("disputed.json").cases) {
    assert.deepEqual(evaluateLatePayment(c.terms, c.event), c.expected, c.name);
    assert.equal(c.dispute.requiresHumanReview, true, `${c.name}: review flag`);
    assert.equal(c.dispute.computationContested, false, `${c.name}: computation agreed`);
  }
});
