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

    #[test]
    fn every_file_parses_and_names_only_real_rules() {
        let builtin = RuleSet::builtin();
        for (lang, t) in tables() {
            for (id, text) in t {
                assert!(builtin.get(id).is_some(), "{lang}: unknown rule id {id}");
                assert!(text.name.is_some(), "{lang}: {id} has no name");
            }
        }
    }

    #[test]
    fn placeholders_survive_translation() {
        let builtin = RuleSet::builtin();
        for (lang, t) in tables() {
            for (id, text) in t {
                let want = builtin.get(id).unwrap().consequence_en.contains("{size}");
                let have = text
                    .consequence
                    .as_deref()
                    .unwrap_or("{size}")
                    .contains("{size}");
                assert!(!want || have, "{lang}: {id} lost its {{size}} placeholder");
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
