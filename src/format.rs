//! Reads and writes the plain-text lesson format described in content/FORMAT.md.

use crate::model::{Course, Item, Lesson, Level};

#[derive(Clone, PartialEq)]
enum Section {
    None,
    Dialogue,
    Notes(String),
    Ex1,
    Ex2,
}

fn section_of(t: &str) -> Option<Section> {
    let key = t.strip_suffix(':')?;
    match key {
        "dialogue" => Some(Section::Dialogue),
        "ex1" => Some(Section::Ex1),
        "ex2" => Some(Section::Ex2),
        "notes" => Some(Section::Notes(String::new())),
        _ => key.strip_prefix("notes.").map(|l| Section::Notes(l.to_string())),
    }
}

/// `key: value` or `key.lang: value`.
fn key_value(t: &str) -> Option<(&str, &str)> {
    let (k, v) = t.split_once(':')?;
    let k = k.trim();
    if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_') {
        return None;
    }
    Some((k, v.trim()))
}

pub fn parse_course(src: &str, id: &str) -> Result<Course, String> {
    let mut course = Course {
        id: id.to_string(),
        ..Default::default()
    };
    let mut lesson: Option<Lesson> = None;
    let mut section = Section::None;

    let finish = |lesson: &mut Option<Lesson>, course: &mut Course| {
        if let Some(mut l) = lesson.take() {
            for n in l.notes.values_mut() {
                *n = n.trim_matches('\n').to_string();
            }
            course.lessons.push(l);
        }
    };

    for (i, raw) in src.lines().enumerate() {
        let n = i + 1;
        let line = raw.trim_end();
        let t = line.trim();

        if let Section::Notes(lang) = &section {
            let ends = t.starts_with("==") || section_of(t).is_some();
            if !ends {
                if let Some(l) = lesson.as_mut() {
                    let note = l.notes.entry(lang.clone()).or_default();
                    note.push_str(line);
                    note.push('\n');
                }
                continue;
            }
        }

        if t.is_empty() || t.starts_with('#') {
            continue;
        }

        if let Some(rest) = t.strip_prefix("==") {
            finish(&mut lesson, &mut course);
            let mut words = rest.split_whitespace();
            let number = words
                .next()
                .and_then(|w| w.parse::<u32>().ok())
                .ok_or(format!("line {n}: a lesson starts with `== <number> <level>`"))?;
            let level = match words.next() {
                Some(w) => Level::parse(w).ok_or(format!("line {n}: unknown level `{w}` (use A1 A2 B1 B2 C1 C2)"))?,
                None => Level::A1,
            };
            lesson = Some(Lesson {
                number,
                level,
                ..Default::default()
            });
            section = Section::None;
            continue;
        }

        if let Some(s) = section_of(t) {
            if lesson.is_none() {
                return Err(format!("line {n}: `{t}` must come after a `== <number> <level>` line"));
            }
            section = s;
            continue;
        }

        if t == "-" || t.starts_with("- ") {
            let text = t[1..].trim().to_string();
            let item = Item {
                text,
                ..Default::default()
            };
            let l = lesson.as_mut().ok_or(format!("line {n}: sentence outside a lesson"))?;
            match section {
                Section::Dialogue => l.dialogue.push(item),
                Section::Ex1 => l.ex1.push(item),
                Section::Ex2 => l.ex2.push(item),
                _ => return Err(format!("line {n}: sentences go under `dialogue:`, `ex1:` or `ex2:`")),
            }
            continue;
        }

        let Some((key, value)) = key_value(t) else {
            return Err(format!("line {n}: not understood: `{t}`"));
        };

        let indented = raw.starts_with(' ') || raw.starts_with('\t');
        if indented && matches!(section, Section::Dialogue | Section::Ex1 | Section::Ex2) {
            let l = lesson.as_mut().expect("section implies lesson");
            let list = match section {
                Section::Dialogue => &mut l.dialogue,
                Section::Ex1 => &mut l.ex1,
                _ => &mut l.ex2,
            };
            let item = list.last_mut().ok_or(format!("line {n}: `{key}:` comes before any `- sentence`"))?;
            if key == "pron" {
                item.pron = value.to_string();
            } else {
                item.tr.insert(key.to_string(), value.to_string());
            }
            continue;
        }

        match lesson.as_mut() {
            None => match key {
                "course" => course.lang = value.to_string(),
                "name" => course.name = value.to_string(),
                _ => match key.strip_prefix("name.") {
                    Some(l) => {
                        course.name_tr.insert(l.to_string(), value.to_string());
                    }
                    None => return Err(format!("line {n}: unknown course key `{key}`")),
                },
            },
            Some(l) => match key {
                "title" => l.title = value.to_string(),
                "audio" => l.audio = value.to_string(),
                "level" => l.level = Level::parse(value).ok_or(format!("line {n}: unknown level `{value}`"))?,
                _ => match key.strip_prefix("title.") {
                    Some(lang) => {
                        l.title_tr.insert(lang.to_string(), value.to_string());
                    }
                    None => return Err(format!("line {n}: unknown lesson key `{key}`")),
                },
            },
        }
    }
    finish(&mut lesson, &mut course);

    if course.lang.is_empty() {
        return Err("the file must start with `course: <language code>`".into());
    }
    if course.name.is_empty() {
        course.name = crate::model::lang_name(&course.lang, "en");
    }
    course.sort();
    Ok(course)
}

fn write_items(out: &mut String, header: &str, items: &[Item]) {
    if items.is_empty() {
        return;
    }
    out.push_str(header);
    out.push_str(":\n");
    for it in items {
        out.push_str(&format!("- {}\n", it.text.trim()));
        for (lang, tr) in &it.tr {
            if !tr.trim().is_empty() {
                out.push_str(&format!("  {lang}: {}\n", tr.trim()));
            }
        }
        if !it.pron.trim().is_empty() {
            out.push_str(&format!("  pron: {}\n", it.pron.trim()));
        }
    }
    out.push('\n');
}

pub fn write_course(c: &Course) -> String {
    let mut out = String::new();
    out.push_str("# Lesson file — format explained in FORMAT.md\n");
    out.push_str(&format!("course: {}\nname: {}\n", c.lang, c.name));
    for (l, n) in &c.name_tr {
        out.push_str(&format!("name.{l}: {n}\n"));
    }
    out.push('\n');
    for l in &c.lessons {
        out.push_str(&format!("== {} {}\n", l.number, l.level.name()));
        out.push_str(&format!("title: {}\n", l.title.trim()));
        for (lang, t) in &l.title_tr {
            if !t.trim().is_empty() {
                out.push_str(&format!("title.{lang}: {}\n", t.trim()));
            }
        }
        if !l.audio.trim().is_empty() {
            out.push_str(&format!("audio: {}\n", l.audio.trim()));
        }
        out.push('\n');
        write_items(&mut out, "dialogue", &l.dialogue);
        for (lang, text) in &l.notes {
            if text.trim().is_empty() {
                continue;
            }
            if lang.is_empty() {
                out.push_str("notes:\n");
            } else {
                out.push_str(&format!("notes.{lang}:\n"));
            }
            for line in text.trim_matches('\n').lines() {
                // A note line that looks like a section header would end the notes.
                let t = line.trim();
                if t.starts_with("==") || section_of(t).is_some() {
                    out.push_str("   ");
                }
                out.push_str(line.trim_end());
                out.push('\n');
            }
            out.push('\n');
        }
        write_items(&mut out, "ex1", &l.ex1);
        write_items(&mut out, "ex2", &l.ex2);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "course: es\nname: Test\nname.it: Prova\n\n== 2 A2\ntitle: Hola\ntitle.it: Ciao\n\ndialogue:\n- Hola, ¿qué tal?\n  it: Ciao, come va?\n  en: Hi\n  pron: ola\n\nnotes.it:\n1. Una nota\n   che continua: sì.\n\nex1:\n- Bien.\n  it: Bene.\nex2:\n- Muy [bien].\n  it: Molto bene.\n\n== 1 A1\ntitle: Uno\n";

    #[test]
    fn parses_sample() {
        let c = parse_course(SAMPLE, "t").unwrap();
        assert_eq!(c.lang, "es");
        assert_eq!(c.name_tr["it"], "Prova");
        assert_eq!(c.lessons.len(), 2);
        assert_eq!(c.lessons[0].number, 1, "sorted by number");
        let l = &c.lessons[1];
        assert_eq!(l.level, Level::A2);
        assert_eq!(l.dialogue[0].tr["it"], "Ciao, come va?");
        assert_eq!(l.dialogue[0].pron, "ola");
        assert_eq!(l.notes["it"], "1. Una nota\n   che continua: sì.");
        assert_eq!(l.ex2[0].blanks(), vec!["bien"]);
    }

    #[test]
    fn round_trip() {
        let c = parse_course(SAMPLE, "t").unwrap();
        let again = parse_course(&write_course(&c), "t").unwrap();
        assert_eq!(c.lessons, again.lessons);
    }

    #[test]
    fn errors_name_the_line() {
        let err = parse_course("course: es\n== x A1\n", "t").unwrap_err();
        assert!(err.starts_with("line 2"), "{err}");
    }

    /// Every built-in course parses, sits in its level's planned lesson
    /// numbers, and is complete for each base language (Italian, English,
    /// Spanish; the Spanish course itself needs no Spanish translation).
    #[test]
    fn builtin_courses_are_complete() {
        let (courses, problems) = crate::store::builtin_courses();
        assert!(problems.is_empty(), "{problems:?}");
        let langs: Vec<&str> = courses.iter().map(|c| c.lang.as_str()).collect();
        assert_eq!(langs, ["es", "fr", "de", "pt", "zh", "la", "vi"]);
        for c in &courses {
            let code = &c.lang;
            assert!(c.lessons.len() >= 13, "{code}: {} lessons", c.lessons.len());
            for l in &c.lessons {
                let at = format!("{code} lesson {}", l.number);
                let (first, last) = l.level.plan();
                assert!(
                    (first..=last).contains(&l.number),
                    "{at}: {} lessons are numbered {first}–{last}",
                    l.level.name()
                );
                assert!(!l.title.is_empty() && !l.dialogue.is_empty(), "{at}: empty");
                assert!(!l.ex1.is_empty() && !l.ex2.is_empty(), "{at}: exercises missing");
                let bases: &[&str] = if code == "es" { &["it", "en"] } else { &["it", "en", "es"] };
                for &base in bases {
                    assert!(l.notes.contains_key(base), "{at}: notes.{base} missing");
                    assert!(l.title_tr.contains_key(base), "{at}: title.{base} missing");
                    for it in l.dialogue.iter().chain(&l.ex1).chain(&l.ex2) {
                        assert!(it.tr.contains_key(base), "{at}: `{}` has no {base}", it.text);
                    }
                }
                for it in &l.ex2 {
                    assert!(!it.blanks().is_empty(), "{at}: ex2 `{}` has no [blank]", it.text);
                }
            }
        }
    }
}
