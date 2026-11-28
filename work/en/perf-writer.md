# Perfectionnement writer: what every week must follow

Read first, in this order:
1. work/BRIEF.md in full (sections "Perfectionnement" and "Checks" apply; the
   Foundation sections give the learner's background and the house style).
2. METHOD.md section 2, work/PATTERNS.md section "Perfectionnement",
   content/FORMAT.md.
3. The plan: work/en/perfectionnement.md (rules at the top, then your lessons).
4. The approved lessons of this course: work/en/perfectionnement/001.txt to the
   last one written. Match their layout, length, tone and note style. Read at
   least 001 (a word lesson), 004 (a portrait), 005 (a story) and 007 (a
   revision lesson), and all lessons of the previous week.
5. Foundation is known to the learner: work/en/foundation/*.txt.

Files: work/en/perfectionnement/NNN.txt, one lesson each, starting:

    course: en
    stage: perfectionnement
    bases: it

    == N B2      (C1 from lesson 46)
    numeral: Lesson Eight
    title: ...
    title.it: ...

Rules:
- Everything original. Do not open the reference course books (kept outside the repository). Real places, people,
  history: only well-established facts; when unsure, leave it out.
- Text in `dialogue:`, one item per numbered sentence or dialogue line:
  14–25 items, 150–400 words, of the kind and with the ending the plan gives.
  Each item has `it:` with a free, natural Italian translation; idioms first
  literally in brackets.
- No pron lines, no culture section. Stress or a sound is noted inside a
  note only where it is irregular, with the signs of
  content/en/pronunciation.md exactly: stressed syllable between asterisks
  splitting syllables as the key does (no‣*θam*bëlënd), θ for th, sc / sci /
  sc' for sh (*ko‣*nisc'), e for the e of bed (*e*dinbrë), è only for the
  vowel of cat.
- notes.it: 8–15 notes (word lessons 15–30, almost all idioms, the last note
  the expression list "take a break (frase 3, nota 3)", with the right
  sentence and note numbers). Each note starts from the words of the text.
- In Italian text the earlier course is "Fondamenti" (the app's Italian name),
  never "Foundation". In the English text it is "Foundation".
- Back references: "(Fondamenti, lezione N)" or "(Fondamenti, lezione N:
  word)" where N is the Foundation lesson whose DIALOGUE first has that word
  or form (grep `^- ` lines of work/en/foundation/*.txt; a word only in notes
  does not count). For a grammar point, N is the lesson that explains it.
  "(lezione N)" alone means this course; use it for earlier lessons of this
  course and, sparingly, for later ones the plan names.
- Inside one bracket never mix the two courses: write "(Fondamenti, lezione
  51; in questo corso, lezione 3, nota 8)". Every "frase N", "nota N" and
  "lezione N, nota M" must point at the right sentence or note: count them.
  When a note says who does what in the text (whose foot, whose gaffe),
  check it against the text.
- Exercise sentences are small scenes: each must be logical on its own
  ("go Dutch: it is fairer that way", not "cheaper than a taxi"). The
  Italian has no stray English ("è un falso amico", not "False friend").
- Every note starts on its own line ("1. ..."), never glued to the end of the one before.
- Stress in verse or in a sentence is marked in bold with asterisks (the *sky* a*bove*), never with capitals.
- In the Italian, a speaker keeps one form of address (tu, Lei or voi) through the whole lesson.
- A one-syllable word never carries stress asterisks (kla‣k, not *kla‣k*).
- Explain idioms in their real sense first (deceptively spacious = more
  spacious than it looks, which estate agents use for small rooms).
- Grammar claims must be exact; do not overstate a rule ("always", "never",
  "for all persons") when usage varies; say what is usual and what is also
  heard.
- Every English sentence must make sense in context; read each line as a
  native reader would. Linkers must be logical: no "so" where nothing follows
  from what came before.
- Every in-course reference "(lezione N, nota M)" must point to a note that
  really treats that word or rule: open the note and check. Never put a
  reference next to a word that note does not contain.
- A note explains only what is in its own sentence: no stray words from
  elsewhere (no "Gravy", "Population", "Short for" when the text lacks them).
- Ordinary need (needs, need to) is not the modal needn't; do not mix them.
- Real-world details must be right: a British account number has eight
  digits and the sort code (xx-xx-xx) is separate; check such facts.
- A note never claims something about another text (a comma, a word, a
  choice) without rereading that text.
- When a note says "in speech one would say ...", the English given must be
  grammatical.
- Figured sounds only with the key's signs (content/en/pronunciation.md): the
  sound of j and soft g is gi / g' (job → giob, page → peig', budget →
  *ba*git), never dj or dg; there is no ò; the y sound is y (minute adj. →
  mai*nyu‣*t). Check every transcription against the key's table.
- Exercises restate the lesson's facts exactly: the same character does the
  same thing, and a claim about a group must hold for every member (not
  "the sisters, none of whom lived to thirty-two" when Charlotte did).
- Keep the British word the text uses: queue, not line, in a British scene.
- A note's header repeats its sentence exactly as the dialogue has it.
- Italian forms of address follow real use: workers and strangers address a
  customer or an adult they do not know with lei, not tu.
- A reference in brackets names one course only: "(Fondamenti, lezione N:
  word)" and "(lezione N, nota M)" go in separate brackets.
- Italian polite forms: Lei, le, la in lowercase throughout (lei, assicurarle),
  as in the rest of the course.
- From lesson 50 the course is C1 and the active wave runs: do not describe
  how the app shows it beyond what lesson 49's aside already says.
- Revision ex1 sentences must be plain, natural statements, not puzzles that
  cram two lesson points together.
- ex1: five sentences of 5–20 words to translate, each with `it:`. ex2: five
  sentences with one to three [blanks] of one or two words each, with `it:`;
  each blank has only one right answer given the Italian.
- Revision lessons (every 7th): `kind: revision`, no `dialogue:`; notes.it
  with a few introductory lines, then the plan's numbered sections with
  tables (`| ... |` rows as in 007.txt) and examples from the week's texts;
  ex1 and ex2 hold the practice (five + five).
- `aside.it:` only where the plan says.

Checks before you finish:
- `target/release/marea --check work/en/perfectionnement/NNN.txt` prints
  ok with no problem for each file.
- Reread every lesson: the joke works and the notes explain it correctly,
  natural British English, natural Italian, one answer per blank, no doubtful
  fact, every Fondamenti reference verified by grep.
- Touch nothing but your own lesson files. No git, no cargo.

Report briefly: per lesson items, words, notes; checker output; facts you
were unsure of; any choice the plan did not settle.
