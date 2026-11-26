# Vietnamese for Spanish speakers, Mastery: what every week must follow

Read first, in this order:
1. work/BRIEF.md: section "Mastery" (and "Checks"). Where it says Italian,
   `it:`, `notes.it`, `task.it`, `rubric.it`, read Spanish: `es:`,
   `notes.es`, `task.es`, `rubric.es`.
2. METHOD.md section 3, content/FORMAT.md (`tasks:`, `task.es:`,
   `rubric.es:`, `refs:`).
3. The plan: work/vi/es/mastery.md (rules at the top, then your week's table).
4. The model of a Mastery lesson (English for Italians, same format):
   work/en/mastery/001.txt, 004.txt and 007.txt (a revision). Copy their
   layout, length and note style, not their content.
5. The approved lessons of this course written so far (work/vi/es/mastery/),
   all of the previous week. Before Mastery 8 exists, read Perfectionnement
   lessons 36, 46, 47, 48 and 49 (work/vi/es/perfectionnement/) for the note
   style and the Spanish.
6. Perfectionnement and Foundation are known to the learner:
   work/vi/es/perfectionnement/*.txt, work/vi/es/foundation/*.txt.

Files: work/vi/es/mastery/NNN.txt, one lesson each, starting:

    course: vi
    stage: mastery
    bases: es

    == N C1      (C2 from lesson 26)
    numeral: Bài thứ nhất
    title: ...
    title.es: ...
    kind: revision    (only on revision lessons; title "Ôn tập", "Repaso")

`numeral:` the Vietnamese ordinal as in the earlier courses (Bài thứ nhất,
Bài thứ hai, Bài thứ ba, Bài thứ tư … Bài thứ mười bốn, Bài thứ hai mươi
mốt, Bài thứ hai mươi tư … Bài thứ một trăm).

Shape (BRIEF "Mastery" is the authority):
- `dialogue:` one item per numbered sentence of the text: 25–40 items,
  about 450–900 syllables, of the kind and with the ending the plan gives.
  Each item has `es:` with a free, natural Spanish translation (idioms:
  the Spanish meaning, then the literal sense in brackets, only where it
  helps). No pron lines, no sounds, no culture.
- `notes.es:` 10–18 notes on register and its shifts, xưng hô, what is meant
  but not said, irony, cultural and historical references, thành ngữ and
  collocations, Hán Việt against native words, the precise word against its
  near-synonyms, rhetoric; contrasts with Spanish. Each note starts on its
  own line ("1. ...") with the words of the text, exactly as the dialogue
  has them. A note is at most about 120 words.
- `ex1:` five demanding sentences or short passages to translate, where the
  nuance matters, each with `es:`; new sentences, never copies of the text.
  `ex2:` five collocations, idioms or connectors in context, one or two
  syllables per [blank], with `es:` a plain Spanish translation; each blank
  has only one right answer given the Spanish (never blank one of two
  near-synonyms the Spanish cannot tell apart).
- `tasks:` the plan's one or two tasks. Each item: the model answer as the
  item's text (Vietnamese, of the length the task asks), `task.es:` the
  instruction in Spanish, `rubric.es:` four or five self-check points
  separated by `;` (meaning; register; structure; idiom; effect, adapted).
- Revision lessons (7, 14 ... 98): `kind: revision`; `dialogue:` the revision
  text, 20–30 items, each with `refs:` to the week's lessons its words or
  devices come from; `notes.es:` a few lines, then four to six numbered
  sections that put the week's focus in order, with tables (`| ... |` rows)
  and examples quoted from the week's texts; no ex1/ex2; one task.
- `aside.es:` only where the plan says.

Rules:
- Everything original. Do not open the reference course books (kept outside the repository). Literary pieces are "in the
  manner of" an era or genre, never imitating a living author's text; quote
  only titles or a few words of public-domain works. Real places, people,
  history: only well-established facts; when unsure, leave it out.
  Invented books, studies and companies use invented names, never a real
  publisher or institution.
- Northern Vietnamese as the standard, correct spelling and tone marks in
  the common print style (hòa, khỏe, thủy, quý), natural as people say and
  write it today. Hanoi words in northern scenes (bát, cốc, ô, là, bố / mẹ);
  a regional word only where the text is about that region, and then the
  real one (if unsure of a dialect feature, leave it out).
- Xưng hô follows real use: the kinship word by age and relationship, kept
  through the scene; vâng, dạ, ạ, thưa, không dám go up the age ladder only.
  Kinship (North): the elder siblings of either parent are bác; cô, chú,
  cậu, dì are the younger ones.
- Spanish: neutral, tú to the learner, never usted to the learner; one form
  of address per speaker through a lesson in the translations (usted to an
  older stranger or a customer). Spanish prose stays Spanish: Vietnamese
  words only quoted as examples, never used as Spanish words. "Deme",
  "dime" without accent. Dishes: pho, bún chả, bánh mì.
- In Spanish text the earlier courses are "Fundamentos" and
  "Perfeccionamiento"; this course is "Maestría". References: "(lección N)"
  or "(lección N, nota M)" is this course; "(Perfeccionamiento, lección N,
  nota M)" and "(Fundamentos, lección N)" name the earlier courses; one
  course per bracket. Every reference must point to a note or dialogue
  line that really has that word or device: open it and check
  (python3 work/tools/first_seen.py DIR 'word' finds the first dialogue
  line). If no earlier note says it, give no reference.
- Tone names: no mark = ngang, à huyền, á sắc, ả hỏi, ã ngã, ạ nặng; check
  every tone a note names against the mark. Any verse you write keeps its
  form exactly (lục bát: 6/8 syllables, 2nd bằng, 4th trắc, 6th bằng, 8th
  bằng, 6th and 8th of the eight one ngang and one huyền, the rhymes
  chained; song thất lục bát and câu đối likewise); check every position
  before writing a note about it. Counts of lines and syllables match.
- A note explains only what is in its own sentence, and never claims
  something about another text without rereading it. When a note says who
  does what in the text, check it against the text.
- Grammar and usage claims exact: no "siempre / nunca" where usage varies;
  say what is usual and what is also heard. Idioms in their real sense
  first; irony and implicature: what is said, what is meant, how the reader
  knows (only where implicature is the point).
- The story must be plausible and keep its own facts from line to line
  (places, days, means of transport, who did what). A joke's logic holds.
  Any count the text states is true. A revision introduces nothing the week
  did not have and gives no character someone else's act; it reuses the
  week's people with their own names.
- Legal and official documents fit Vietnamese practice on their date; if
  unsure, keep them generic.
- Notes never mention the plan, the brief, the writer or copyright ("el gag
  del plan", "texto original", "no citamos…"): the learner never sees them.
  No writer's memo left in the text.
- Fixed phrases in their real shape, and the word people really use: không
  biết trả lời ra sao, xới cơm, hay ra trò, chưa ra hồn, nói cho ra ngô ra
  khoai, mốc (mould; nấm is a mushroom), kính lão (reading glasses; kính
  cận is for myopia), ngã đánh oạch. Northern verbs: giục, thúc (hối for
  "hurry" is southern). If unsure a phrase is said that way, write a
  plainer sentence.
- Family terms follow the tree (ông nội is the father's father; the mother
  of the house is then his daughter-in-law). Customs and scenes must hold:
  a giỗ is held where the altar is; someone without the language needs an
  interpreter; Vietnamese public life has no adversarial campaigns or
  press-conference attacks (a contested vote fits a residents' board).
- In the Spanish, family members use tú with each other.
- A figure is right in its order of magnitude (coffee: over a million
  tonnes a year) or left out.
- Notes contain no remarks about the lesson itself ("el foco C1", "dato de
  manual", "no inventamos…", "el cierre del plan") and no English words
  ("hedging": write "atenuación").
- Model answers in tasks are good, natural Vietnamese of the asked length
  and register, not a copy of the text, and they meet their own rubric.

Checks before you finish:
- `target/release/lingua_onde --check work/vi/es/mastery/NNN.txt` prints ok
  with no problem and no `gap:` line, for each file.
- Count items and syllables of each text against the plan's ranges.
- Reread every lesson: the ending and the joke work and the notes explain
  them correctly; one answer per blank; no doubtful fact; every reference
  verified by opening the cited text.
- Touch nothing but your own lesson files. No git, no cargo.

Report briefly: per lesson items, syllables, notes, tasks; checker output;
facts you were unsure of; any choice the plan did not settle.
