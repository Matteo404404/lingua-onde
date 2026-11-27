# Spanish for English speakers, Perfectionnement: what every week must follow

Read first, in this order:
1. work/BRIEF.md: the sections "The learner", "Perfectionnement" and
   "Checks" (where it says Italian, read English: `notes.en:`, `en:` lines,
   `aside.en:`; where it says English as the target, read Spanish).
2. METHOD.md section 2, work/PATTERNS.md section "Perfectionnement",
   content/FORMAT.md.
3. The plan: work/es/en/perfectionnement.md (rules at the top, then your
   lessons).
4. The model of a Perfectionnement lesson, finished and approved in another
   pair: work/nl/it/perfectionnement/001.txt (a word lesson), 004.txt (a
   portrait), 005.txt (a story), 007.txt (a revision lesson). Their notes are
   in Italian and their text in Dutch: copy their layout, length and note
   style, never their language or content.
5. The approved lessons of this course: work/es/en/perfectionnement/ from
   001.txt to the last one written; read all lessons of the previous week.
   Once week 1 is approved, these are your first models.
6. Foundation is known to the learner: work/es/en/foundation/*.txt (100
   lessons, Spanish of Spain, notes in English) and its plan
   work/es/en/foundation.md, which names the lesson that explains each
   grammar point. Never teach again what Foundation taught: point back to it
   and go further. Its voice (short, plain, practical, set against English)
   is the voice of your notes.
7. The pronunciation key content/es/pronunciation.md, section "For English
   speakers (English readers)", for the rare note that quotes a sound.

Never open the reference course books (the Assimil PDFs kept outside the
repository). Everything you write is original.

Files: work/es/en/perfectionnement/NNN.txt, one lesson each, starting:

    course: es
    stage: perfectionnement
    bases: en

    == N B2      (C1 from lesson 46)
    numeral: Lección primera
    title: ...
    title.en: ...

## The lesson

- `numeral:` Lección + the ordinal up to ten, feminine (Lección primera,
  segunda ... décima), then the cardinal, made feminine where it ends in uno
  (Lección once ... veintiuna, treinta y una, cuarenta y una, cincuenta y
  una, sesenta y una ... Lección setenta), exactly as Foundation's lesson of
  the same number writes it: copy it from work/es/en/foundation/NNN.txt. No
  `numeral.pron.en:`.
- `title:` a short Spanish title (the plan's, or better); `title.en:` its
  English. Revision lessons: `kind: revision`, title "Repaso y matices",
  title.en "Revision and finer points".
- `dialogue:` the text, one item per numbered sentence or dialogue line:
  14–25 items (the plan's heading gives the size), 150–400 words, of the
  kind and with the ending the plan gives. Each item has `en:` with a free,
  natural British English translation; for an idiom, the English meaning
  first, then the literal sense in brackets: «she misses her village (she
  throws her village of less)». No [square brackets]: they belong to
  Foundation's first months.
- No `pron.en:` lines, no `sounds.en:`, no culture section: culture lives
  in the text and the notes.
- `notes.en:` 8–15 notes, in the order of the text. Word lessons 15–30,
  almost all idioms; their last note is the expression list, one line each:
  "echar de menos (sentence 4, note 5)", with the right sentence and note
  numbers. Each note: the headword (the words of the text, exactly as the
  text has them), the meaning, the literal sense of an idiom, one to three
  short examples with translation, a family of related words or senses when
  useful, a contrast with English when it teaches something, "remember"
  back references.
- `ex1:` five NEW sentences of 5–20 words to translate (never copies of text
  lines), each with `en:`. From lesson 50 the learner rebuilds them from the
  English, so the English must point to one natural Spanish sentence (say
  "(formal)" or "(to a friend)" when tú or usted would both fit).
- `ex2:` five sentences with one to three [blanks] of one or two words
  each, with `en:` the plain English sentence (never a hint or a riddle).
  Each blank has only one right answer given the English. Never blank a word
  the English cannot tell from a near-synonym: quizá / quizás / tal vez,
  aunque / a pesar de que, porque / ya que / puesto que, sin embargo / no
  obstante, por eso / por lo tanto, pero / mas, el cual / que / quien after
  a preposition, tú / usted unless the English says which, ser / estar where
  both are possible. Never blank an imperfect or pluperfect subjunctive: the
  -ra and -se forms (hubiera / hubiese) are both right; blank another word
  of the sentence.
- Revision lessons (7, 14 ... 70): `kind: revision`, no `dialogue:`;
  `notes.en:` a few lines of encouragement, then the plan's numbered
  sections with tables (`| ... |` rows as in
  work/nl/it/perfectionnement/007.txt) and examples taken from the week's
  texts; `ex1:` and `ex2:` hold the practice (five + five), plain natural
  sentences, not puzzles that cram two points together.
- `aside.en:` only where the plan says (lesson 49).
- Every note starts on its own line ("1. ..."). Free text is never
  hard-wrapped: each note, each line of the revision introduction and the
  aside are one line each; only table rows and the expression list take one
  line per row.

## Pronunciation: figure_es.py only through --say

Never run work/tools/figure_es.py on a lesson file of this course (with or
without --write). It is written for Foundation: on a file it figures every
line in full for lessons 1–28, adds `numeral.pron.en:` and pron lines to
every B2 lesson and none to C1 ones, and counts "new words" against the
wrong lessons. If a `pron.en:` or `numeral.pron.en:` line is in one of your
files, delete it.

A note quotes a figured form only for a word that is new and hard: a
foreign or regional name, the x of México and Oaxaca (said as the jota), a
stress that decides the meaning (término, termino, terminó), a sound of
another variety that the note is about. Get it with
`python3 -I work/tools/figure_es.py --say "word"` and copy its output
exactly, with the key's signs (the stressed syllable between asterisks, é,
ou, ss, H, c and ç, gn, tch, ï). A one-syllable word never carries
asterisks. If the output looks wrong for a name (it reads the spelling, not
the name's own language), leave the figure out and list the word in your
report. For the sound of another variety (voseo's sh, the Caribbean s),
describe it through English words, as the key does, never with an invented
sign.

## Rules

- Everything original. Real places, people and history: only
  well-established facts; when unsure, leave it out. Legends are told as
  legends ("según la leyenda", "se cuenta que"). Quote only titles or a few
  words of public-domain works (the plan gives the quotations allowed);
  never quote a line of García Márquez, Borges, Neruda or any author who
  died after 1945. Describe a real artwork only as it is.
- Facts in the plan were checked: keep them. If you change or drop one, say
  so in the report with the reason; never replace a true fact with a guess,
  and never add a date, number or name you are not sure of. Things that may
  change are written so they stay true ("desde hace más de un siglo",
  "tradicionalmente"); use the current name of a museum or institution
  (Museo Nacional del Prado, Museo Reina Sofía).
- Standard Spanish of Spain as Madrid speaks and writes it today, spelled by
  the Real Academia (guion, solo and este without accent; ¿ and ¡ at the
  start; « » for quoted speech in the Spanish, as Foundation does): vosotros,
  coche, ordenador, móvil, zumo, vale, the perfect for a time not over.
  Natural Spanish, nothing a Madrileño would find odd.
- Other varieties only in the portraits about them (Canarias 30, México 37,
  the Río de la Plata 43, the Caribbean 50, the United States 55) and in
  notes that name them: there the narrator still writes the standard of
  Spain, and each word or form of the variety is quoted, named ("in Mexico
  ...", "in Argentina and Uruguay ...") and given its Spain equivalent.
  Never present a Latin American or regional form as standard, never let a
  Spanish character use one.
- Catalan, Galician and Basque only in their portraits (11, 18, 24) and in
  notes that name them: each word quoted, named ("in Catalan ...") and given
  its Spanish; spelled correctly in its own language, or left out. Never
  call them dialects of Spanish. Official names in those languages stay as
  they are (A Coruña, Ourense, Girona, Lleida, Vitoria-Gasteiz).
- Address: usted to strangers, customers, older people, officials; tú among
  friends, family, colleagues, young people. One form per speaker through
  the lesson, unless the plan makes the shift the point (lesson 29); a note
  says which when the English cannot show it.
- English: British spelling and words (colour, realise, flat, mobile,
  holiday, queue, the underground, chemist's), natural, you to the learner,
  plain and warm. English prose stays English: quote a Spanish word as an
  example, never use it as an English word ("he bought a ticket", not "he
  bought a billete"). In English lines and notes use the English name of a
  place where there is one (Seville, Andalusia, Catalonia, the Basque
  Country, the Canary Islands, Mexico City); in Spanish lines the Spanish
  name (Londres, Edimburgo, Nueva York).
- In English text the earlier course is "Foundation"; this course is "this
  course".
- Back references: "(Foundation, lesson N)" or "(Foundation, lesson N:
  word)" where N is the Foundation lesson whose DIALOGUE first has that word
  or form. Find it with
  `python3 -I work/tools/first_seen.py work/es/en/foundation 'regex'`
  (a word only in notes or culture does not count). For a grammar point, N
  is the lesson whose note explains it: the Foundation plan names it; open
  the lesson and check the note says it.
- "(lesson N)" alone means this course; use it for earlier lessons of this
  course (check with first_seen.py on work/es/en/perfectionnement) and,
  sparingly, for later ones the plan names. A bracket names one course only:
  "(Foundation, lesson 72)" and "(lesson 3, note 8)" go in separate
  brackets. Every "sentence N", "note N" and "lesson N, note M" must point at
  the right sentence or note: open it and count. Never put a reference next
  to a word that note does not contain; if no earlier note says it, give no
  reference.
- Dialogues must hold together line by line: each line answers the one
  before it, nobody asks what they have just said or been told, nobody
  repeats a question already answered, and a speaker knows what their role
  knows. Read the dialogue aloud as two people before you write the notes.
  One speaker per line; speakers alternate unless a line says otherwise.
- A story keeps its own facts straight from line to line (tenses
  consistent, the same person does the same thing, places and times stay
  put, who pays whom, what is already true at the start). Times, amounts and
  clocks add up. The twist must follow from the whole text and pay off its
  premise, and the note must explain the language of the line, never the
  joke itself.
- A history or biography text keeps one tense system: the past (preterite
  and imperfect) throughout, or the historic present throughout, with the
  present only for what is still true today. Never switch halfway.
- No recurring characters: a name used in an earlier lesson is another
  person; never write "you already know him (lesson N)" about a character.
- Explain idioms in their real sense first (echar de menos = to miss), then
  the literal image. Use an idiom only in its real sense; if the scene does
  not fit the sense, change the scene. Fixed phrases in their real shape
  (llevar razón, echar una mano, tirar la casa por la ventana); reflexive
  idioms keep their pronoun (echarse atrás: se echó atrás; quedarse de
  piedra: me quedé de piedra). Only idioms you are sure exist in the Spanish
  of Spain; if you are not sure a phrase is said that way, write a plainer
  sentence. Mark familiar words as familiar; no swear words.
- Grammar claims must be exact; do not overstate a rule ("always", "never")
  when usage varies: say what is usual and what is also heard; if a rule has
  exceptions or you are not sure, describe only the example in the text.
  Keep the truths Foundation taught: the subject pronoun dropped unless it
  contrasts; ser for identity, origin, profession, time and where an event
  takes place, estar for place and state; the perfect for a time not over,
  the preterite for a time that is; the imperfect for scene and habit, the
  preterite for the event; gustar agrees with the thing liked; the double
  negative; a before a person who is the object; pronouns before a
  conjugated verb, joined to an infinitive, a gerund and an affirmative
  imperative; le / les + lo becomes se lo; si + present or past indicative,
  never si + future, conditional or present subjunctive (si tuviera, never
  si tendría); a lo mejor + indicative; antes de que always + subjunctive;
  cuando + subjunctive for the future; impersonal haber always singular
  (había muchas personas).
- The English speaker's traps to name when the text meets them: ser and
  estar; the subjunctive where English has an infinitive or nothing (quiero
  que vengas); por and para; aspect (supe, conocí, quise, pude); sino and
  pero; llevar and traer, ir and venir; the -ing form said with an
  infinitive (fumar está prohibido); the possessive with body parts (me
  duele la cabeza, se lavó las manos); the false friends of Foundation 96
  and the calques of lesson 55.
- Irregular forms you are not sure of: check them in Foundation or use
  another verb (conducir → conduje, condujo, never "conducí"; andar →
  anduve; satisfacer → satisfizo; caber → quepo; prever → previó).
- The Spanish lines never contain an English word, not even in quotes ("un
  trastero es un storage room" is wrong): explain in Spanish inside the
  text, and put the English only in the `en:` line and the notes. The
  exceptions are the anglicisms that are the topic of lessons 55 (and 56),
  quoted and named as such.
- English lines translate the Spanish line, nothing more (no added "a walk,
  a coffee").
- A note explains only what is in its own sentence: no stray words from
  elsewhere. It never claims something about another text without rereading
  it. Any count a note states (seven idioms, three verbs) is true: count it.
- Notes never mention the plan, the brief or "the gag of the plan", and make
  no remarks about the lesson itself or about writing it: the learner never
  sees the plan. Write "the punchline", "the last line".
- A note's examples must be idiomatic Spanish: if you are not sure a
  sentence sounds natural, use the lesson's own sentence instead.
- Exercise sentences are small scenes, logical on their own, made of words
  the learner has met (Foundation and this course so far, plus the lesson's
  own); they restate the lesson's facts exactly (the same person does the
  same thing; a claim about a group holds for every member).
- Every reference to "ayer", "la semana que viene" or another lesson fits
  the course order: lesson N is read after lesson N − 1. No time span the
  course cannot keep ("dentro de medio año" when the lesson is five weeks
  away).
- Verse (lesson 34): every line is a full sentence or a clean fragment, the
  romance really has eight syllables a line counted the Spanish way (with
  sinalefa, and the last-stress rule), and the even lines really share the
  assonance.
- Never leave working notes to yourself in a lesson file ("(Note: ...
  replace it.)", "check", "?", an alternative in brackets). Before you
  finish, search your files for "note:", "actually", "fix", "TODO" and for
  English inside Spanish lines, and remove them: a lesson file contains only
  the lesson.
- From lesson 50 the course is C1 and the active wave runs: do not describe
  how the app shows it beyond what lesson 49's aside says.

Rules added after each reviewed week go at the end of this section, under
"Rules added after week N"; the newest are the ones most often broken.

## Checks before you finish

- `target/release/lingua_onde --check work/es/en/perfectionnement/NNN.txt`
  prints ok with no problem for each file.
- `python3 -I work/tools/extra_checks.py work/es/en/perfectionnement/*.txt`
  reports no repeated title (the revision lessons' "Repaso y matices"
  excepted) and no dialogue line repeated in another lesson.
- No `pron.en:` or `numeral.pron.en:` line in any file.
- Every "(Foundation, lesson N)" verified with first_seen.py; every
  "(lesson N, note M)", "sentence N" and "note N" opened and counted.
- Reread every lesson once against this brief: the dialogue coherent line
  by line, the punchline works and the notes explain its language
  correctly, natural Spanish of Spain, natural British English, one answer
  per blank, no regional or other-language word unlabelled, no doubtful
  fact, every count true, notes not hard-wrapped.
- Touch nothing but your own lesson files. No git, no cargo. The commands
  you may run besides reading files and grep: the checker, extra_checks.py,
  first_seen.py and figure_es.py --say above.

Report briefly: per lesson items, words, notes; checker output; facts you
were unsure of and what you did with them; any word figure_es.py --say
figured wrongly; any choice the plan did not settle; anything in the plan
you think is wrong (quote it, do not change it).

## Rules added after week 1

- A scene of bickering (who pays) must still make sense line by line: a
  speaker who has just given in does not argue again two lines later, and a
  waiter does not ask a question that his next line makes pointless.
- The second wave is called la segunda vuelta, as in Foundation lesson 100.
- An anecdote's twist must be one the reader can see at once (the cross was
  showing the time, not the degrees), not a muddle of two readings.
- Narration keeps one tense frame: in a present-tense day, ha quedado, not
  quedó.
- No dash at the start of a dialogue line.
- When a revision table quotes a line, copy it from the final lesson file.

## Rules added after week 2

- The last line of a story states the twist cleanly (era su fiesta
  sorpresa, y el único que había llegado tarde era él), not a summary of
  what the reader already knows.
- A satire's last line must be literal enough to be understood: no image
  that means nothing ("la ha convertido en acueducto").
- An exercise never quotes a line that was cut or changed: check against
  the final dialogue.

## Rules added after week 3

- A first-person twist must not contradict what the narrator said earlier:
  a narrator who turns out to be the singer cannot have complained about
  hearing the singing; plant the clue instead (yo no lo oigo nunca, porque a
  esa hora ya estoy en la ducha).
- "otra obra de" only after the first work has been named.
- A comparison must compare two different things (la crema catalana,
  parecida a las natillas), never a thing with itself.

## Rules added after week 4

- Times in a story must agree with each other: if she wakes at half past
  nine and is late, the party was in the morning.
- Do not repeat a request already settled two lines before (the contract
  in writing); use the line to set up the twist.
- Satire may exaggerate habits, but not present an illegal habit as normal
  today (smoking indoors has been banned since 2011).

## Rules added after week 5

- Verse: write the lines in order, count each one aloud (sinalefa, the
  last-stress rule), and check that every even line has the same
  assonance; a capital letter only after a full stop.
- A word lesson may be a list, but each line must still make sense on its
  own (no "la de las maletas, que van siete, y así llevas las de perder").
- Dates you are not sure to the year are given by decade (en los años
  treinta).
