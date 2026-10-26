"""Negative fixtures ensure the on-demand audit detects incomplete translations."""

import copy
import tomllib
import unittest

from audit_rule_i18n import audit_table, source_fingerprint


class TranslationAuditTests(unittest.TestCase):
    def setUp(self):
        self.rules = {"test.cache": {
            "name_en": "Cache",
            "consequence_en": "Re-download {size}",
        }}
        self.translations = {"test.cache": {
            "name": "缓存", "consequence": "重新下载 {size}",
        }}

    def test_valid_unicode_and_escaped_toml(self):
        for name in ["繁體中文", "日本語", "한국어", "Größe", "français",
                     "español", "português", "Русский", "العربية", "हिन्दी"]:
            with self.subTest(name=name):
                text = tomllib.loads(
                    '["test.cache"]\nname = "' + name + '"\n'
                    'consequence = "Été \\"quoted\\" \\\\ {size} ‎--resume"\n'
                )
                self.assertEqual(audit_table(self.rules, text), [])

    def test_missing_record_is_rejected(self):
        self.assertIn("missing translation record",
                      audit_table(self.rules, {})[0])

    def test_missing_or_blank_fields_are_rejected(self):
        for field in ("name", "consequence"):
            for value in (None, "", " \t\n", 12):
                with self.subTest(field=field, value=value):
                    text = copy.deepcopy(self.translations)
                    if value is None:
                        del text["test.cache"][field]
                    else:
                        text["test.cache"][field] = value
                    self.assertTrue(any(
                        f".{field}: missing or blank" in error
                        for error in audit_table(self.rules, text)
                    ))

    def test_missing_renamed_or_duplicate_tokens_are_rejected(self):
        for value in ("下载", "下载 {taille}", "下载 {size} {size}",
                      "下载 {size} {invented}"):
            with self.subTest(value=value):
                self.translations["test.cache"]["consequence"] = value
                self.assertIn("placeholders differ",
                              audit_table(self.rules, self.translations)[0])

    def test_invented_token_in_name_is_rejected(self):
        self.translations["test.cache"]["name"] = "缓存 {size}"
        self.assertIn("test.cache.name: placeholders differ",
                      audit_table(self.rules, self.translations)[0])

    def test_new_rules_may_use_runtime_fallback(self):
        self.rules["future.cache"] = {"name_en": "Future", "consequence_en": "Future"}
        self.assertEqual(audit_table(self.rules, self.translations,
                                     expected_ids={"test.cache"}), [])

    def test_fingerprints_bind_all_source_fields_and_field_boundaries(self):
        rule = {"name_zh": "缓存", "name_en": "Cache",
                "consequence_zh": "下载 {size}",
                "consequence_en": "Download {size}"}
        original = source_fingerprint(rule)
        for field in rule:
            changed = dict(rule)
            changed[field] += " updated"
            self.assertNotEqual(source_fingerprint(changed), original)
        self.assertNotEqual(source_fingerprint({"name_zh": "ab", "name_en": "c"}),
                            source_fingerprint({"name_zh": "a", "name_en": "bc"}))

    def test_unknown_rule_is_rejected(self):
        self.translations["unknown.cache"] = self.translations["test.cache"]
        self.assertIn("unknown.cache: unknown rule id",
                      audit_table(self.rules, self.translations))


if __name__ == "__main__":
    unittest.main()
