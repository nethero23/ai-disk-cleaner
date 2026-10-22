import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import ts from "typescript";

// Parse, never execute, locale modules. Both source languages and every
// translated stale-version hint must ask for review, not promise safe removal.
function staleHints(path) {
  const source = ts.createSourceFile(String(path), readFileSync(path, "utf8"), ts.ScriptTarget.Latest, true);
  const hints = [];
  function visit(node) {
    if (ts.isPropertyAssignment(node) && node.name.getText(source) === "aiKindHint") {
      let object = node.initializer;
      while (ts.isAsExpression(object)) object = object.expression;
      assert.ok(ts.isObjectLiteralExpression(object));
      const hint = object.properties.find((p) => ts.isPropertyAssignment(p) && p.name.getText(source) === "stale_versions");
      assert.ok(hint && ts.isStringLiteral(hint.initializer), `${path}: missing stale-version hint`);
      hints.push(hint.initializer.text);
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  return hints;
}

test("all stale-version translations require review instead of promising safe deletion", () => {
  const root = new URL("../src/", import.meta.url);
  const paths = [new URL("i18n.ts", root), ...readdirSync(new URL("locales/", root)).filter((f) => f.endsWith(".ts")).map((f) => new URL(`locales/${f}`, root))];
  let count = 0;
  for (const path of paths) {
    const hints = staleHints(path);
    assert.equal(hints.length, path.pathname.endsWith("/i18n.ts") ? 2 : 1, String(path));
    for (const hint of hints) {
      assert.match(hint, /确认|確認|verify usage|prüfen|vérifiez|compruébelo|verifique|проверьте|تحقق|जाँचें|확인/iu, `${path}: must require usage review`);
      count++;
    }
  }
  assert.equal(count, 12);
});
