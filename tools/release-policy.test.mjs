import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { releasePolicy } from "./release-policy.mjs";

test("candidate publishes NSIS only; stable keeps NSIS/MSI", () => {
  assert.deepEqual(releasePolicy("v0.8.0-rc.1", ["0.8.0-rc.1"]), { prerelease: true, bundles: "nsis" });
  assert.deepEqual(releasePolicy("v0.8.0", ["0.8.0"]), { prerelease: false, bundles: "nsis,msi" });
});

test("release policy rejects malformed tags and mismatched application versions", () => {
  for (const tag of ["0.8.0", "v0.8", "v0.8.0\n", "v0.8.0/other", "v0.8.0-rc.1\nINJECTED=value"]) {
    assert.throws(() => releasePolicy(tag, [tag.slice(1)]));
  }
  assert.throws(() => releasePolicy("v0.8.0", []));
  assert.throws(() => releasePolicy("v0.8.0", ["0.8.0", "0.7.1"]));
});

test("workflow never promotes candidates to latest or generates stable updater/manifests", () => {
  const workflow = readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
  assert.match(workflow, /needs: core/);
  assert.match(workflow, /node tools\/release-policy\.mjs "\$tag"/);
  assert.match(workflow, /prerelease: \$\{\{ env\.RELEASE_PRERELEASE == 'true' \}\}/);
  assert.match(workflow, /make_latest: \$\{\{ env\.RELEASE_PRERELEASE == 'true' && 'false' \|\| 'true' \}\}/);
  for (const step of ["Updater manifest", "winget and Scoop manifests"]) {
    const block = workflow.slice(workflow.indexOf(`- name: ${step}`)).split(/\n      - /)[0];
    assert.match(block, /env\.RELEASE_PRERELEASE != 'true'/);
  }
});
