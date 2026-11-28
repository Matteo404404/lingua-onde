# Vietnamese for Spanish speakers, Perfectionnement: what every week must follow

Read first, in this order:
1. work/BRIEF.md: the section "Perfectionnement" and "Checks". Where it
   says Italian, `it:`, `notes.it`, read Spanish, `es:`, `notes.es`.
2. METHOD.md section 2, work/PATTERNS.md section "Perfectionnement",
   content/FORMAT.md.
3. The plan: work/vi/es/perfectionnement.md (rules at the top, then your
   lessons).
4. The model of a Perfectionnement lesson (in English for Italians, same
   format): work/en/perfectionnement/001.txt (a word lesson), 004.txt (a
   portrait), 005.txt (a story), 007.txt (a revision lesson). Copy their
   layout, length and note style, not their content.
5. The approved lessons of this course: work/vi/es/perfectionnement/ from
   001.txt to the last one written; read all lessons of the previous week.
6. Foundation is known to the learner: work/vi/es/foundation/*.txt (100
   lessons, northern Vietnamese, notes in Spanish).

Files: work/vi/es/perfectionnement/NNN.txt, one lesson each, starting:

    course: vi
    stage: perfectionnement
    bases: es

    == N B2      (C1 from lesson 46)
    numeral: Bài thứ nhất
    title: ...
    title.es: ...

Rules:
- `numeral:` the Vietnamese ordinal as in Foundation: Bài thứ nhất, Bài thứ
  hai, Bài thứ ba, Bài thứ tư, Bài thứ năm … Bài thứ mười bốn, Bài thứ hai
  mươi mốt, Bài thứ hai mươi tư … Bài thứ bảy mươi. No numeral.pron.es.
- `title:` a short Vietnamese title (the plan's, or better); `title.es:`
  its Spanish. Revision lessons: `kind: revision`, title "Ôn tập", title.es
  "Repaso".
- Everything original. Do not open the reference course books (kept outside the repository). Real places, people,
  history: only well-established facts; when unsure, leave it out. Quote
  only titles or a few words of old public-domain works.
- Northern Vietnamese (Hanoi), correct spelling and tone marks in the
  common print style (hòa, khỏe, thủy, quý), natural as people say and
  write it today. A southern or Huế word only where the plan or a note
  names it as such.
- Text in `dialogue:`, one item per numbered sentence or dialogue line:
  14–25 items, about 250–550 syllables, of the kind and with the ending the
  plan gives. Each item has `es:` with a free, natural Spanish
  translation; for an idiom, the Spanish meaning, then the literal sense
  in brackets: «estaba loco de alegría (izaba banderas en el vientre)».
- No pron lines, no culture section. A sound is noted inside a note only
  where it matters (a pun, a tone pair, a regional form), quoted with the
  figures of content/vi/pronunciation.md exactly.
- notes.es: 8–15 notes (word lessons 15–30, almost all idioms; the last
  note is the expression list, one line each: "ăn ảnh (frase 4, nota 5)",
  with the right sentence and note numbers). Each note starts from the
  words of the text: its header repeats them exactly as the text has them.
- Every note starts on its own line ("1. ..."), never glued to the one
  before.
- In Spanish text the earlier course is "Fundamentos", never "Foundation".
- Back references: "(Fundamentos, lección N)" or "(Fundamentos, lección N:
  word)" where N is the Foundation lesson whose DIALOGUE first has that word
  or form (grep the `^- ` lines of work/vi/es/foundation/*.txt; a word only
  in notes does not count). For a grammar point, N is the lesson whose note
  explains it. "(lección N)" alone means this course. A bracket names one
  course only: "(Fundamentos, lección 81)" and "(lección 3, nota 8)" go in
  separate brackets. Every "frase N", "nota N" and "lección N, nota M" must
  point at the right sentence or note: open it and count. Never put a
  reference next to a word that note does not contain.
- Spanish: neutral, tú to the learner ("recuerda", "fíjate"), never usted
  to the learner, no English words. Spanish prose stays Spanish: quote a
  Vietnamese word as an example, never use it as a Spanish word. In a
  translation, one speaker keeps one form of address (tú or usted) through
  the lesson, chosen as a Spanish speaker would (usted to an older
  stranger or a customer). Spanish spelling: "deme", "dime" (no accent).
  Dishes in Spanish text keep plain spelling: pho, bún chả, bánh mì.
- Xưng hô follows real use: the kinship word by age and relationship, kept
  through the scene; vâng, dạ, ạ and thưa go up the age ladder only. The
  notes explain the choice when it teaches something.
- Explain idioms in their real sense first, then the literal image.
- Grammar claims must be exact: do not overstate a rule ("siempre",
  "nunca") when usage varies; say what is usual and what is also heard.
  Tone names: no mark = ngang, à huyền, á sắc, ả hỏi, ã ngã, ạ nặng; check
  every tone a note names against the mark.
- Every Vietnamese sentence must make sense in context; linkers must be
  logical (no vì thế where nothing follows).
- A note explains only what is in its own sentence; it never claims
  something about another text without rereading it.
- Exercise sentences are small scenes, logical on their own. They restate
  the lesson's facts exactly. ex1: five NEW sentences of 5–20 syllables to
  translate (never copies of text lines), each with `es:`. ex2: five
  sentences with one to three [blanks] of one or two syllables each, with
  `es:` the plain Spanish sentence (never a hint or a riddle); each blank
  has only one right answer given the Spanish.
- Revision lessons (every 7th): `kind: revision`, no `dialogue:`; notes.es
  with a few introductory lines, then the plan's numbered sections with
  tables (`| ... |` rows as in work/en/perfectionnement/007.txt) and
  examples from the week's texts; ex1 and ex2 hold the practice (five +
  five), plain natural sentences, not puzzles.
- An idiom is used only in its real sense (ăn theo is "ride on someone
  else's success", not "imitate"); if the scene does not fit the sense,
  change the scene.
- Northern words in the text: là (iron), not ủi; a southern word only where
  the text is about the South.
- Never blank a word that has a near-synonym the Spanish cannot tell apart:
  bèn / liền, hóa ra / thì ra, vừa / mới (… đã), để / cho, để / nhằm.
- Every reference to "yesterday", "next week" or another lesson must fit
  the course's order: lesson N is read after lesson N − 1.
- Spoken lines stay plausible for the speaker: a guard does not ask what he
  should already know; a narrator does not call a character "chú ấy" in the
  character's own line.
- Social formulas go in their real direction: không dám and dạ are said by
  the younger to the elder, never the reverse. Kinship (North): the elder
  siblings of either parent are bác; cô, chú, cậu, dì are the younger ones.
- No Vietnamese words or sentences inside Spanish notes (not "so sánh
  với", "với người lạ", "mâm" as Spanish): write them in Spanish.
- No recurring characters: a name used in an earlier lesson is another
  person; never write "ya la conoces (lección N)" about a character.
- Use the word people really use: chuyến (a bus run), not suất; "tương
  truyền" for a tradition history does not confirm (who built a bridge).
- No Spanish words inside the Vietnamese text, and no advice to writers
  inside a lesson ("no los dejes en blanco juntos").
- A story keeps its own facts straight from line to line (a man who walks
  all day does not then park his motorbike). Describe a real artwork only
  as it is; when unsure of a detail, leave it out.
- Notes never mention the plan, the brief or "el gag del plan": the
  learner never sees them. Write "el gag", "el cierre".
- Any verse you write must keep its form exactly (lục bát: 6/8 syllables,
  2nd bằng, 4th trắc, 6th bằng, 8th bằng; the rhymes chained); check each
  position against the tone marks before writing a note about it.
- Hanoi words in northern scenes: bát (not tô), cốc (not chén or ly for
  water), kem caramen (flan). A joke's logic must hold (the TV, not the
  lamp, uses the power).
- Fixed phrases in their real shape: không biết trả lời ra sao, xới cơm
  (not đổ cơm), hay ra trò (ra trò after an adjective), chưa ra hồn, nói
  cho ra ngô ra khoai. If you are not sure a phrase is said that way,
  write a plainer sentence.
- Customs and scenes must hold: a giỗ is held where the altar is (the
  eldest son's house); a character who does not share a language needs
  someone to translate.
- Notes contain no remarks about the lesson itself ("el foco C1 vive
  aquí", "dato de manual", "no cites el cuerpo entero", "sin novelar")
  and no English words.
- Family terms follow the tree: ông nội is the father's father, so the
  father of the house is his son and the mother his daughter-in-law.
- Scenes fit Vietnamese life: no adversarial election campaigns or press
  conferences; a contested vote fits a residents' or building board.
  Northern verbs: giục, thúc (hối for "hurry" is southern).
- In Spanish, family members use tú with each other.
- `aside.es:` only where the plan says.
- From lesson 50 the course is C1 and the active wave runs: do not describe
  how the app shows it beyond what lesson 49's aside says.

Checks before you finish:
- `target/release/marea --check work/vi/es/perfectionnement/NNN.txt`
  prints ok with no problem for each file.
- Reread every lesson: the joke works and the notes explain it correctly,
  natural Vietnamese, natural Spanish, one answer per blank, no doubtful
  fact, every Fundamentos reference verified by grep, every count true.
- Touch nothing but your own lesson files. No git, no cargo.

Report briefly: per lesson items, syllables, notes; checker output; facts
you were unsure of; any choice the plan did not settle.
