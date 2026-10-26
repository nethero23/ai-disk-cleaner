import { test } from "node:test";
import assert from "node:assert/strict";
import { isBehind } from "../src/pages/AiTools/versions.ts";

// The same cases also exercise the Rust comparator in aiconfig/versions.rs.
test("version display matches numeric Rust comparison", () => {
  for (const [pin, latest, expected] of [
    ["1.0.0", "1.2.0", true], ["1.2.0", "1.2.0", false],
    ["1.2", "1.2.0", false], ["1.2.0", "1.2", false],
    ["1.10", "1.2", false], ["1.2", "v1.10.0", true],
    ["latest", "1.2.0", false], ["^1.0", "2.0", false],
  ]) {
    assert.equal(isBehind(pin, latest), expected, `${pin} / ${latest}`);
  }
});
