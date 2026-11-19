# Lesson file format

One plain-text file (UTF-8) holds one course. The app reads the built-in
courses from this folder and your own courses from its data folder
(`~/.local/share/lingua-onde/courses/`). You can write lessons here by hand
or with the lesson editor inside the app; both produce this format.

Rules:

- Lines starting with `#` are comments. Blank lines are ignored.
- Keys are written `key: value`. A key may carry a language: `title.it`,
  `notes.en`. Language codes: `it` Italian, `en` English, `es` Spanish,
  `fr` French, `de` German, `pt` Portuguese, `zh` Chinese, `la` Latin,
  `vi` Vietnamese. Built-in lessons give translations and notes in `it`, `en`
  and `es` (the Spanish course in `it` and `en`).
- A lesson starts with `== <number> <level>`; level is one of
  `A1 A2 B1 B2 C1 C2`. Built-in courses follow the shape of a full course
  (100 lessons up to B2, 100 more from B2 to C1, then C2), so each level owns
  a range of numbers: A1 1–25, A2 26–50, B1 51–75, B2 76–150, C1 151–200,
  C2 201–300. New lessons take a free number in their level's range.
- A built-in course may be split over several files (`es.txt`,
  `es-part2.txt`...): each starts with the same `course:` line and they are
  listed in `src/store.rs`.
- `dialogue:`, `ex1:` and `ex2:` hold items. An item starts with `- ` followed
  by the sentence in the target language. The indented lines under it give the
  translation for each base language (`  it: ...`, `  en: ...`) and an optional
  pronunciation hint (`  pron: ...`, pinyin for Chinese, vowel lengths for Latin).
  Put the word-for-word literal sense in parentheses inside the translation,
  e.g. `it: Come va (che tale)?`.
- `notes.it:` / `notes.en:` start free text that runs until the next section;
  number each note (`1. ...`). Use the dialogue word or phrase first, then the
  explanation.
- `ex1:` = "translate": target sentences the learner translates in their head.
- `ex2:` = "fill in the missing words": put each missing word in `[brackets]`
  inside the target sentence; the translation is shown as the prompt.
- `audio:` (optional) names a recording for the lesson, absolute or relative to
  the course's audio folder. Without it the app looks in the course's audio
  folder for a file whose name ends with the lesson number (`Leccion 12.mp3`,
  `L012.mp3`) and otherwise reads the text aloud with a downloaded voice.

Example:

```
course: es
name: Spanish starter course
name.it: Corso base di spagnolo

== 1 A1
title: ¡Hola!
title.it: Ciao!
title.en: Hello!

dialogue:
- Hola, Marta. ¿Qué tal?
  it: Ciao, Marta. Come va (che tale)?
  en: Hi, Marta. How are you (what such)?
- Muy bien, gracias. ¿Y tú?
  it: Molto bene, grazie. E tu?
  en: Very well, thanks. And you?

notes.it:
1. ¿Qué tal? è il modo più comune per chiedere "come va?". In spagnolo le
   domande si aprono con ¿ e si chiudono con ?.
notes.en:
1. ¿Qué tal? is the everyday "how are you?". Spanish questions open with ¿
   and close with ?.

ex1:
- ¿Qué tal, Pedro?
  it: Come va, Pedro?
  en: How are you, Pedro?

ex2:
- Muy [bien], [gracias].
  it: Molto bene, grazie.
  en: Very well, thanks.
```
