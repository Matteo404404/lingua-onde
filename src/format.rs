//! Reads and writes the plain-text lesson format described in content/FORMAT.md.

use crate::model::{BASES, Course, Item, Lesson, Level, Stage};

/// The free-text sections of a lesson, one per base language.
#[derive(Clone, Copy, PartialEq)]
enum Text {
    Sounds,
    Notes,
    Culture,
}

const TEXTS: [(&str, Text); 3] = [("sounds", Text::Sounds), ("notes", Text::Notes), ("culture", Text::Culture)];

#[derive(Clone, PartialEq)]
enum Section {
    None,
    Dialogue,
    Text(Text, String),
    Ex1,
    Ex2,
    Tasks,
}

fn section_of(t: &str) -> Option<Section> {
    let key = t.strip_suffix(':')?;
    match key {
        "dialogue" => return Some(Section::Dialogue),
        "ex1" => return Some(Section::Ex1),
        "ex2" => return Some(Section::Ex2),
        "tasks" => return Some(Section::Tasks),
        _ => {}
    }
    TEXTS.iter().find_map(|(name, kind)| {
        let rest = key.strip_prefix(name)?;
        let lang = if rest.is_empty() { "" } else { rest.strip_prefix('.')? };
        Some(Section::Text(*kind, lang.to_string()))
    })
}

fn texts_mut(l: &mut Lesson, kind: Text) -> &mut std::collections::BTreeMap<String, String> {
    match kind {
        Text::Sounds => &mut l.sounds,
        Text::Notes => &mut l.notes,
        Text::Culture => &mut l.culture,
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

fn items_of<'a>(l: &'a mut Lesson, section: &Section) -> Option<&'a mut Vec<Item>> {
    match section {
        Section::Dialogue => Some(&mut l.dialogue),
        Section::Ex1 => Some(&mut l.ex1),
        Section::Ex2 => Some(&mut l.ex2),
        Section::Tasks => Some(&mut l.tasks),
        Section::None | Section::Text(..) => None,
    }
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
            for map in [&mut l.sounds, &mut l.notes, &mut l.culture] {
                for n in map.values_mut() {
                    *n = n.trim_matches('\n').to_string();
                }
            }
            course.lessons.push(l);
        }
    };

    for (i, raw) in src.lines().enumerate() {
        let n = i + 1;
        let line = raw.trim_end();
        let t = line.trim();

        if let Section::Text(kind, lang) = &section {
            let ends = t.starts_with("==") || section_of(t).is_some();
            if !ends {
                if let Some(l) = lesson.as_mut() {
                    let note = texts_mut(l, *kind).entry(lang.clone()).or_default();
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
            match items_of(l, &section) {
                Some(list) => list.push(item),
                None => {
                    return Err(format!("line {n}: sentences go under `dialogue:`, `ex1:`, `ex2:` or `tasks:`"));
                }
            }
            continue;
        }

        let Some((key, value)) = key_value(t) else {
            return Err(format!("line {n}: not understood: `{t}`"));
        };

        let indented = raw.starts_with(' ') || raw.starts_with('\t');
        let list = match lesson.as_mut() {
            Some(l) if indented => items_of(l, &section),
            _ => None,
        };
        if let Some(list) = list {
            let item = list.last_mut().ok_or(format!("line {n}: `{key}:` comes before any `- sentence`"))?;
            if key == "pron" {
                item.pron = value.to_string();
            } else if key == "refs" {
                let refs: Result<Vec<u32>, _> = value.split([' ', ',']).filter(|w| !w.is_empty()).map(str::parse).collect();
                item.refs = refs.map_err(|_| format!("line {n}: `refs:` takes lesson numbers"))?;
            } else if let Some(lang) = key.strip_prefix("pron.") {
                item.figured.insert(lang.to_string(), value.to_string());
            } else if let Some(lang) = key.strip_prefix("task.") {
                item.task.insert(lang.to_string(), value.to_string());
            } else if let Some(lang) = key.strip_prefix("rubric.") {
                item.rubric.insert(lang.to_string(), value.to_string());
            } else {
                item.tr.insert(key.to_string(), value.to_string());
            }
            continue;
        }

        match lesson.as_mut() {
            None => match key {
                "course" => course.lang = value.to_string(),
                "bases" => course.bases = value.split_whitespace().map(String::from).collect(),
                "stage" => course.stage = Some(Stage::parse(value).ok_or(format!("line {n}: unknown stage `{value}`"))?),
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
                "numeral" => l.numeral = value.to_string(),
                "kind" if value == "revision" => l.revision = true,
                "kind" => return Err(format!("line {n}: the only lesson kind is `revision`")),
                "level" => l.level = Level::parse(value).ok_or(format!("line {n}: unknown level `{value}`"))?,
                _ => {
                    if let Some(lang) = key.strip_prefix("title.") {
                        l.title_tr.insert(lang.to_string(), value.to_string());
                    } else if let Some(lang) = key.strip_prefix("numeral.pron.") {
                        l.numeral_pron.insert(lang.to_string(), value.to_string());
                    } else {
                        return Err(format!("line {n}: unknown lesson key `{key}`"));
                    }
                }
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

/// The lesson numbers of `level` in `stage`. Drafts without a stage use the
/// old single 300-lesson plan they were written on.
pub fn plan(stage: Option<Stage>, level: Level) -> Option<(u32, u32)> {
    match stage {
        Some(st) => st.plan(level),
        None => Some(match level {
            Level::A1 => (1, 25),
            Level::A2 => (26, 50),
            Level::B1 => (51, 75),
            Level::B2 => (76, 150),
            Level::C1 => (151, 200),
            Level::C2 => (201, 300),
        }),
    }
}

/// What is wrong in a built-in lesson: its number outside its level's range
/// in the course, an empty part, a missing translation, title or note in
/// one of the course's base languages (`Course::bases`), an
/// exercise 2 sentence without [blanks], an ordinary lesson without both
/// exercises, a revision lesson off the 7th place, or a missing pronunciation line
/// (every Chinese sentence carries pinyin, every Latin dialogue line its vowel lengths).
pub fn builtin_problems(c: &Course, l: &Lesson) -> Vec<String> {
    let mut out = Vec::new();
    let lang = c.lang.as_str();
    let at = format!("{lang} lesson {}", l.number);
    match plan(c.stage, l.level) {
        Some((first, last)) if !(first..=last).contains(&l.number) => {
            out.push(format!("{at}: {} lessons are numbered {first}–{last}", l.level.name()))
        }
        None => out.push(format!("{at}: this course has no {} lessons", l.level.name())),
        _ => {}
    }
    if l.title.is_empty() || l.dialogue.is_empty() {
        out.push(format!("{at}: title or dialogue is empty"));
    }
    if !l.revision && (l.ex1.is_empty() || l.ex2.is_empty()) {
        out.push(format!("{at}: ex1 or ex2 is empty"));
    }
    if l.revision && !l.number.is_multiple_of(7) {
        out.push(format!("{at}: revision lessons are every 7th (7, 14, 21...)"));
    }
    let bases = c.bases();
    for &base in &bases {
        if l.notes.get(base).is_none_or(|n| n.trim().is_empty()) {
            out.push(format!("{at}: notes.{base} missing"));
        }
        if l.title_tr.get(base).is_none_or(|t| t.is_empty()) {
            out.push(format!("{at}: title.{base} missing"));
        }
    }
    let keys = l
        .notes
        .keys()
        .chain(l.title_tr.keys())
        .chain(l.sounds.keys())
        .chain(l.culture.keys());
    for k in keys.filter(|k| !k.is_empty() && !known(lang, k)) {
        out.push(format!("{at}: unexpected language `{k}` in a title or notes"));
    }
    let parts = [("dialogue", &l.dialogue), ("ex1", &l.ex1), ("ex2", &l.ex2), ("tasks", &l.tasks)];
    for (part, items) in parts {
        for it in items {
            item_problems(&mut out, &at, lang, &bases, part, it);
            if it.refs.iter().any(|r| *r >= l.number) {
                out.push(format!("{at}: {part} `{}`: refs must name earlier lessons", it.text));
            }
        }
    }
    out
}

/// A line for a base language other than the course's own: allowed, so a
/// course can gain a base language lesson by lesson.
fn known(lang: &str, k: &str) -> bool {
    BASES.contains(&k) && k != lang
}

fn item_problems(out: &mut Vec<String>, at: &str, lang: &str, bases: &[&str], part: &str, it: &Item) {
    let s = &it.text;
    if s.is_empty() {
        out.push(format!("{at}: empty sentence in {part}"));
    }
    let open = part == "tasks";
    for &base in bases {
        // A task's model answer may be long: its translation is optional.
        if part != "tasks" && it.tr.get(base).is_none_or(|t| t.is_empty()) {
            out.push(format!("{at}: {part} `{s}` has no {base}"));
        }
        if open && it.task.get(base).is_none_or(|t| t.is_empty()) {
            out.push(format!("{at}: {part} `{s}` has no task.{base}"));
        }
        if part == "tasks" && it.rubric.get(base).is_none_or(|t| t.is_empty()) {
            out.push(format!("{at}: {part} `{s}` has no rubric.{base}"));
        }
    }
    let langs = it.tr.keys().chain(it.figured.keys()).chain(it.task.keys()).chain(it.rubric.keys());
    for k in langs.filter(|k| !known(lang, k)) {
        out.push(format!("{at}: {part} `{s}` has a line for an unexpected language `{k}`"));
    }
    let cloze = part == "ex2";
    if !cloze && s.contains(['[', ']']) {
        out.push(format!("{at}: {part} `{s}` has [brackets], which only exercise 2 uses"));
    }
    if cloze && (it.blanks().is_empty() || s.matches('[').count() != s.matches(']').count()) {
        out.push(format!("{at}: ex2 `{s}` needs balanced [blanks]"));
    }
    if !open && !it.task.is_empty() {
        out.push(format!("{at}: {part} `{s}`: task lines belong to tasks:"));
    }
    let needs_pron = (lang == "zh" && part != "tasks") || (lang == "la" && part == "dialogue");
    if needs_pron && it.pron.is_empty() {
        out.push(format!("{at}: {part} `{s}` has no pron:"));
    }
}

/// What a built-in lesson still lacks to be finished, though it works:
/// the figured pronunciation of A1–A2 dialogue lines, the structured
/// revision of every 7th lesson, the open tasks of a Mastery lesson.
/// Lines start with "gap:" so they can be told from errors.
pub fn builtin_gaps(c: &Course, l: &Lesson) -> Vec<String> {
    let mut out = Vec::new();
    let (stage, lang, bases) = (c.stage, c.lang.as_str(), c.bases());
    let at = format!("{lang} lesson {}", l.number);
    if matches!(l.level, Level::A1 | Level::A2) {
        let missing = l.dialogue.iter().filter(|it| bases.iter().any(|b| it.figured(b).is_none())).count();
        if missing > 0 {
            out.push(format!(
                "{at}: gap: {missing} dialogue lines lack pron.{} (figured pronunciation)",
                bases.join("/pron.")
            ));
        }
    }
    if l.revision && l.dialogue.iter().any(|it| it.refs.is_empty()) {
        out.push(format!(
            "{at}: gap: revision dialogue lines should name the lessons they come from (refs:)"
        ));
    }
    if l.number.is_multiple_of(7) && !l.revision {
        out.push(format!("{at}: gap: every 7th lesson should be a revision lesson (kind: revision)"));
    }
    if stage == Some(Stage::Mastery) && !l.revision && l.tasks.is_empty() {
        out.push(format!("{at}: gap: a Mastery lesson needs open tasks (tasks:)"));
    }
    out
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
        if !it.refs.is_empty() {
            let refs: Vec<String> = it.refs.iter().map(u32::to_string).collect();
            out.push_str(&format!("  refs: {}\n", refs.join(" ")));
        }
        for (key, map) in [("pron", &it.figured), ("task", &it.task), ("rubric", &it.rubric)] {
            for (lang, v) in map {
                if !v.trim().is_empty() {
                    out.push_str(&format!("  {key}.{lang}: {}\n", v.trim()));
                }
            }
        }
    }
    out.push('\n');
}

fn write_text(out: &mut String, name: &str, map: &std::collections::BTreeMap<String, String>) {
    for (lang, text) in map {
        if text.trim().is_empty() {
            continue;
        }
        if lang.is_empty() {
            out.push_str(&format!("{name}:\n"));
        } else {
            out.push_str(&format!("{name}.{lang}:\n"));
        }
        for line in text.trim_matches('\n').lines() {
            // A line that looks like a section header would end the text.
            let t = line.trim();
            if t.starts_with("==") || section_of(t).is_some() {
                out.push_str("   ");
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }
        out.push('\n');
    }
}

pub fn write_course(c: &Course) -> String {
    let mut out = String::new();
    out.push_str("# Lesson file — format explained in FORMAT.md\n");
    out.push_str(&format!("course: {}\n", c.lang));
    if let Some(st) = c.stage {
        out.push_str(&format!("stage: {}\n", st.code()));
    }
    if !c.bases.is_empty() {
        out.push_str(&format!("bases: {}\n", c.bases.join(" ")));
    }
    out.push_str(&format!("name: {}\n", c.name));
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
        if !l.numeral.trim().is_empty() {
            out.push_str(&format!("numeral: {}\n", l.numeral.trim()));
        }
        for (lang, p) in &l.numeral_pron {
            out.push_str(&format!("numeral.pron.{lang}: {}\n", p.trim()));
        }
        if !l.audio.trim().is_empty() {
            out.push_str(&format!("audio: {}\n", l.audio.trim()));
        }
        if l.revision {
            out.push_str("kind: revision\n");
        }
        out.push('\n');
        write_items(&mut out, "dialogue", &l.dialogue);
        write_text(&mut out, "sounds", &l.sounds);
        write_text(&mut out, "notes", &l.notes);
        write_items(&mut out, "ex1", &l.ex1);
        write_items(&mut out, "ex2", &l.ex2);
        write_text(&mut out, "culture", &l.culture);
        write_items(&mut out, "tasks", &l.tasks);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "course: es\nname: Test\nname.it: Prova\n\n== 2 A2\ntitle: Hola\ntitle.it: Ciao\n\ndialogue:\n- Hola, ¿qué tal?\n  it: Ciao, come va?\n  en: Hi\n  pron: ola\n  pron.it: òla\n\nnotes.it:\n1. Una nota\n   che continua: sì.\n\nex1:\n- Bien.\n  it: Bene.\nex2:\n- Muy [bien].\n  it: Molto bene.\n\n== 1 A1\ntitle: Uno\n";

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
        assert_eq!(l.dialogue[0].figured("it"), Some("òla"));
        assert_eq!(l.dialogue[0].figured("en"), None, "no fallback to another reader's spelling");
        assert_eq!(l.dialogue[0].tr.len(), 2, "pron.it is not a translation");
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

    /// Every built-in course parses and passes `builtin_problems`. What
    /// courses still lack (`builtin_gaps`) is listed by `--check`, not failed here.
    #[test]
    fn builtin_courses_are_valid() {
        let (courses, problems) = crate::store::builtin_courses();
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(courses.len(), 27, "three courses for each of nine languages");
        let ids: Vec<&str> = courses.iter().take(3).map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["core-en", "core-en-perfectionnement", "core-en-mastery"]);
        let problems: Vec<String> = courses
            .iter()
            .flat_map(|c| c.lessons.iter().flat_map(|l| builtin_problems(c, l)))
            .collect();
        assert!(
            problems.is_empty(),
            "{} problems, first: {:?}",
            problems.len(),
            &problems[..problems.len().min(20)]
        );
    }

    #[test]
    fn problems_and_gaps() {
        let mut c = parse_course(SAMPLE, "t").unwrap();
        let st = Some(Stage::Foundation);
        let with = |lang: &str, stage: Option<Stage>| Course {
            lang: lang.into(),
            stage,
            ..Default::default()
        };
        let es = with("es", st);
        let l = &mut c.lessons[1];
        let p = builtin_problems(&es, l);
        assert!(p.iter().any(|p| p.contains("A2 lessons are numbered 26–50")), "{p:?}");
        assert!(p.iter().any(|p| p.contains("notes.en missing")), "{p:?}");
        l.number = 26;
        l.notes.insert("en".into(), "1. A note".into());
        l.title_tr.insert("en".into(), "Hi".into());
        for it in l.dialogue.iter_mut().chain(&mut l.ex1).chain(&mut l.ex2) {
            it.tr.insert("en".into(), "x".into());
        }
        assert_eq!(builtin_problems(&es, l), Vec::<String>::new());
        let for_italians = Course {
            bases: vec!["it".into()],
            ..with("es", st)
        };
        l.notes.remove("en");
        assert_eq!(builtin_problems(&for_italians, l), Vec::<String>::new());
        assert!(builtin_problems(&es, l).iter().any(|p| p.contains("notes.en missing")));
        l.notes.insert("en".into(), "1. A note".into());
        assert!(
            builtin_problems(&with("zh", st), l)
                .iter()
                .any(|p| p.contains("ex1 `Bien.` has no pron:"))
        );
        assert!(
            builtin_problems(&with("es", Some(Stage::Mastery)), l)
                .iter()
                .any(|p| p.contains("has no A2 lessons"))
        );
        // Drafts without a stage follow the old 300-lesson plan.
        l.level = Level::B2;
        l.number = 140;
        assert!(builtin_problems(&with("es", None), l).is_empty());
        l.level = Level::A2;
        l.number = 26;

        let g = builtin_gaps(&es, l);
        assert_eq!(
            g,
            ["es lesson 26: gap: 1 dialogue lines lack pron.it/pron.en (figured pronunciation)"]
        );
        l.dialogue[0].figured.insert("en".into(), "OH-lah".into());
        assert!(builtin_gaps(&es, l).is_empty());
        l.number = 28;
        assert!(builtin_gaps(&es, l)[0].contains("every 7th lesson"));

        // A revision lesson: its 7th place, exercises optional.
        l.revision = true;
        l.number = 27;
        assert!(
            builtin_problems(&es, l)
                .iter()
                .any(|p| p.contains("revision lessons are every 7th"))
        );
        l.number = 35;
        l.ex1.clear();
        l.ex2.clear();
        assert_eq!(builtin_problems(&es, l), Vec::<String>::new());
        l.revision = false;
        assert!(builtin_problems(&es, l).iter().any(|p| p.contains("ex1 or ex2 is empty")));
    }

    #[test]
    fn new_sections_round_trip() {
        let src = "course: nl\nstage: mastery\nbases: it\nname: X\n\n== 7 C1\ntitle: T\nkind: revision\n\ndialogue:\n- Hoi.\n  it: Ciao.\n\nsounds.it:\n1. ui si legge quasi «ai».\n\nnotes.it:\n1. Ripasso.\n\nex1:\n- Ja.\n  it: Sì.\n\nex2:\n- Ik [fiets].\n  it: Vado in bici.\n\nculture.it:\nIn Olanda tutti vanno in bici.\n\ntasks:\n- Een model.\n  task.it: Riassumi.\n  rubric.it: Senso intatto; registro giusto\n".replace("title: T\n", "title: T\ntitle.it: T\nnumeral: Zevende les\nnumeral.pron.it: zéfende les\n").replace("  it: Ciao.\n", "  it: Ciao.\n  refs: 1 3\n");
        let c = parse_course(&src, "t").unwrap();
        assert_eq!(c.stage, Some(Stage::Mastery));
        assert_eq!(c.bases, ["it"]);
        let l = &c.lessons[0];
        assert!(l.revision);
        assert_eq!(l.numeral, "Zevende les");
        assert_eq!(l.numeral_pron["it"], "zéfende les");
        assert_eq!(l.dialogue[0].refs, [1, 3]);
        assert_eq!(l.sounds_in("it"), "1. ui si legge quasi «ai».");
        assert_eq!(l.culture_in("it"), "In Olanda tutti vanno in bici.");
        assert_eq!(l.tasks[0].task_in("it"), "Riassumi.");
        assert_eq!(l.tasks[0].rubric_in("it"), ["Senso intatto", "registro giusto"]);
        let again = parse_course(&write_course(&c), "t").unwrap();
        assert_eq!(c.lessons, again.lessons);
        assert_eq!((again.stage, again.bases.clone()), (Some(Stage::Mastery), vec!["it".to_string()]));
        assert!(builtin_problems(&c, l).is_empty(), "{:?}", builtin_problems(&c, l));
    }
}
