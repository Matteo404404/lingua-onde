# Dutch for Italian speakers, Foundation: what every week must follow

Read first, in this order:
1. work/BRIEF.md: the sections "The learner", "How a Foundation lesson is
   built", "The revision lesson" and "Checks". Where it speaks of English,
   read Dutch; where it names figure_en.py and content/en/pronunciation.md,
   use the tools and key below.
2. METHOD.md section 1, content/FORMAT.md, work/PATTERNS.md.
3. The plan: work/nl/it/foundation.md (your lessons' blocks).
4. The pronunciation key: content/nl/pronunciation.md. Learn it: sounds
   lines and notes quote sounds with exactly its signs.
5. Every lesson already written in work/nl/it/foundation/ (all of them in
   the first weeks; later at least the two weeks before yours): reuse their
   words, point back to their notes, match their voice and format.

Files: work/nl/it/foundation/NNN.txt, one lesson each, starting:

    course: nl
    stage: foundation
    bases: it

    == N A1
    numeral: Eerste les
    title: ...
    title.it: ...

## The lesson

- `numeral:` the Dutch ordinal + les: Eerste les, Tweede les, Derde les,
  Vierde les, Vijfde les, Zesde les, Zevende les, Achtste les, Negende les,
  Tiende les, Elfde les, Twaalfde les, Dertiende les ... Twintigste les,
  Eenentwintigste les, Tweeëntwintigste les ... Dertigste les ...
  Honderdste les. The tool adds `numeral.pron.it:`.
- `title:` a short Dutch title of the scene (the plan's); `title.it:` its
  Italian. Revision lessons: title "Herhaling", title.it "Ripasso".
- `dialogue:` one item per line of the plan's size, each with `it:`, a
  natural Italian translation; where Dutch builds the sentence differently,
  add the word-for-word sense in brackets, only where it teaches something:
  «Mi piace il caffè (bevo volentieri caffè).»
- Address: u to strangers, older people, in shops and offices; jij/je among
  young people, friends, family, colleagues of the same level. In Italian
  use lei / tu to match, one form per speaker through the lesson.
- `pron.it:` never write it by hand. Run
  `python3 -I work/tools/figure_nl.py work/nl/it/foundation/NNN.txt --write`
  after the dialogue is final: it adds the numeral line and, in lessons
  1–28, the whole sentence; from lesson 29 only the new words. Then read
  each pron line once: if a word's stress or sound is clearly wrong (a
  word you know is stressed elsewhere, e.g. it put the stress on the last
  syllable of koffie), correct that line by hand with the key's signs and
  list the word in your report.
- `sounds.it:` one to three points on a sound of this lesson compared with
  Italian, quoting figured sounds with the key's signs (χu‣t, hœis, *ma‣*kë),
  in nearly every lesson of weeks 1–4, then when a sound is new or tricky.
- `notes.it:` four to eight notes on words of the dialogue, in order, as
  BRIEF.md says. Explain word order (verb second, the verb at the end),
  de/het, separable verbs and the like simply and truly, compared with
  Italian. Point back ("l'abbiamo visto nella lezione 3") and ahead ("lo
  vedremo nella lezione di ripasso").
- `ex1:` five items, `ex2:` five sentences (four in lessons 1–2) with
  [blanks], `culture.it:` and `aside.it:` as BRIEF.md and the plan say.
- Revision lessons: BRIEF.md "The revision lesson", in Italian, with the
  plan's sections and full tables (verb tables, de/het, word order with
  examples, numbers...). The dialogue is a NEW little scene between two
  people about one situation, never a montage of the week's lines: no names
  or people from earlier lessons, one speaker per line, speakers alternate,
  the twist makes sense on its own. Lessons 7, 14, 21 and 28 get pron.it on
  their dialogue too (the tool does it).

## Rules

- Standard Netherlands Dutch as people say it today, correct spelling (the
  Groene Boekje: zo'n, café's, ideeën, cadeautje). Natural Dutch, nothing a
  Dutch speaker would find odd or Belgian unless a note names Belgium.
- Everything original. Never open the reference course books (they are kept
  outside the repository). Only well-established facts about the
  Netherlands.
- Italian: natural, tu to the learner ("ricorda", "nota"), never lei or voi
  to the learner, no English words in Italian text where Italian has one.
- Italian prose stays Italian: quote a Dutch word as an example, never use
  it as an Italian word ("ha comprato la bici", not "ha comprato la fiets").
- Each lesson uses only words and forms met before plus the plan's new step.
  Check with `python3 -I work/tools/progression.py . nl/it foundation 1 N`.
- Any count a note states is true; every "(lezione N)" points to the lesson
  where the word first appears in a dialogue line; ex2 Italian is a plain
  translation, never a riddle or a hint about the answer; no line
  contradicts another; one answer per blank (if je and jij both fit, the
  Italian must settle it, or the blank must avoid it).
- Names belong to one lesson; no continuing story, no recurring cast.
- Grammar truths to keep exact: the finite verb is second in a main clause
  and last in a subordinate one; after inverted je/jij the -t drops (werk
  je?, but werkt u?); de-words and het-words, and that the adjective takes
  -e except before a het-noun with een/geen/no article (een groot huis, het
  grote huis); niet goes late, geen replaces een/no article; the perfect
  takes zijn with change of place or state (ik ben gegaan, hij is
  gegroeid); the participle -t/-d by 't kofschip; separable prefixes go to
  the end (ik bel je op) and join the verb in the participle (opgebeld).
- Italian traps to name when they come: kamer (stanza, not camera), slim
  (furbo), raar (strano), winkel (negozio), alstublieft also as "ecco" and
  "prego" when offering, "Dank je wel" / "Graag gedaan", ik heb gelijk (ho
  ragione), the verb never after the subject when something else opens the
  sentence ("Morgen ga ik", not "Morgen ik ga").

- Revision dialogues: a scene that makes sense line by line when read
  aloud, with a clear twist at the end (007.txt and 014.txt are the
  models). Never a string of the week's sentences.
- Revision tables: every example true and from the lesson cited (a word
  used for a person is not an example of a rule about things).
- Italian: "i Paesi Bassi" for the country (Holland is two provinces);
  ex1 Italian is plain and natural; ex1 Dutch sentences natural (not "de
  kamer op de zee").
- Facts in the plan are checked: if you change one, say so in the report
  with the reason; never replace a true fact with a guess.

- Figured sounds quoted in notes follow the key exactly: ie, oe, uu are
  short (zit for ziet, χut, nü), long only before r (vi‣r, bü‣r); the
  relaxed short i is I (zIt, Ik); copy a word's figure from its pron line.
- Free text is never hard-wrapped: each note, each sounds point, the
  culture paragraph and the aside are one line each, as in 001.txt.
- One speaker per dialogue line; tenses consistent inside a story told in
  the past.

## Checks before you finish

- Run figure_nl.py --write on each lesson (see above).
- `target/release/marea --check work/nl/it/foundation/NNN.txt` prints
  ok with no problem and no `gap:` line, for each file.
- `python3 -I work/tools/progression.py . nl/it foundation 1 N`: no unmet
  exercise word, the new words within the plan's size.
- Reread each lesson: Dutch natural and correct, the twist works, the notes
  true, one answer per blank, every reference right.
- Touch nothing but your own lesson files. No git, no cargo.

Report briefly: per lesson the lines and new words, the checker output, the
pron words you corrected by hand, anything you were unsure of.
