//! The UI's language: English (the source language) and Traditional Chinese (Taiwan).
//!
//! DeckCraft upstream ships English only; this fork adds a thin translation layer that lives
//! entirely in this crate. Every UI string stays English in the code and is translated at the
//! point where it is drawn, through [`tr`] (plain strings), [`tr_id`] (command labels, keyed by
//! command id so the engine's own labels never change) and the `trf!` macro (`format!`-style
//! templates whose `{}` placeholders must line up with the English original). A string with no
//! translation falls back to English, so a missing entry can never break the UI.
//!
//! The catalogue is `i18n/zh_hant.tsv`, one `key<TAB>translation` per line, keyed by the English
//! string itself; command-id entries use the key `id:<command-id>`. Inside a key or translation,
//! `\n`, `\t` and `\\` are escapes (two-line ribbon labels contain a real newline).

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

/// An interface language.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    ZhHant,
}

impl Lang {
    /// The stored form (UiState's `language`, the `DECKCRAFT_LANG` variable).
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::ZhHant => "zh-Hant",
        }
    }

    /// Parse a language tag (`zh-Hant`, `zh_TW`, `zh-HK`, `en-US`…). Traditional-Chinese tags
    /// (script `Hant`, or the TW/HK/MO regions, or a bare `zh`) resolve to [`Lang::ZhHant`];
    /// Simplified tags (`Hans`, CN, SG) stay English in this fork.
    pub fn parse(tag: &str) -> Option<Lang> {
        let t = tag.trim().trim_matches('"');
        let lower = t.to_ascii_lowercase();
        let norm = lower.replace('_', "-");
        if norm == "en" || norm.starts_with("en-") || norm.starts_with("en_") {
            return Some(Lang::En);
        }
        if norm.starts_with("zh") {
            if norm.contains("hans") || norm.contains("-cn") || norm.contains("-sg") {
                return Some(Lang::En);
            }
            return Some(Lang::ZhHant);
        }
        None
    }

    /// The language a fresh start should use: `DECKCRAFT_LANG` first, then the locale variables,
    /// then (on the web) the browser's language. A language saved in the UI state wins over this
    /// once the host has loaded it, because `SlideApp::logic` re-applies the saved value.
    pub fn detect() -> Lang {
        #[cfg(not(target_arch = "wasm32"))]
        {
            for var in ["DECKCRAFT_LANG", "LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
                if let Ok(v) = std::env::var(var) {
                    for part in v.split(':') {
                        if let Some(l) = Lang::parse(part) {
                            return l;
                        }
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(l) = navigator_language().and_then(|t| Lang::parse(&t)) {
                return l;
            }
        }
        Lang::En
    }
}

#[cfg(target_arch = "wasm32")]
fn navigator_language() -> Option<String> {
    let global = js_sys::global();
    let nav = js_sys::Reflect::get(&global, &js_sys::JsString::from("navigator")).ok()?;
    let lang = js_sys::Reflect::get(&nav, &js_sys::JsString::from("language")).ok()?;
    lang.as_string()
}

static LANG: AtomicU8 = AtomicU8::new(0);

/// The interface language currently in effect.
pub fn lang() -> Lang {
    if LANG.load(Ordering::Relaxed) == 1 { Lang::ZhHant } else { Lang::En }
}

/// Switch the interface language. Callers also reinstall the fonts (the Traditional face leads
/// the UI font family only in Traditional Chinese) and repaint.
pub fn set_lang(l: Lang) {
    LANG.store(if l == Lang::ZhHant { 1 } else { 0 }, Ordering::Relaxed);
}

const ZH_HANT_TSV: &str = include_str!("i18n/zh_hant.tsv");

fn catalog() -> &'static HashMap<&'static str, &'static str> {
    static CATALOG: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut map = HashMap::new();
        for line in ZH_HANT_TSV.lines() {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('\t') {
                // Keys stay in their escaped file form (tr escapes its query the same way);
                // translations are unescaped once and leaked to 'static.
                let value: &'static str = Box::leak(unescape(value).into_boxed_str());
                map.insert(key, value);
            }
        }
        map
    })
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t")
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Translate a UI string. English (or an unknown string) comes back unchanged.
pub fn tr(s: &str) -> &str {
    if lang() != Lang::ZhHant || s.is_empty() {
        return s;
    }
    let map = catalog();
    if let Some(v) = map.get(s) {
        return v;
    }
    if s.contains(['\\', '\n', '\t'])
        && let Some(v) = map.get(escape(s).as_str())
    {
        return v;
    }
    s
}

/// Translate a command's label: the `id:<id>` entry when the catalogue has one (command labels
/// repeat a lot — "Delete" is several commands — and ids disambiguate), else [`tr`] of the label.
pub fn tr_id<'a>(id: &str, fallback_label: &'a str) -> &'a str {
    if lang() == Lang::ZhHant {
        let key = format!("id:{id}");
        if let Some(v) = catalog().get(key.as_str()) {
            return v;
        }
    }
    tr(fallback_label)
}

/// [`tr`] a `format!`-style template, then fill its `{}` placeholders in order.
pub fn trf(template: &str, args: &[&dyn Display]) -> String {
    let t = tr(template);
    let mut out = String::with_capacity(t.len() + 16);
    let mut rest = t;
    for a in args {
        match rest.split_once("{}") {
            Some((head, tail)) => {
                out.push_str(head);
                out.push_str(&a.to_string());
                rest = tail;
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

/// `trf` with the ergonomics of `format!`: `trf!("Saved {}", path)`.
#[macro_export]
macro_rules! trf {
    ($template:expr, $($arg:expr),+ $(,)?) => {
        $crate::i18n::trf($template, &[$( & $arg as &dyn ::std::fmt::Display ),+])
    };
    ($template:expr) => {
        $crate::i18n::tr($template).to_string()
    };
}

/// Serializes the tests that flip the process-wide language (here and the UI tests in `lib.rs`),
/// so parallel test threads cannot change each other's language mid-assertion.
#[cfg(test)]
pub(crate) static TEST_LANG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn language_tags_resolve() {
        assert_eq!(Lang::parse("zh-Hant"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh_TW.UTF-8"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh-HK"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh-MO"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh-Hans"), Some(Lang::En));
        assert_eq!(Lang::parse("zh-CN"), Some(Lang::En));
        assert_eq!(Lang::parse("en"), Some(Lang::En));
        assert_eq!(Lang::parse("en-US"), Some(Lang::En));
        assert_eq!(Lang::parse("fr-FR"), None);
        assert_eq!(Lang::parse(""), None);
    }

    #[test]
    fn english_is_the_identity() {
        let _guard = TEST_LANG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_lang(Lang::En);
        assert_eq!(tr("Home"), "Home");
        assert_eq!(tr_id("edit.cut", "Cut"), "Cut");
        assert_eq!(trf("Slide {} of {}", &[&2, &9]), "Slide 2 of 9");
    }

    #[test]
    fn traditional_translates_and_falls_back() {
        let _guard = TEST_LANG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_lang(Lang::ZhHant);
        assert_eq!(tr("Home"), "常用");
        assert_eq!(tr("New\nSlide"), "新增\n投影片");
        assert_eq!(tr("No such string anywhere"), "No such string anywhere");
        assert_eq!(trf("Slide {} of {}", &[&2, &9]), "投影片 2 / 9");
        assert_eq!(trf("Saved {}", &[&"/tmp/a.deckcraft"]), "已儲存 /tmp/a.deckcraft");
        set_lang(Lang::En);
        assert_eq!(tr("Home"), "Home");
    }
}
