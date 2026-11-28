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
- `target/release/marea --check work/es/en/foundation/NNN.txt` prints
  ok with no problem and no `gap:` line, for each file.
- `python3 -I work/tools/progression.py . es/en foundation 1 N`: no unmet
  exercise word, the new words within the plan's size.
- Reread each lesson: Spanish natural and correct, the twist works, the notes
  true, one answer per blank, every reference right.
- Touch nothing but your own lesson files. No git, no cargo.

Report briefly: per lesson the lines and new words, the checker output, any
word the tool figured wrongly, anything you were unsure of.

## Rules added after week 1

- Never explain a joke in a note ("the joke is in the last line"): the last
  line must work on its own. Notes explain language, not the story.
- No invented trivia ("the number you hear most often on a bus"), no claims
  about how a place works that you cannot be sure of (paying, opening hours,
  logos). A culture paragraph says less rather than guess.
- No literal gloss in brackets when it is the same as the translation.
- A note does not say the same thing twice in two sentences.
- refs in a revision lesson name earlier lesson numbers, not note sections.
- A revision scene must hang together as one conversation: each line answers
  the one before. Use the forms of the week (usted, también, hablamos).
- Foreign place names that the tool cannot figure (Leeds) are avoided; use a
  name Spanish writes in its own way (Brístol, Londres, Edimburgo).
- The figured form quoted inside sounds.en or notes.en must match the
  generated pron.en exactly (*lé*Hoss, not lé*Hos*).
- Exercise sentences are things a person would say; never "Buenas tardes,
  ¿y tú?" with nothing before it.

## Rules added after week 2

- Never promise a lesson number for a later topic unless the plan's block
  for that lesson says so; otherwise write "later" or "we will come back to
  it". A back-reference must name the lesson where the word really appeared
  (para was in lesson 6).
- Bold marks in a quoted figure must open and close: *tréïn*ta, never
  tréïn*ta.
- A note explains the sentence the dialogue really has: if the line is "el
  profesor soy yo", the note explains that, not a simpler "soy yo".
- English word-order claims must be right ("she is always tired": always
  comes after "is").
- A revision scene keeps one setting and one pair of speakers, and tú or
  usted stays the same for each speaker from first line to last.
- A revision section does not repeat a table from an earlier revision; it
  takes the next step.

## Rules added after week 3

- The twist line must say the reveal plainly in the scene's own words ("La
  jefa es ella"), not lean on a reading the learner has to guess.
- Translate the English that a British speaker would say: "I saw her this
  morning", "I got up at eight", with the literal Spanish perfect in the
  (gloss) if useful.
- Exercises use only forms already explained; no new phrase like "ya no"
  slipped into ex1.
- Stress claims about endings must be checked against the figured line: in
  -aba the stress is on the first a (lla*ma*ba), not on an í.
- The answer to "¿Y las comedias?" is "Tampoco": translate the sense ("not
  those either"), not a stock phrase that does not fit ("me neither").

## Rules added after week 4

- A twist must be possible: nobody fails to recognise a sister. If the
  plan's twist cannot happen, keep its setting and grammar and find a twist
  that can, said in the last line.
- Every line of a scene has a reason to be said by that speaker at that
  moment (no customer asking for "another good shop" after buying).
- No guillemets or quotes inside a dialogue line: write "están gritando:
  ¡gol!", so the figured line stays right.
- uno becomes un before a noun, also in compounds: ciento un euros,
  veintiún años.
- Plural claims: verde and azul have one form for both genders, but they do
  take -s and -es in the plural.
- A circular note ("as it is in ¡qué mala suerte!") is replaced by one that
  says what the form is.

## Rules added after week 5

- The dialogue obeys the lesson's own rule: hace un año takes the
  preterite (las compré hace un año), never the perfect.
- Before writing "lesson N" for a later topic, open that block of
  foundation.md and see that it is there (lesson 36 is the imperfect with
  the preterite; éxito is lesson 45). Otherwise write "later".
- A note on a form explains the form in the dialogue; never cite a form the
  dialogue does not have (se los pidas).
- An odd number of lines is fine: the scene does not need strict
  alternation to end on the twist.
- A vague question in a scene ("¿Y fue bueno?") is replaced by one that
  cannot be misread.

## Rules added after week 6

- Keep the plan's twist when it works; replace it only when it cannot
  happen, and never with a flat statement of fact ("Santiago está lejos del
  mar").
- A state in the past (not knowing, not having) is the imperfect: no tenía
  tu número, no sabía; saber in the preterite (supe) means "found out".
- A note does not explain a form that is only in the exercises.
- Every respelling in sounds.en uses the key's signs from figure_es.py
  --say (yo*bér*), never an English-style "yoh-vair".
- Word order in exercises is the natural one (coméis cuatro veces al día).

## Rules added after week 7

- The twist is the last line. Never add a reply after it.
- A revision scene is one conversation that moves forward: every line
  follows from the one before, nobody contradicts what they said two lines
  earlier, and it ends on the plan's twist.
- A note never wanders off the dialogue (no "a burglary is un robo" when no
  line has robo).
- A culture paragraph does not repeat an earlier lesson's (the portero was
  lesson 36).
- When a line is unclear in English ("the red one is mine" for "I have the
  same one"), change the Spanish so it says what is meant.

## Rules added after week 8

- If the plan's scene has no twist, make one that the scene sets up (the
  candidate's mother is from Cádiz, so he will understand the cook), not a
  line that repeats what has already happened.
- No line after the twist to tidy the story up ("Pues pagamos los dos").
- A blank never hides a number or a word the English gives away letter for
  letter; blank the verb form the lesson teaches.
- A verb's preposition is explained with the verb (aprender a), not called
  a "to-infinitive".

## Rules added after week 9

- A joke must survive its own language: if the whole scene is in Spanish,
  a character cannot be shown not understanding Spanish.
- Pronoun position in the negative command: no + pronoun + verb (no te
  pares); the pronoun never comes before no.
- Cultural words with no English match (padrino at a wedding) keep the
  Spanish word in the translation and the note says what it is; never
  force a wrong English equivalent ("best man").
- A line never contradicts the one before ("¿Hay camas?" "Sí, pero está
  completo").

## Rules added after week 10

- The conditional of probability is about the past (estaría cansada,
  "she must have been tired"); a guess about now takes the future (estará
  cansada).
- Facts with a number are given as safely as they are known ("a más de
  doscientos kilómetros"), never rounded into something false.
- Within one lesson the times and promises stay consistent (if the clerk
  said "en media hora", nobody later says "a las doce").
- A note never calls a correct variant wrong (en camino and de camino are
  both said).
- Spanish relatives: "una pregunta a la que el cuadro no dice nada" is not
  Spanish; when unsure, use the plain que.
- Valencian or Catalan spellings with à crash the tool: use the Spanish
  spelling the RAE gives (mascletá).

## Rules added after week 11

- Before writing a line, know who says it: in a two-person scene, count the
  lines so that each speaker's words fit what that person knows and wants.
- A non sequitur is not a joke ("Aunque sea un año, lo pago yo" answering
  nothing).
- An exercise sentence never contradicts the lesson's story or its own
  logic (if booking earlier would have saved them from the room, it is "no
  nos habría tocado").
- culture.en, aside.en: the text goes on the line after the tag, as in the
  models.
- A back-reference names the lesson where the form was really taught (the
  pluperfect is lesson 53; si with the present is lesson 44).

## Rules added after week 12

- A twist that hangs on a name must be consistent with every earlier line:
  if the best man reads his own wedding notes, the names he reads are his
  own (Laura y Roberto), and nobody else in the scene contradicts that.
- In a formal argument, the twist reveals a motive the scene has hidden
  (the neighbour who proposed the change is the one who gains from it).
- With haber the participle never agrees (se me ha roto el plato, se me
  han olvidado los nombres); only the verb follows the thing.
- Aunque and por eso never go in the same sentence: "como ..., ..." is the
  way to give a cause.
- A verb with its own se (volverse, irse) is not the impersonal se.

## Rules added after week 13

- Never write "culture.en: none."; leave the section out.
- Abbreviations the tool cannot figure (P.D.) are written in full in a
  dialogue line (Posdata).
- Greetings or words in Spain's other languages only when you are sure of
  them; leave out what you are not sure of.
- In a lesson whose idioms are the point, the twist still has to be a
  twist the scene sets up, not a line that repeats an idiom.
- Read the whole scene once more before finishing: if a complaint, a
  question or a fact is dropped halfway (the air conditioning that works
  after all), the scene is rewritten.

## Rules added after week 14

- A line meant to be said in English has the same text in its en: line;
  figure_es.py then leaves it without a figure. Use this only where the
  plan asks for English.
- A recap scene is not padded with polite lines that lead nowhere; each
  line moves the conversation to its twist.
