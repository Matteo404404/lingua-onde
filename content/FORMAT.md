# Lesson file format

One plain-text file (UTF-8) holds one course. The app reads the built-in
courses from this folder and your own courses from its data folder
(`~/.local/share/lingua-onde/courses/`). You can write lessons here by hand
or with the lesson editor inside the app; both produce this format. What each
part is for is explained in `METHOD.md`.

## Course lines

- `course: es` the target language. Codes: `it` Italian, `en` English, `es`
  Spanish, `fr` French, `de` German, `nl` Dutch, `pt` Portuguese, `zh`
  Chinese, `la` Latin, `vi` Vietnamese.
- `stage: foundation`, `perfectionnement` or `mastery` (built-in courses).
- `bases: it` the learners' languages the course is written for. Without it,
  a built-in course gives Italian, English and Spanish (the Spanish course
  Italian and English).
- `name:` and `name.it:`, `name.en:`... the course's name.

A built-in course is split by level: `es/foundation-a1.txt`,
`es/foundation-a2.txt`, ... `es/mastery-c2.txt`, each starting with the same
`course:` and `stage:` lines; the first file of a course carries the names.
The files are listed in `src/store.rs`. Built-in lessons are numbered inside
their course, each level in its own range:

| Course | Levels and lesson numbers |
|---|---|
| Foundation (100) | A1 1–25, A2 26–50, B1 51–75, B2 76–100 |
| Perfectionnement (70) | B2 1–45, C1 46–70 |
| Mastery (125) | C1 1–25, C2 26–125 |

## Lessons

Lines starting with `#` are comments; blank lines are ignored. Keys are
written `key: value`; a key may carry a language (`title.it`, `notes.en`).

- `== <number> <level>` starts a lesson; level is `A1 A2 B1 B2 C1 C2`.
- `title:` in the target language, `title.it:`... in the learner's.
- `numeral:` the lesson number spelled in the target language ("Lección
  primera"), and `numeral.pron.it:`... its figured pronunciation.
- `kind: revision` marks a revision lesson (every 7th built-in lesson).
- `audio:` (optional) names a recording, absolute or relative to the course's
  audio folder. Without it the app looks in that folder for a file whose name
  ends with the lesson number (`Leccion 12.mp3`, `L012.mp3`), and otherwise
  reads the text aloud with a downloaded voice.

Sections, in this order:

- `dialogue:` the dialogue or text (a revision lesson's revision dialogue).
- `sounds.it:` pronunciation observations, free text.
- `notes.it:` the notes, free text; a revision lesson's grammar recap.
- `ex1:` exercise 1, sentences to translate into the learner's language.
- `ex2:` exercise 2, each missing word in `[brackets]` inside the target
  sentence; the translation is the prompt.
- `culture.it:` a cultural note, free text.
- `tasks:` open tasks (Mastery).
- `aside.it:` a word from the author (encouragement, an announcement), free
  text.

Free-text sections run until the next section. Number the notes (`1. ...`)
and start each from the word or phrase of the dialogue it explains.

`dialogue:`, `ex1:`, `ex2:` and `tasks:` hold items. An item starts with `- `
and the sentence in the target language. Indented lines under it give:

- `it: ...`, `en: ...` the translation for each base language, with the
  word-for-word sense in parentheses where the languages differ:
  `it: Come va (che tale)?`.
- `pron: ...` pinyin (Chinese, every item) or vowel lengths (Latin, every
  dialogue line).
- `pron.it: ...` the figured pronunciation for an Italian reader (likewise
  `pron.en`, `pron.es`): the sentence spelled as that reader would read it
  aloud, following `content/<lang>/pronunciation.md`. Built-in Foundation
  courses give the whole line on every dialogue line of lessons 1–28, then
  only the line's new words ("… tii …"). The app shows the learner's one.
- `refs: 1 3` (revision dialogue) the earlier lessons the line's words come
  from; the app shows "(1, 3)".
- `task.it: ...` and `rubric.it: ...` (tasks only): the instruction, and the
  self-check criteria separated by `;`. The item's sentence is a model answer;
  its translation is optional.

## Checking

`lingua_onde --check <file>...` lists what is wrong in a built-in lesson (a
number outside its level's range, a missing translation, title or note in
one of the course's base languages, an exercise 2 sentence without
`[blanks]`, a missing `pron:` where due) and, on lines with "gap:", what it
still lacks to be finished: the figured pronunciation of lessons 1–28, the
revision lesson of every 7th place, the lesson references of a revision
dialogue, the open tasks of a Mastery lesson.

## Example

```
course: es
stage: foundation
bases: it
name: Spanish
name.it: Spagnolo

== 1 A1
title: ¡Hola!
title.it: Ciao!
numeral: Lección primera
numeral.pron.it: lecthióm priméra

dialogue:
- Hola, Marta. ¿Qué tal?
  it: Ciao, Marta. Come va (che tale)?
  pron.it: óla, màrta. che tal?

sounds.it:
1. La h non si pronuncia mai: hola si legge óla.

notes.it:
1. ¿Qué tal? è il modo più comune per chiedere "come va?". In spagnolo le
   domande si aprono con ¿ e si chiudono con ?.

ex1:
- ¿Qué tal, Pedro?
  it: Come va, Pedro?

ex2:
- Muy [bien], [gracias].
  it: Molto bene, grazie.

culture.it:
In Spagna ci si saluta spesso con due baci sulle guance, anche appena presentati.
```
