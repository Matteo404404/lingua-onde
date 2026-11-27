# Spanish for English speakers, Foundation: what every week must follow

Read first, in this order:
1. work/BRIEF.md: the sections "The learner", "How a Foundation lesson is
   built", "The revision lesson" and "Checks". Where it speaks of Italian,
   read English (notes.en, en: lines); where it names figure_en.py and
   content/en/pronunciation.md, use the tool and key below.
2. METHOD.md section 1, content/FORMAT.md, work/PATTERNS.md.
3. The plan: work/es/en/foundation.md, its opening pages and your lessons'
   blocks.
4. The pronunciation key: content/es/pronunciation.md, the section "For
   English speakers (English readers)". Learn it: sounds lines and notes quote
   sounds with exactly its signs.
5. Every lesson already written in work/es/en/foundation/ (all of them in the
   first weeks; later at least the two weeks before yours): reuse their
   words, point back to their notes, match their voice and format. 001.txt and
   002.txt are the models.

Never open the reference course books (the Assimil PDFs kept outside the
repository). Everything you write is original.

Files: work/es/en/foundation/NNN.txt, one lesson each, starting:

    course: es
    stage: foundation
    bases: en

    == N A1
    numeral: Lección primera
    title: ...
    title.en: ...

## The lesson

- `numeral:` Lección + the ordinal up to ten, feminine (Lección primera,
  segunda, tercera, cuarta, quinta, sexta, séptima, octava, novena, décima),
  then the cardinal number, made feminine where it ends in uno (Lección once,
  doce, trece ... veinte, veintiuna, veintidós ... treinta, treinta y una,
  treinta y dos ... noventa y nueve, Lección cien). The tool adds
  `numeral.pron.en:`.
- `title:` a short Spanish title of the scene (the plan's); `title.en:` its
  English. Revision lessons: title "Revisión y notas", title.en "Revision and
  notes".
- `dialogue:` one item per line of the plan's size. Each `en:` line is
  natural British English first; where Spanish builds the sentence
  differently, add the literal sense in (parentheses), and put [square
  brackets] around English words the Spanish does not have, as Spanish with
  Ease does: «Are [you] English?», «Here you are. (Here [you] have.)», «What
  can I get you? (What [for] you do I put?)». Only where it teaches something,
  and only in dialogue lines: exercises never have brackets in their English.
- Address: tú among friends, family, young people, colleagues; usted to
  strangers who are older, in formal settings, from staff to customers when
  the plan says so; vosotros for "you all" (Spain). One form per speaker
  through the lesson, and the English note says which.
- `pron.en:` never write it by hand. Run
  `python3 -I work/tools/figure_es.py work/es/en/foundation/NNN.txt --write`
  after the dialogue is final, and again after every change to it: it
  rewrites all pron lines (and the numeral line), the whole sentence in
  lessons 1–28, only the new words from lesson 29. It stops on a digit: write
  numbers in words in the dialogue. If a word comes out wrong (a foreign name,
  an x said as j in México), do not edit the line: list the word in your
  report, and the reviewer adds it to the tool's FIX table.
- `sounds.en:` one to three points on a sound of this lesson compared with
  English, quoting figured sounds copied from the pron lines (*boué*no,
  Ha*biér*, *ya*mo), in nearly every lesson of weeks 1–4, then when a sound is
  new or tricky. Describe sounds through English words, as the key does (the
  ch of Scottish loch, the ni of onion, the tt of American better).
- `notes.en:` four to eight notes on words of the dialogue, in order, as
  BRIEF.md says, in the voice of 001–002: short, plain, practical, set against
  English ("Spanish does not need the 'I'"; "literally 'less badly': the
  nearest English is 'just as well'"). Point back ("as in lesson 3") and ahead
  ("we will see the whole table in the revision lesson").
- `ex1:` five items, `ex2:` five sentences (four in lessons 1–2) with
  [blanks], `culture.en:` and `aside.en:` as BRIEF.md and the plan say.
- Revision lessons: BRIEF.md "The revision lesson", in English, with the
  plan's sections and full tables (verb tables, pronouns, ser and estar with
  examples, numbers). The notes open with a few lines of reassurance (no rule
  need be learnt by heart; doubts are normal). The dialogue is a NEW little
  scene between two people about one situation, never a montage of the week's
  lines: no names or people from earlier lessons, one speaker per line,
  speakers alternate, the twist makes sense on its own. Lessons 7, 14, 21 and
  28 get pron.en on their dialogue too (the tool does it).

## Rules

- Spanish of Spain as Madrid speaks it today, correct spelling and accents
  (¿ and ¡ at the start, á é í ó ú, ü, ñ), « » for quoted speech: vosotros,
  coche, ordenador, móvil, zumo, vale, the perfect for what happened today.
  Nothing a Madrileño would find odd or Latin American, unless a note names
  Latin America.
- Everything original. Only well-established facts about Spain; quotes and
  proverbs given exactly; no invented idioms or sayings presented as real.
- English: British spelling and words (colour, flat, mobile, holiday,
  queue, the underground, chemist's), you to the learner, plain and warm.
  English prose stays English: quote a Spanish word as an example, never use
  it as an English word ("he bought a ticket", not "he bought a billete").
- Each lesson uses only words and forms met before plus the plan's new step.
  Check with `python3 -I work/tools/progression.py . es/en foundation 1 N`.
- Any count a note states is true; every "(lesson N)" points to the lesson
  where the word first appears in a dialogue line
  (`python3 -I work/tools/first_seen.py work/es/en/foundation 'word'`); the ex2
  English is a plain translation, never a riddle or a hint; no line
  contradicts another; one answer per blank (if tú and usted, or ser and
  estar, both fit, the English must settle it, or the blank must avoid it).
- Names belong to one lesson; no continuing story, no recurring cast.
- The dialogue holds together: each line answers the one before, one speaker
  per line, speakers alternate, nobody says something the scene has already
  made impossible. The punchline is the last line, and the joke the notes
  explain is the joke the dialogue tells.
- No working notes anywhere: no "No:", "?", "(check)" or alternatives left in
  a note, a sounds point or a translation. Every line is finished text.

- Grammar truths to keep exact: the subject pronoun is dropped unless it
  contrasts or stresses; ser for identity, origin, profession (with no
  article: es médica), character, time and dates; estar for place, state and
  mood, and with the gerund; hay (one form) for what exists, está for where a
  known thing is; in Spain the perfect for a time not over (hoy, esta semana,
  alguna vez), the preterite for a time that is (ayer, el año pasado, hace
  dos años); the imperfect for scene and habit, the preterite for the event;
  gustar agrees with the thing liked (me gustan los churros); the double
  negative (no ... nada); a before a person who is the object (veo a Marta);
  pronouns before a conjugated verb, joined to an infinitive, a gerund and an
  affirmative imperative (with an accent to keep the stress: levántate); le
  or les + lo becomes se lo; si + present, never si + future; the
  subjunctive only where the plan brings it.
- The English speaker's traps to name when they come: ser and estar; "it"
  has a gender (la lista → la olvidas); tener for age, hunger, cold (tengo
  treinta años); pedir (ask for) and preguntar (ask a question); por and
  para; gustar turned round; the false friends (largo, simpático, regular,
  constipado, embarazada, éxito, librería, carpeta, actual, sensible,
  asistir, realizar, molestar, discutir, recordar, decepción); the tapped r
  and the rolled rr; full vowels, never "uh"; an s never said z.

- Revision dialogues: a scene that makes sense line by line when read
  aloud, with a clear twist at the end. Never a string of the week's
  sentences.
- Revision tables: every example true and from the lesson cited.
- Facts in the plan are checked: if you change one, say so in the report
  with the reason; never replace a true fact with a guess.
- Free text is never hard-wrapped: each note, each sounds point, the
  culture paragraph and the aside are one line each, as in 001.txt.
- Tenses consistent inside a story told in the past.

## Checks before you finish

- Run figure_es.py --write on each lesson (see above).
- `target/release/lingua_onde --check work/es/en/foundation/NNN.txt` prints
  ok with no problem and no `gap:` line, for each file.
- `python3 -I work/tools/progression.py . es/en foundation 1 N`: no unmet
  exercise word, the new words within the plan's size.
- Reread each lesson: Spanish natural and correct, the twist works, the notes
  true, one answer per blank, every reference right.
- Touch nothing but your own lesson files. No git, no cargo.

Report briefly: per lesson the lines and new words, the checker output, any
word the tool figured wrongly, anything you were unsure of.
