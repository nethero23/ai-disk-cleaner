#!/usr/bin/env python3
"""Audit current bundled rule translations with Python 3.11+ (stdlib only).

This audit requires complete text for the authored translation baseline.
New bundled/downloaded rules and changed source text use the current runtime
fallback. Rust tests remain the product's CI checks.

Run: python3 tools/audit_rule_i18n.py [--json] [--rules-dir PATH]
Test: python3 -m unittest discover -s tools -p 'test_audit_rule_i18n.py'
"""

import argparse
from collections import Counter
import json
import hashlib
from pathlib import Path
import re
import sys
import tomllib
import unicodedata


LANGUAGES = ("zh-TW", "ja", "ko", "de", "fr", "es", "pt", "ru", "ar", "hi")
FIELDS = ("name", "consequence")


def placeholders(text):
    """Compare all brace tokens including spelling and occurrence counts."""
    return Counter(re.findall(r"\{[^{}]*\}", text))


def audit_table(rules, translations, expected_ids=None):
    errors = []
    expected_ids = rules.keys() if expected_ids is None else expected_ids
    for rule_id in sorted(set(expected_ids) - translations.keys()):
        errors.append(f"{rule_id}: missing translation record")
    for rule_id in sorted(translations.keys() - rules.keys()):
        errors.append(f"{rule_id}: unknown rule id")
    for rule_id in sorted(rules.keys() & translations.keys()):
        text = translations[rule_id]
        if not isinstance(text, dict):
            errors.append(f"{rule_id}: expected a translation table")
            continue
        for field in FIELDS:
            value = text.get(field)
            if not isinstance(value, str) or not value.strip():
                errors.append(f"{rule_id}.{field}: missing or blank string")
                continue
            source = rules[rule_id].get(f"{field}_en", "")
            if placeholders(value) != placeholders(source):
                errors.append(
                    f"{rule_id}.{field}: placeholders differ: "
                    f"expected {dict(placeholders(source))}, "
                    f"got {dict(placeholders(value))}"
                )
            if "\ufffd" in value or "\x00" in value:
                errors.append(f"{rule_id}.{field}: replacement character or NUL")
    return errors


def source_fingerprint(rule):
    digest = hashlib.sha256()
    for field in ("name_zh", "name_en", "consequence_zh", "consequence_en"):
        value = rule.get(field, "").encode("utf-8")
        digest.update(len(value).to_bytes(8, "big"))
        digest.update(value)
    return digest.hexdigest()


def audit_directory(rules_dir):
    rules = {}
    source_errors = []
    for path in sorted(rules_dir.glob("*.toml")):
        for rule in tomllib.loads(path.read_text(encoding="utf-8"))["rule"]:
            rule_id = rule["id"]
            if rule_id in rules:
                source_errors.append(f"duplicate source rule id: {rule_id}")
            rules[rule_id] = rule
            for field in FIELDS:
                if placeholders(rule.get(f"{field}_en", "")) != placeholders(
                    rule.get(f"{field}_zh", "")
                ):
                    source_errors.append(f"{rule_id}.{field}: en/zh tokens differ")
    if not rules:
        source_errors.append("no builtin rules found")
    fingerprint_path = rules_dir / "i18n" / "source_fingerprints.toml"
    authored = tomllib.loads(fingerprint_path.read_text(encoding="utf-8"))
    stale_sources = sorted(rule_id for rule_id, expected in authored.items()
                           if rule_id in rules
                           and source_fingerprint(rules[rule_id]) != expected)
    reports = []
    for language in LANGUAGES:
        path = rules_dir / "i18n" / f"{language}.toml"
        try:
            raw = path.read_text(encoding="utf-8")
            translations = tomllib.loads(raw)
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
            reports.append({"language": language, "errors": [str(error)]})
            continue
        errors = audit_table(rules, translations, authored.keys())
        reports.append({
            "language": language,
            "records": len(translations),
            **{
                f"valid_{field}s": sum(
                    isinstance(text, dict)
                    and isinstance(text.get(field), str)
                    and bool(text[field].strip())
                    for text in translations.values()
                )
                for field in FIELDS
            },
            "placeholder_fields": sum(
                bool(placeholders(text.get(field, "")))
                for text in translations.values() if isinstance(text, dict)
                for field in FIELDS if isinstance(text.get(field), str)
            ),
            # Directional marks are legitimate in Arabic strings; report them
            # separately from parser/translation errors.
            "unicode_format_characters": dict(Counter(
                f"U+{ord(char):04X}" for char in raw
                if unicodedata.category(char) == "Cf"
            )),
            "errors": errors,
        })
    return {"source_rule_count": len(rules), "source_errors": source_errors,
            "stale_sources_using_fallback": stale_sources,
            "new_rules_using_fallback": sorted(rules.keys() - authored.keys()),
            "locales": reports}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--rules-dir", type=Path, default=(
        Path(__file__).resolve().parents[1] / "crates/cleaner-core/rules"
    ))
    args = parser.parse_args()
    try:
        report = audit_directory(args.rules_dir)
    except (OSError, KeyError, UnicodeError, tomllib.TOMLDecodeError) as error:
        print(f"Source rules failed to load: {error}", file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        print(f"Builtin rules: {report['source_rule_count']}")
        for locale in report["locales"]:
            print(f"{locale['language']}: "
                  f"{locale.get('records', 0)} records, "
                  f"{locale.get('valid_names', 0)} names, "
                  f"{locale.get('valid_consequences', 0)} consequences, "
                  f"{len(locale['errors'])} errors")
            for error in locale["errors"]:
                print(f"  {error}")
        if report["new_rules_using_fallback"]:
            print("New rules (runtime fallback): " + ", ".join(
                report["new_rules_using_fallback"]))
        if report["stale_sources_using_fallback"]:
            print("Changed source (runtime fallback): " + ", ".join(
                report["stale_sources_using_fallback"]))
        for error in report["source_errors"]:
            print(f"Source: {error}")
    return int(bool(report["source_errors"]) or any(
        locale["errors"] for locale in report["locales"]
    ))


if __name__ == "__main__":
    sys.exit(main())
