//! The app's own words in the learner's language. Each base language with a
//! table (`content/ui/<code>.txt`, format in `content/FORMAT.md`) translates
//! the English text written in the code; any other base sees the English.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::RwLock;

use chrono::{Datelike, NaiveDate};

type Table = HashMap<String, String>;

/// The tables built into the app, by base language.
const TABLES: &[(&str, &str)] = &[("it", include_str!("../content/ui/it.txt")), ("es", include_str!("../content/ui/es.txt"))];

// One language for the whole interface, set from the learner's choice each
// frame, so drawing code and background messages need not carry it.
static CURRENT: RwLock<Option<(&'static str, &'static Table)>> = RwLock::new(None);

/// Shows the interface in `base` (English when it has no table).
pub fn set_base(base: &str) {
    static LOADED: std::sync::OnceLock<Vec<(&str, Table)>> = std::sync::OnceLock::new();
    let tables = LOADED.get_or_init(|| TABLES.iter().map(|(code, text)| (*code, parse(text))).collect());
    let table = tables.iter().find(|(code, _)| *code == base).map(|(code, t)| (*code, t));
    *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = table;
}

/// Pairs of lines, English then translation, separated by blank lines;
/// `#` starts a comment and `\n` stands for a line break.
fn parse(text: &str) -> Table {
    let lines: Vec<String> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.replace("\\n", "\n"))
        .collect();
    lines
        .split(|l| l.is_empty())
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            assert!(pair.len() == 2, "ui table: an entry is not two lines: {pair:?}");
            (pair[0].clone(), pair[1].clone())
        })
        .collect()
}

/// `en` in the interface's language.
pub fn tr(en: &str) -> &str {
    match *CURRENT.read().unwrap_or_else(|e| e.into_inner()) {
        Some((_, t)) => t.get(en).map(String::as_str).unwrap_or(en),
        None => en,
    }
}

/// A template with numbered places (`{0}`, `{1}`) filled with `args`.
pub fn trf(en: &str, args: &[&dyn Display]) -> String {
    let mut out = tr(en).to_string();
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), &a.to_string());
    }
    out
}

/// `one` or `many`, templates with the count as `{0}`: "1 lesson", "5 lessons".
pub fn plural(n: usize, one: &str, many: &str) -> String {
    trf(if n == 1 { one } else { many }, &[&n])
}

/// A language's name inside a sentence: Italian writes it lowercase ("in inglese").
pub fn in_sentence(name: &str) -> String {
    match *CURRENT.read().unwrap_or_else(|e| e.into_inner()) {
        Some(("it", _)) | Some(("es", _)) => name.to_lowercase(),
        _ => name.to_string(),
    }
}

/// "Monday 23 November".
pub fn long_date(d: NaiveDate) -> String {
    let weekday = [tr("Monday"), tr("Tuesday"), tr("Wednesday"), tr("Thursday"), tr("Friday"), tr("Saturday"), tr("Sunday")];
    let month = [
        tr("January"),
        tr("February"),
        tr("March"),
        tr("April"),
        tr("May"),
        tr("June"),
        tr("July"),
        tr("August"),
        tr("September"),
        tr("October"),
        tr("November"),
        tr("December"),
    ];
    trf(
        "{0} {1} {2}",
        &[&weekday[d.weekday().num_days_from_monday() as usize], &d.day(), &month[d.month0() as usize]],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCES: &[(&str, &str)] = &[
        ("app.rs", include_str!("app.rs")),
        ("appendix.rs", include_str!("appendix.rs")),
        ("audio.rs", include_str!("audio.rs")),
        ("editor.rs", include_str!("editor.rs")),
        ("format.rs", include_str!("format.rs")),
        ("i18n.rs", include_str!("i18n.rs")),
        ("main.rs", include_str!("main.rs")),
        ("model.rs", include_str!("model.rs")),
        ("placement.rs", include_str!("placement.rs")),
        ("practice.rs", include_str!("practice.rs")),
        ("session.rs", include_str!("session.rs")),
        ("store.rs", include_str!("store.rs")),
        ("theme.rs", include_str!("theme.rs")),
        ("update.rs", include_str!("update.rs")),
    ];

    /// The string literals passed straight to `tr(`, `trf(` and `plural(`
    /// (outside nested brackets, so `trf`'s arguments are left out).
    fn keys(src: &str) -> Vec<String> {
        let src = src.split("#[cfg(test)]").next().unwrap_or(src);
        let b = src.as_bytes();
        let mut out = Vec::new();
        for name in ["tr(", "trf(", "plural("] {
            for (at, _) in src.match_indices(name) {
                if at > 0 && (b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_' || b[at - 1] == b'.') {
                    continue;
                }
                let (mut i, mut depth) = (at + name.len(), 1);
                while i < b.len() && depth > 0 {
                    match b[i] {
                        b'(' | b'[' | b'{' => depth += 1,
                        b')' | b']' | b'}' => depth -= 1,
                        b'\'' if b.get(i + 2) == Some(&b'\'') => i += 2,
                        b'"' => {
                            let mut s = String::new();
                            i += 1;
                            while b[i] != b'"' {
                                if b[i] == b'\\' {
                                    i += 1;
                                    match b[i] {
                                        b'n' => s.push('\n'),
                                        c => s.push(c as char),
                                    }
                                    i += 1;
                                } else {
                                    let ch = src[i..].chars().next().unwrap();
                                    s.push(ch);
                                    i += ch.len_utf8();
                                }
                            }
                            if depth == 1 {
                                out.push(s);
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
        }
        out
    }

    fn places(s: &str) -> Vec<String> {
        let mut v: Vec<String> = (0..10).map(|i| format!("{{{i}}}")).filter(|p| s.contains(p.as_str())).collect();
        v.sort();
        v
    }

    #[test]
    fn every_key_is_translated_and_every_entry_used() {
        let used: Vec<(&str, String)> = SOURCES.iter().flat_map(|(f, src)| keys(src).into_iter().map(move |k| (*f, k))).collect();
        assert!(used.len() > 100, "the scan found only {} keys", used.len());
        for (code, text) in TABLES {
            let table = parse(text);
            let lines = text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).count();
            assert_eq!(table.len() * 2, lines, "content/ui/{code}.txt has an English text twice");
            let missing: Vec<_> = used.iter().filter(|(_, k)| !table.contains_key(k)).collect();
            assert!(missing.is_empty(), "content/ui/{code}.txt lacks: {missing:#?}");
            let unused: Vec<_> = table.keys().filter(|k| !used.iter().any(|(_, u)| u == *k)).collect();
            assert!(unused.is_empty(), "content/ui/{code}.txt has entries no code uses: {unused:#?}");
            for (en, t) in &table {
                assert_eq!(places(en), places(t), "content/ui/{code}.txt: the places differ in {en:?}");
            }
        }
    }

    #[test]
    fn lookup_falls_back_to_english() {
        assert_eq!(parse("# c\nOne\nUno\n\nTwo {0}\nDue {0}\n")["Two {0}"], "Due {0}");
        set_base("xx");
        assert_eq!(trf("{0} of {1}", &[&2, &5]), "2 of 5");
        assert_eq!(plural(1, "{0} lesson", "{0} lessons"), "1 lesson");
    }
}
