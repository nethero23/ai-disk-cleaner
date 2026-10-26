import { test } from "node:test";
import assert from "node:assert/strict";
import { createLanguageSync, reportMatchesLanguage } from "../src/languageSync.ts";

const deferred = () => {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};
const tick = () => new Promise((resolve) => setImmediate(resolve));

test("an old rules response cannot overwrite a newer language", async () => {
  const ja = deferred();
  const de = deferred();
  const requested = [];
  const shown = [];
  const sync = createLanguageSync("en", async () => {}, (language) => {
    requested.push(language);
    return language === "ja" ? ja.promise : de.promise;
  }, (rules) => shown.push(rules));
  const first = sync.select("ja");
  await tick();
  const second = sync.select("de");
  await tick();
  assert.deepEqual(requested, ["ja", "de"]);
  de.resolve("German rules");
  await second;
  ja.resolve("Japanese rules");
  await first;
  assert.deepEqual(shown, ["German rules"]);
});

test("set_lang writes finish in selection order even when the first is slow", async () => {
  const firstWrite = deferred();
  const writes = [];
  const shown = [];
  const sync = createLanguageSync("en", async (language) => {
    writes.push(language);
    if (language === "ja") await firstWrite.promise;
  }, async (language) => language, (rules) => shown.push(rules));
  const first = sync.select("ja");
  await tick();
  const second = sync.select("de");
  await tick();
  assert.deepEqual(writes, ["ja"]);
  firstWrite.resolve();
  await Promise.all([first, second]);
  assert.deepEqual(writes, ["ja", "de"]);
  assert.deepEqual(shown, ["de"]);
});

test("opening settings cannot publish an obsolete rule list", async () => {
  const old = deferred();
  const shown = [];
  const sync = createLanguageSync("ja", async () => {},
    (language) => language === "ja" ? old.promise : Promise.resolve("de"),
    (rules) => shown.push(rules));
  const opening = sync.refresh();
  await sync.select("de");
  old.resolve("ja");
  await opening;
  assert.deepEqual(shown, ["de"]);
});

test("settings writes preserve selection order and failed writes do not poison the queue", async () => {
  const failed = deferred();
  const persisted = [];
  const sync = createLanguageSync("ja", (language) => {
    persisted.push(language);
    return language === "ja" ? failed.promise : Promise.resolve();
  }, async (language) => language, () => {});
  const first = sync.select("ja");
  const rejection = assert.rejects(first, /disk full/);
  await tick();
  const settings = sync.write(async (language) => persisted.push(`settings:${language}`));
  const latest = sync.select("ar");
  failed.reject(new Error("disk full"));
  await Promise.all([rejection, settings, latest]);
  assert.deepEqual(persisted, ["ja", "settings:ja", "ar"]);
});

test("only the latest rule refresh in one language is published", async () => {
  const old = deferred();
  const shown = [];
  let requests = 0;
  const sync = createLanguageSync("fr", async () => {},
    () => ++requests === 1 ? old.promise : Promise.resolve("updated rules"),
    (rules) => shown.push(rules));
  const first = sync.refresh();
  await sync.refresh();
  old.resolve("old rules");
  await first;
  assert.deepEqual(shown, ["updated rules"]);
});

test("scan completion checks its captured locale against the current UI", () => {
  assert.equal(reportMatchesLanguage({ ui_lang: "ja" }, "de"), false);
  assert.equal(reportMatchesLanguage({ ui_lang: "ar" }, "ar"), true);
  assert.equal(reportMatchesLanguage({}, "en"), true);
  assert.equal(reportMatchesLanguage({ ui_lang: null }, "zh"), true);
  assert.equal(reportMatchesLanguage({}, "ja"), false);
});
