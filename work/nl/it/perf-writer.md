# Dutch for Italian speakers, Perfectionnement: what every week must follow

Read first, in this order:
1. work/BRIEF.md: the sections "The learner", "Perfectionnement" and
   "Checks" (where it says English, read Dutch).
2. METHOD.md section 2, work/PATTERNS.md section "Perfectionnement",
   content/FORMAT.md.
3. The plan: work/nl/it/perfectionnement.md (rules at the top, then your
   lessons).
4. The model of a Perfectionnement lesson, with Italian notes in the same
   format: work/en/perfectionnement/001.txt (a word lesson), 004.txt (a
   portrait), 005.txt (a story), 007.txt (a revision lesson). Copy their
   layout, length and note style, not their content.
5. The approved lessons of this course: work/nl/it/perfectionnement/ from
   001.txt to the last one written; read all lessons of the previous week.
6. Foundation is known to the learner: work/nl/it/foundation/*.txt (100
   lessons, Netherlands Dutch, notes in Italian) and its plan
   work/nl/it/foundation.md, which names the lesson that explains each
   grammar point. Never teach again what Foundation taught: point back to
   it and go further.
7. The pronunciation key content/nl/pronunciation.md, for the rare note
   that quotes a sound.

Files: work/nl/it/perfectionnement/NNN.txt, one lesson each, starting:

    course: nl
    stage: perfectionnement
    bases: it

    == N B2      (C1 from lesson 46)
    numeral: Eerste les
    title: ...
    title.it: ...

## The lesson

- `numeral:` the Dutch ordinal + les, exactly as Foundation's lesson of the
  same number writes it (Eerste les, Tweeëntwintigste les, Drieëntwintigste
  les ... Zeventigste les): copy it from work/nl/it/foundation/NNN.txt. No
  `numeral.pron.it:`.
- `title:` a short Dutch title (the plan's, or better); `title.it:` its
  Italian. Revision lessons: `kind: revision`, title "Herhaling", title.it
  "Ripasso".
- `dialogue:` the text, one item per numbered sentence or dialogue line:
  14–25 items, 150–400 words, of the kind and with the ending the plan
  gives. Each item has `it:` with a free, natural Italian translation; for
  an idiom, the Italian meaning first, then the literal sense in brackets:
  «è al verde (siede su seme nero)».
- No `pron.it:` lines, no `sounds.it:`, no culture section: culture lives in
  the text and the notes.
- `notes.it:` 8–15 notes, in the order of the text. Word lessons 15–30,
  almost all idioms; their last note is the expression list, one line
  each: "op zwart zaad zitten (frase 4, nota 5)", with the right sentence
  and note numbers. Each note: the headword (the words of the text,
  exactly as the text has them), the meaning, the literal sense of an
  idiom, one to three short examples with translation, a family of related
  words or senses when useful, a contrast with Italian when it teaches
  something, "ricorda" back references.
- `ex1:` five NEW sentences of 5–20 words to translate (never copies of text
  lines), each with `it:`. From lesson 50 the learner rebuilds them from
  the Italian, so the Italian must point to one natural Dutch sentence.
- `ex2:` five sentences with one to three [blanks] of one or two words
  each, with `it:` the plain Italian sentence (never a hint or a riddle).
  Each blank has only one right answer given the Italian. Never blank a word
  the Italian cannot tell from a near-synonym: je / jij, hoewel / ofschoon,
  omdat / doordat, maar / echter, dus / daarom, toch / desondanks,
  waarschijnlijk / vermoedelijk, als / wanneer.
- Revision lessons (7, 14 ... 70): `kind: revision`, no `dialogue:`;
  `notes.it:` a few lines of encouragement, then the plan's numbered
  sections with tables (`| ... |` rows as in work/en/perfectionnement/007.txt)
  and examples taken from the week's texts; `ex1:` and `ex2:` hold the
  practice (five + five), plain natural sentences, not puzzles that cram
  two points together.
- `aside.it:` only where the plan says (lesson 49).
- Every note starts on its own line ("1. ..."). Free text is never
  hard-wrapped: each note, each line of the revision introduction and the
  aside are one line each; only table rows and the expression list take
  one line per row.

## Rules

- Everything original. Do not open the reference course books (kept outside
  the repository). Real places, people and history: only well-established
  facts; when unsure, leave it out. Legends are told as legends ("secondo
  la leggenda"). Quote only titles or a few words of public-domain works.
  Describe a real artwork only as it is.
- Facts in the plan were checked: keep them. If you change or drop one, say
  so in the report with the reason; never replace a true fact with a guess,
  and never add a date, number or name you are not sure of.
- Standard Netherlands Dutch as people say and write it today, spelled by
  the Groene Boekje (pannenkoek, zo'n, ideeën, geüpdatet). Natural Dutch,
  nothing a Dutch speaker would find odd. Netherlands words in the text
  (patat or friet, slager, mobiel, leuk, zin), never Belgian ones.
- Flemish only in the Flemish portraits (18, 43) and in notes that name
  Belgium: there the narrator still writes standard Dutch, and each Flemish
  word is quoted, named as Belgian ("in Belgio si dice ...") and given its
  Netherlands equivalent. Never present a Flemish word or form as standard.
- Address in Dutch: u to strangers, customers, older people, officials; je
  / jij among friends, family, colleagues. The Italian matches (lei / tu),
  one form per speaker through the lesson; polite lei, le, la in lowercase.
- Italian: natural, tu to the learner ("ricorda", "nota", "fai
  attenzione"), never lei or voi to the learner. No English words in
  Italian text. Italian prose stays Italian: quote a Dutch word as an
  example, never use it as an Italian word ("ha preso la bici", not "ha
  preso la fiets").
- The country is "i Paesi Bassi" (plural: "i Paesi Bassi sono"); "Olanda"
  only for the two provinces of Holland, when the text means them. The
  language and the people stay "l'olandese", "gli olandesi". Flanders is
  "le Fiandre", its speech "fiammingo".
- Italian grammar: the congiuntivo where Italian wants it (penso che sia,
  sebbene fosse, prima che arrivi, affinché tu possa, sembra che abbia, è
  possibile che venga); se + congiuntivo imperfetto + condizionale (se
  avessi tempo, verrei), never "se avrei". Gender and number agree with the
  Italian word, never with the Dutch one: het meisje is "la ragazza ...
  lei", het huis "la casa ... la", de fiets "la bici"; a het-word is not
  masculine or neuter in Italian.
- In Italian text the earlier course is "Fondamenti", never "Foundation";
  this course is "questo corso".
- Back references: "(Fondamenti, lezione N)" or "(Fondamenti, lezione N:
  word)" where N is the Foundation lesson whose DIALOGUE first has that
  word or form. Find it with
  `python3 -I work/tools/first_seen.py work/nl/it/foundation 'regex'`
  (a word only in notes or culture does not count). For a grammar point, N
  is the lesson whose note explains it: the Foundation plan names it;
  open the lesson and check the note says it.
- "(lezione N)" alone means this course; use it for earlier lessons of this
  course (check with first_seen.py on work/nl/it/perfectionnement) and,
  sparingly, for later ones the plan names. A bracket names one course
  only: "(Fondamenti, lezione 72)" and "(lezione 3, nota 8)" go in separate
  brackets. Every "frase N", "nota N" and "lezione N, nota M" must point at
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
  put). The twist must follow from the text, and the note must explain the
  joke the text really tells.
- No recurring characters: a name used in an earlier lesson is another
  person; never write "lo conosci già (lezione N)" about a character.
- Explain idioms in their real sense first (op zwart zaad zitten = essere
  al verde), then the literal image. Use an idiom only in its real sense;
  if the scene does not fit the sense, change the scene. Fixed phrases in
  their real shape (blijven zitten, not "zitten blijven"); if you are not
  sure a phrase is said that way, write a plainer sentence.
- Grammar claims must be exact; do not overstate a rule ("sempre", "mai")
  when usage varies: say what is usual and what is also heard. Keep the
  truths Foundation taught: the finite verb second in a main clause and
  last in a subordinate one; after inverted je/jij the -t drops (word je?,
  but wordt u?); the adjective -e except before a het-noun with een, geen
  or no article; the perfect with zijn for a change of place or state;
  't kofschip; separable prefixes at the end and inside the participle
  (opgebeld), inseparable be-, ge-, ver-, ont- never split and without ge-.
- Stress or a sound in a note only where it is irregular or decides the
  meaning. For a stress pair write the Dutch way, with accents on the
  spelling: vóórkomen / voorkómen, óndergaan / ondergáán. When the sound
  itself is the point, never figure it by hand: copy the word's figure from
  a Foundation pron line, or run
  `python3 -I work/tools/figure_nl.py --say "word"` and copy its output,
  with the key's signs exactly (bold stress between asterisks, ‣, ë, χ, ü,
  œ, ø, I). A one-syllable word never carries asterisks.
- A note explains only what is in its own sentence: no stray words from
  elsewhere. It never claims something about another text without
  rereading it. Any count a note states (seven idioms, three verbs) is
  true: count it.
- Notes never mention the plan, the brief or "il gag del piano", and make no
  remarks about the lesson itself or about writing it: the learner never
  sees the plan. Write "la battuta", "il finale".
- Exercise sentences are small scenes, logical on their own, made of words
  the learner has met (Foundation and this course so far, plus the
  lesson's own); they restate the lesson's facts exactly (the same person
  does the same thing; a claim about a group holds for every member).
- Every reference to "ieri", "la settimana prossima" or another lesson
  fits the course order: lesson N is read after lesson N − 1.
- From lesson 50 the course is C1 and the active wave runs: do not describe
  how the app shows it beyond what lesson 49's aside says.

Rules added after week 1:
- Check the logic of every fact inside the text: who pays whom (the one who paid sends the Tikkie), what is already true at the start (a phone "on silent" cannot ring), who is relieved. A story that contradicts itself is the most common error.
- Never put an Italian word in a Dutch line ("dit corso"); never a time span the course cannot keep ("over een half jaar" when the lesson is five weeks away).
- wiens is for masculine and het-words, wier for feminine words (de stad, de provincie) and plurals; both are written language, waarvan in speech.
- A note's examples must be idiomatic Dutch: if you are not sure a sentence sounds natural, use the lesson's own sentence instead.
- Italian lines translate the Dutch line, nothing more (no added "una passeggiata, un caffè").

Rules added after week 2:
- The Dutch lines never contain Italian words, not even in quotes ("een kaartje is een biglietto" is wrong): explain in Dutch inside the text, and put the Italian only in the "it:" line and the notes.
- Never leave working notes to yourself in a lesson file ("(Note: ... replace it.)").
- Check irregular past forms you are not sure of: plegen → placht (never "plag"); if unsure, use vroeger + imperfect.
- In Italian translations use the Italian names of places where they exist (Groninga, Zelanda, Fiandre, L'Aia, Anversa, Gand).
- The closing line must follow from the whole text and pay off its premise (a careful saver pays cash; nobody asks "with a loan?" right after "I pay cash").

Rules added after week 3:
- Only idioms you are sure exist in standard Dutch ("Wat loopt hier los?" does not; "Wat is hier aan de hand?" does). When in doubt, leave the idiom out rather than invent its meaning.
- Times, amounts and clocks in a story must add up (a kitchen clock "behind" makes sense only if the real time is already past the limit).
- A word of one city or region (bolleke, Antwerp) is presented as that, not as "the Belgian word"; a word's literal origin must be right (a spuigat is a ship's scupper).
- "maar" joins two things that really contrast; check each "maar" sentence makes sense.

Rules added after week 4:
- A history text keeps one tense: the past (imperfect) for past events, the present only for what is still true today. Do not slip into the historical present halfway.
- Before you finish, search your files for any English or any comment to yourself ("actually", "fix", "note:") and remove it: a lesson file contains only the lesson.
- Use the current name of a museum or institution (Kunstmuseum Den Haag, no longer Gemeentemuseum) and the work's real Dutch title (Tekenende handen).

Rules added after week 5:
- Notes are written in Italian: Dutch words appear only as the quoted examples, never as part of the Italian sentence ("leggi het hardop" is wrong).
- A verse line is still a full Dutch sentence or a clean fragment: no missing verb ("ik hoor mezelf een zware zucht"), and rhymes must really rhyme.
- Reflexive idioms keep their reflexive pronoun (zich iets in het hoofd halen: wat haal je je in je hoofd?).
- Things that may have changed recently (where parliament sits, a museum's name) are written so they stay true: "van oudsher", "traditioneel".

Rules added after week 6:
- "had niet hoeven + infinito" is "non c'era bisogno di", never "non avresti dovuto" (that is "had niet moeten").
- History and status facts need their exact stages (Suriname: colony until 1954, a country within the Kingdom until 1975).
- Grammar statements in notes must be true in general; if a rule has exceptions or you are not sure, describe only the example in the text.

## Checks before you finish

- `target/release/marea --check work/nl/it/perfectionnement/NNN.txt`
  prints ok with no problem for each file.
- `python3 -I work/tools/extra_checks.py work/nl/it/perfectionnement/*.txt`
  reports no repeated title (the revision lessons' "Herhaling" excepted)
  and no dialogue line repeated in another lesson.
- Every "(Fondamenti, lezione N)" verified with first_seen.py; every
  "(lezione N, nota M)", "frase N" and "nota N" opened and counted.
- Reread every lesson once against this brief: the dialogue coherent line
  by line, the joke works and the notes explain it correctly, natural
  Netherlands Dutch, natural Italian (congiuntivo, agreement, "i Paesi
  Bassi"), one answer per blank, no Flemish word unlabelled, no doubtful
  fact, every count true, notes not hard-wrapped.
- Touch nothing but your own lesson files. No git, no cargo. The commands
  you may run besides reading files and grep: the checker, extra_checks.py,
  first_seen.py and figure_nl.py --say above.

Report briefly: per lesson items, words, notes; checker output; facts you
were unsure of and what you did with them; any choice the plan did not
settle; anything in the plan you think is wrong (quote it, do not change
it).
