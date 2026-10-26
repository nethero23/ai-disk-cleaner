//! Rule names and consequences in UI languages beyond the `name_zh`/`name_en`
//! pair kept in the rule files themselves. One file per language in
//! `rules/i18n/<lang>.toml`, keyed by rule id; a missing entry falls back to
//! English (or Chinese for `zh-TW`, whose fallback reads closer).

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuleText {
    pub name: Option<String>,
    pub consequence: Option<String>,
}

const FILES: &[(&str, &str)] = &[
    ("zh-TW", include_str!("../rules/i18n/zh-TW.toml")),
    ("ja", include_str!("../rules/i18n/ja.toml")),
    ("ko", include_str!("../rules/i18n/ko.toml")),
    ("de", include_str!("../rules/i18n/de.toml")),
    ("fr", include_str!("../rules/i18n/fr.toml")),
    ("es", include_str!("../rules/i18n/es.toml")),
    ("pt", include_str!("../rules/i18n/pt.toml")),
    ("ru", include_str!("../rules/i18n/ru.toml")),
    ("ar", include_str!("../rules/i18n/ar.toml")),
    ("hi", include_str!("../rules/i18n/hi.toml")),
];

type Table = HashMap<String, RuleText>;

fn tables() -> &'static HashMap<&'static str, Table> {
    static T: OnceLock<HashMap<&'static str, Table>> = OnceLock::new();
    T.get_or_init(|| {
        FILES
            .iter()
            .map(|(lang, text)| {
                let t: Table = toml::from_str(text)
                    .unwrap_or_else(|e| panic!("rules/i18n/{lang}.toml is invalid: {e}"));
                (*lang, t)
            })
            .collect()
    })
}

/// The translated text for `rule_id` in `lang`, if this build has one.
pub fn lookup(lang: &str, rule_id: &str) -> Option<&'static RuleText> {
    tables().get(lang)?.get(rule_id)
}

/// Source text at the time the bundled translations were authored. This is
/// independent of the current builtin rules, which may have changed since.
fn source_fingerprints() -> &'static HashMap<String, String> {
    static SOURCE: OnceLock<HashMap<String, String>> = OnceLock::new();
    SOURCE.get_or_init(|| {
        toml::from_str(include_str!("../rules/i18n/source_fingerprints.toml"))
            .expect("invalid bundled translation source fingerprints")
    })
}

fn source_fingerprint(rule: &crate::rules::Rule) -> String {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    for value in [
        &rule.name_zh,
        &rule.name_en,
        &rule.consequence_zh,
        &rule.consequence_en,
    ] {
        digest.update((value.len() as u64).to_be_bytes());
        digest.update(value.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

/// A same-id override or newer builtin may change what deletion means. Never
/// apply an older translation to changed source text; use the current fallback.
pub fn lookup_for_rule(lang: &str, rule: &crate::rules::Rule) -> Option<&'static RuleText> {
    if source_fingerprints().get(&rule.id)? != &source_fingerprint(rule) {
        return None;
    }
    lookup(lang, &rule.id)
}

/// The language whose built-in strings a UI language falls back to.
pub fn core_lang(lang: &str) -> &'static str {
    if lang.starts_with("zh") {
        "zh"
    } else {
        "en"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::RuleSet;

    fn placeholders(text: &str) -> Vec<String> {
        let pattern = regex::Regex::new(r"\{[^{}]*\}").unwrap();
        let mut tokens: Vec<_> = pattern
            .find_iter(text)
            .map(|token| token.as_str().to_owned())
            .collect();
        tokens.sort();
        tokens
    }

    fn validate_table(table: &Table, builtin: &RuleSet) -> Result<(), String> {
        // New builtin/downloaded rules may still fall back. Every rule in the
        // authored translation baseline must remain complete in every locale.
        for id in source_fingerprints().keys() {
            if !table.contains_key(id) {
                return Err(format!("{id}: missing translation record"));
            }
        }
        for (id, text) in table {
            let source = builtin
                .get(id)
                .ok_or_else(|| format!("{id}: unknown rule id"))?;
            for (field, value, original) in [
                ("name", text.name.as_deref(), source.name_en.as_str()),
                (
                    "consequence",
                    text.consequence.as_deref(),
                    source.consequence_en.as_str(),
                ),
            ] {
                let value = value
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| format!("{id}.{field}: missing or blank string"))?;
                // A changed source is intentionally not translated at runtime.
                if source_fingerprints().get(id) == Some(&source_fingerprint(source))
                    && placeholders(value) != placeholders(original)
                {
                    return Err(format!("{id}.{field}: placeholders differ"));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn every_file_has_complete_text_for_every_baseline_rule() {
        let builtin = RuleSet::builtin();
        assert_eq!(FILES.len(), 10);
        assert!(source_fingerprints().len() >= 126);
        for (lang, raw) in FILES {
            let table: Table =
                toml::from_str(raw).unwrap_or_else(|error| panic!("{lang}: invalid TOML: {error}"));
            validate_table(&table, &builtin).unwrap_or_else(|error| panic!("{lang}: {error}"));
        }
    }

    #[test]
    fn validation_rejects_missing_and_blank_names_and_consequences() {
        let builtin = RuleSet::builtin();
        for field in ["name", "consequence"] {
            for value in [None, Some(""), Some(" \t\n")] {
                let mut table = tables()["ja"].clone();
                let text = table.get_mut("pip.cache").unwrap();
                let slot = if field == "name" {
                    &mut text.name
                } else {
                    &mut text.consequence
                };
                *slot = value.map(str::to_owned);
                let error = validate_table(&table, &builtin).unwrap_err();
                assert!(error.contains(&format!("pip.cache.{field}: missing or blank")));
            }
        }
    }

    #[test]
    fn validation_rejects_missing_and_unknown_rule_records() {
        let builtin = RuleSet::builtin();
        let mut table = tables()["ja"].clone();
        let text = table.remove("pip.cache").unwrap();
        assert!(validate_table(&table, &builtin)
            .unwrap_err()
            .contains("pip.cache: missing translation record"));
        table.insert("pip.cache".into(), text.clone());
        table.insert("no.such.rule".into(), text);
        assert!(validate_table(&table, &builtin)
            .unwrap_err()
            .contains("no.such.rule: unknown rule id"));
    }

    #[test]
    fn validation_rejects_missing_renamed_extra_and_duplicate_placeholders() {
        let builtin = RuleSet::builtin();
        let source = builtin
            .rules
            .iter()
            .find(|rule| rule.consequence_en.contains("{size}"))
            .unwrap();
        for value in [
            "download",
            "download {taille}",
            "{size} {size}",
            "{size} {extra}",
        ] {
            let mut table = tables()["ja"].clone();
            table.get_mut(&source.id).unwrap().consequence = Some(value.into());
            assert!(validate_table(&table, &builtin)
                .unwrap_err()
                .contains(&format!("{}.consequence: placeholders differ", source.id)));
        }
        let mut table = tables()["ja"].clone();
        table.get_mut("pip.cache").unwrap().name = Some("Cache {invented}".into());
        assert!(validate_table(&table, &builtin)
            .unwrap_err()
            .contains("pip.cache.name: placeholders differ"));
    }

    #[test]
    fn unicode_and_escaped_strings_round_trip_through_toml() {
        for name in [
            "繁體中文",
            "日本語",
            "한국어",
            "Größe",
            "français",
            "español",
            "português",
            "Русский",
            "العربية",
            "हिन्दी",
        ] {
            let raw = format!(
                r#"["test.cache"]
name = "{name}"
consequence = "Été \"quoted\" \\ {{size}} ‎--resume"
"#
            );
            let table: Table = toml::from_str(&raw).unwrap();
            let text = &table["test.cache"];
            assert_eq!(text.name.as_deref(), Some(name));
            assert_eq!(
                text.consequence.as_deref(),
                Some("Été \"quoted\" \\ {size} ‎--resume")
            );
        }
    }

    #[test]
    fn changed_builtin_source_uses_current_fallback_instead_of_old_translation() {
        let builtin = RuleSet::builtin();
        for rule in &builtin.rules {
            let unchanged = source_fingerprints().get(&rule.id) == Some(&source_fingerprint(rule));
            for (lang, _) in FILES {
                assert_eq!(
                    lookup_for_rule(lang, rule).is_some(),
                    unchanged,
                    "{lang}: {} translation source changed",
                    rule.id
                );
            }
        }
    }

    #[test]
    fn scan_uses_the_ui_language() {
        use crate::scan::{rule_name, ScanOptions};
        let builtin = RuleSet::builtin();
        let rule = builtin.get("pip.cache").unwrap();
        let mut opts = ScanOptions {
            lang: "en".into(),
            ui_lang: "ja".into(),
            ..ScanOptions::default()
        };
        assert_eq!(rule_name(rule, &opts), "pip のキャッシュ");
        opts.ui_lang = "en".into();
        assert_eq!(rule_name(rule, &opts), rule.name_en);
    }

    #[test]
    fn falls_back_to_none_for_unknown() {
        assert!(lookup("ja", "no.such.rule").is_none());
        assert!(lookup("xx", "pip.cache").is_none());
        assert_eq!(core_lang("zh-TW"), "zh");
        assert_eq!(core_lang("ja"), "en");
    }
}
