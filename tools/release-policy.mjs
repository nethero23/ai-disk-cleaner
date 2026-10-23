import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export function releasePolicy(tag, versions) {
  if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?$/.test(tag)) {
    throw new Error("Release tag must be vMAJOR.MINOR.PATCH with an optional prerelease");
  }
  if (!versions.length || versions.some(version => version !== tag.slice(1))) {
    throw new Error("Release tag must match every application version");
  }
  const prerelease = tag.includes("-");
  return { prerelease, bundles: prerelease ? "nsis" : "nsis,msi" };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const version = file => JSON.parse(readFileSync(file, "utf8")).version;
  const rust = readFileSync("Cargo.toml", "utf8").match(/^version = "([^"]+)"/m)?.[1];
  const policy = releasePolicy(process.argv[2], [version("package.json"), version("src-tauri/tauri.conf.json"), rust]);
  console.log(`RELEASE_TAG=${process.argv[2]}\nRELEASE_PRERELEASE=${policy.prerelease}\nRELEASE_BUNDLES=${policy.bundles}`);
}
