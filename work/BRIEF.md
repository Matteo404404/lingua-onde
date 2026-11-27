# Lingua Onde lesson brief

You are writing lessons for Lingua Onde, a desktop app that follows the method
of the Assimil courses. Repository: this one (paths below are relative to its root).

Everything you write is original. Never copy or closely paraphrase Assimil or
any other published course, textbook, song, film or copyrighted text. Do not
open the reference course books (kept outside the repository). Real places, history and public-domain authors
are fine.

## Read first, in this order

1. `METHOD.md`: the method, checked against the Assimil books. Section 1 is
   Foundation.
2. `content/FORMAT.md`: the file format and every key you may use.
3. `work/PATTERNS.md`: how Assimil builds a course, measured lesson by
   lesson in the books: sizes, the route of each grammar point, notes,
   exercises, culture, the author's asides, revision lessons. Your lessons
   must fit it.
4. The plan of your course (table below): one line per lesson. Write each
   lesson to its line.
5. The pronunciation key of your pair (table below).
6. Every lesson of the course before the ones you write, in
   `work/<lang>/foundation/NNN.txt` (and `content/<lang>/foundation-*.txt`).
   Your lessons continue them: you must know which words, forms and notes
   the learner has already met, to reuse them and to point back to them.
7. The approved example lessons of your pair, when there are some (table
   below): match their layout, length, tone and style. When this brief and
   the examples seem to disagree, follow the examples and say so.

| Pair | Lesson files | Plan | Key | Approved examples |
|---|---|---|---|---|
| English for Italian speakers | `work/en/foundation/NNN.txt` | `work/en/syllabus.md` | `content/en/pronunciation.md` | lessons 1–7 in `work/en/foundation/` (checked) |
| Spanish for English speakers | `work/es/en/foundation/NNN.txt` | `work/es/en/foundation.md` (writer: `work/es/en/writer.md`) | `content/es/pronunciation.md`, English readers | lessons 1–2 in `work/es/en/foundation/` |

Lessons are written in order, one week (seven lessons) at a time, never in
parallel: each lesson builds on exactly what came before it.

## The learner

The course is written for speakers of one language (the "base", `bases:` in
the course files). Write every translation, note, observation, title and
cultural note in that language only, for a native speaker of it, comparing
with their language where that helps. Write nothing in other base languages.

## How a Foundation lesson is built

Each lesson is a small scene complete in itself, not an episode of a story:

- Its own title, often witty or a play on words, and its own people: two
  voices, or a name that belongs to this lesson only. Everyday situations
  (a café, a shop, the phone, the station, home, work, the doctor, a
  holiday...), seen with a smile. Most scenes end with a twist or a joke in
  the last line. At most a scene may run over two lessons ("..., continued").
- Short lines, as people really speak. A narrator line or scene change
  ("A little later:") is rare: Lo Spagnolo has three in lessons 26–50. Use
  one only where the scene truly moves on, never to open a lesson; the
  setting belongs in the title or a note.
- Length grows slowly and unevenly, as the size table in `work/PATTERNS.md`
  says (lesson 1: five or six very short lines, about 20–30 words); the plan
  line gives each lesson's size.

- Every form is met before it is explained: a new structure appears in the
  dialogue, often several lessons before a note explains it fully; the
  revision lesson then puts it in order. Earlier words and forms come back
  all the time.

An ordinary lesson, with an illustration of the size of lesson 1 (original,
in English for Italians; you may use it as lesson 1 if the plan's line fits):

```
== 1 A1
numeral: Lesson One
title: A coffee
title.it: Un caffè

dialogue:
- Good morning! A coffee, please.
  it: Buongiorno! Un caffè, per favore.
- With milk?
  it: Con latte?
- Yes, please.
  it: Sì, grazie (sì, per favore).
- Here you are. That's three pounds, please.
  it: Ecco a lei (qui lei è). Sono (quello è) tre sterline, per favore.
- Three pounds? For a coffee?
  it: Tre sterline? Per un caffè?

sounds.it:
1. ...

notes.it:
1. Good morning: ...
2. Please: ...

ex1:
- Good morning!
  it: Buongiorno!
- A coffee with milk.
  it: Un caffè con latte.
- ...

ex2:
- [Good] [morning]! A coffee, [please].
  it: Buongiorno! Un caffè, per favore.
- ...

culture.it:
...
```

- `numeral:` the lesson number in words. Spanish: ordinals up to "Lección
  décima", then cardinals agreeing with "lección" ("Lección once",
  "Lección veintiuna"); English: "Lesson One", "Lesson Twenty-One".
- The translation is natural in the base language; where the target builds
  the sentence differently, add the word-for-word sense in brackets, only
  where it teaches something.
- Figured pronunciation (`pron.it:`): in lessons 1–28 the whole line, on
  every dialogue line and on the numeral; from lesson 29 only the words new
  in the lesson, the rest left out ("… *sè*tëdei …"); a new word is figured
  once, on the first line where it appears, and later lines leave it out
  (the tool does this: do not add it back by hand). A line with no new word
  has no pron line. The signs are those of
  content/en/pronunciation.md, taken from Assimil's Italian editions: the
  stressed syllable between asterisks (bold in the app), ‣ after a long vowel,
  ë the weak vowel, è the open e, θ, dh, sc/sci, ž, y. When a note quotes a
  sound, write it with the same signs (θènk yu‣, ë*baut*); a "do not say"
  example stays in Italian spelling ("non 'lòve'"). The plan
  (work/en/syllabus.md) still quotes some sounds in an older key
  (gǽraazh, jùustə): ignore those spellings and use the tool's. For English do not write it:
  run `python3 -I work/tools/figure_en.py work/en/foundation/NNN.txt --write`,
  which adds the lines by the key, then look at the words it lists under
  "check" (read, live, close...) and correct that line by hand where the
  sound is wrong for the meaning. For other languages write it by the key.
- `sounds.it:` pronunciation observations: one to three "remember" points on
  a sound of this lesson, compared with Italian. In nearly every lesson of
  the first weeks, then only when a sound is new or tricky.
- `notes.it:` four to eight notes, each on a word or phrase of the dialogue,
  in the order they appear: what it means and how it works, compared with
  Italian, often with a related everyday expression. Point back to where
  something was met ("l'abbiamo visto nella lezione 3") and ahead when a
  full explanation will come later ("lo vedremo nella lezione di ripasso").
  True, simple, at the lesson's level.
- `ex1:` exactly five items to translate into Italian. In the first lessons,
  words and short phrases of the dialogue; from about lesson 8, new
  sentences made of this lesson's words and earlier ones, nothing unmet.
  From lesson 50 the learner rebuilds them from the Italian, so the Italian
  must point to one natural target sentence.
- `ex2:` five sentences (four in lessons 1–2) with the missing words in
  `[brackets]`, one to three blanks of one or two words; the Italian is the
  prompt. Each blank has one correct answer given the Italian. The blanks
  test this lesson's notes, sometimes one older point.
- `culture.it:` a paragraph on customs, places, history or the language,
  tied to the scene, in about half the lessons: practical (how things work
  in the country), 5–12 lines, often closing with a short phrase in the
  target language. Only well-established facts.
- `aside.it:` when the plan says so (every four to six lessons): one to three
  lines from the author, warm and brief.
- Never start a line of free text with `==`, `dialogue:`, `ex1:`, `ex2:`,
  `notes`, `sounds`, `culture` or `tasks`.

## The revision lesson (7, 14, 21, ... 98)

```
== 7 A1
kind: revision
numeral: Lesson Seven
title: Revision
title.it: Ripasso

dialogue:
- <short line>
  it: ...
  refs: 1 3

notes.it:
<two or three lines of encouragement: no rules to learn by heart, doubts are normal>

1. <topic> (lezioni 1, 3, 5)
...
```

- `dialogue:` the revision dialogue: about ten short lines (eight to twelve)
  made only of words and forms from the lessons so far, a little scene with
  its own small twist. Every line has `refs:` with the lessons its words and
  forms come from (earlier lessons only).
- `notes.it:` the grammar recap: a few lines of encouragement, then six to
  eight numbered sections that put in order what the six lessons before
  brought, each naming its lessons, in more depth than the daily notes:
  full tables of forms (a markdown table is fine), the rule, examples taken
  from the lessons, and a few completions (all the numbers to twenty, the
  whole verb). In the first revision lessons, pronunciation and stress have
  their own sections. End with one line that closes the week.
- No sounds, exercises or culture.

## Perfectionnement

Read `work/PATTERNS.md`, section "Perfectionnement", and the course plan
(`work/en/perfectionnement.md` for English). The learner has finished
Foundation: use its whole vocabulary freely, and bring what the plan names.

```
course: en
stage: perfectionnement
bases: it

== 4 B2
numeral: Lesson Four
title: Cornwall
title.it: La Cornovaglia

dialogue:
- <first sentence of the text, or first line of a dialogue>
  it: <a free, natural translation; idioms first literally in brackets>
...

notes.it:
1. <headword>: ...

ex1:
...

ex2:
...
```

- Levels: B2 1–45, C1 46–70. The text goes in `dialogue:`, one item per
  numbered sentence (a line of a dialogue or a sentence of prose): 14–25
  items, 150–400 words in all. No figured pronunciation, no `sounds` unless a
  word's stress is irregular, no `culture` section (culture lives in the
  text and the notes).
- The text is complete in itself and of the kind the plan gives. Humour runs
  through it; most texts end with a twist or a wink, the last line often
  undoing the premise. Real places, people and history: only
  well-established facts; quote only titles or a few words of public-domain
  works.
- `notes.it:` 8–15 notes; in a "word" lesson 15–30, nearly all idioms, and the
  last note is the expression list: each idiom with the sentence and note
  numbers ("take a break (frase 3, nota 2)"). Each note: the headword, its
  meaning, its literal sense if it is an idiom, one to three short examples
  with translation, a family of related words or senses when useful,
  "ricordate" back references to earlier lessons or to Foundation.
- `ex1:` five sentences of 5–20 words to translate; `ex2:` five with blanks
  (one to three of one or two words). The corrections need not be the only
  good answers, but the blanks must be.
- Revision lessons (7, 14 ... 70): `kind: revision`, no `dialogue:`;
  `notes.it:` a few lines, then three to five numbered sections on the
  lessons' finer points, with tables and examples; `ex1:` and `ex2:` hold the
  practice (five items each). No exercises are missing elsewhere.
- `aside.it:` only where the plan says.
- Files: `work/<lang>/perfectionnement/NNN.txt`; the checks below apply with
  `perfectionnement` in place of `foundation`.

## Mastery

Read METHOD.md section 3 and the course plan (`work/en/mastery.md` for
English). The learner is C1 and becoming C2.

- Levels: C1 1–25, C2 26–100. The text goes in `dialogue:`, one item per
  numbered sentence: 25–40 items, 350–700 words, of the kind and with the
  ending the plan gives; original, in an authentic style ("in the manner of"
  an era or genre, never imitating a living author's text). No figured
  pronunciation, no `sounds`, no `culture` section.
- `notes.it:` 10–18: register and its shifts, what is meant but not said,
  irony, cultural and historical references, idioms and collocations, the
  precise word against its near-synonyms, rhetoric; contrasts with Italian.
- `ex1:` five demanding sentences or short passages to translate, where the
  nuance matters. `ex2:` five collocations, idioms or connectors in context.
- `tasks:` the plan's one or two tasks. Each item: the model answer as the
  item's text (in English, of the length the task asks), `task.it:` the
  instruction, `rubric.it:` four or five self-check points separated by `;`
  (meaning; register; structure; idiom; effect, adapted to the task).
- Revision lessons (7, 14 ... 98): `kind: revision`; `dialogue:` the
  revision text, 20–30 items, each with `refs:` to the week's lessons;
  `notes.it:` a few lines, then four to six sections that put the week's
  focus in order with examples from its texts; no exercises; one task.
- Files: `work/<lang>/mastery/NNN.txt`; the checks below apply with
  `mastery` in place of `foundation`.

## Checks, for every lesson file

- Each lesson in its own file `work/<lang>/foundation/NNN.txt` (three digits),
  starting with the course lines, a blank line, then the lesson:

  ```
  course: en
  stage: foundation
  bases: it

  == 1 A1
  ...
  ```

- `target/release/lingua_onde --check work/<lang>/foundation/NNN.txt` must
  print `ok` with no problem and no `gap:` line.
- `python3 -I work/tools/progression.py . <lang> foundation <first> <last>`
  lists, for each lesson, its lines and its new words, and any exercise word
  not met yet. Keep within the table above, and leave no unmet exercise word.
- Every "(lezione N)" in sounds, notes and culture names the lesson where
  that word first appears in a dialogue line, as `progression.py . <lang>
  foundation 1 <last>` lists it, not a lesson whose notes or culture only
  mention it. Words used as examples of a sound or a rule are words already
  met in a dialogue (not water, book, bed, much before they come); an
  example word that is new is the point of that note, or is marked as new.
- Reread each lesson once against this brief: counts, one answer per blank,
  Italian natural, the joke as the note explains it is the joke the dialogue
  tells, quotes and proverbs given exactly, nothing unmet.
- Touch nothing but your own lesson files. No git, no cargo.

## Report

Short: the lessons written with their line and new-word counts; the checker
output; the words figure_en.py flagged and what you did; any case the brief,
the key or the plan did not settle and how you decided; anything you think
is wrong in them (quote it, do not change it).
