You write the end of the book of the English (British) Mastery course for Italian speakers ("Padronanza" in Italian text), in this repository. The 100 lessons are finished and approved: work/en/mastery/001.txt to 100.txt (revision lessons 7, 14 ... 98).

Read first:
1. content/en/appendix-perfectionnement.it.txt, sections `== grammar`, `== index` and `== phrases`: the model for layout, tone and length. Copy its format exactly.
2. content/en/appendix-mastery.it.txt: it has `== intro` only. Keep the intro unchanged.
3. work/en/mastery.md (the week themes and each lesson's focus).
4. All fourteen revision lessons (007, 014, ..., 098) in full: their notes already put each week's devices in order with tables. Then the notes of the other lessons as needed.

Append to content/en/appendix-mastery.it.txt, after the intro, three sections:

== grammar
14 to 18 sections `## N Title` (Italian), one or more per week theme, as style and grammar of C2: register and phatic language; irony and understatement (litotes); rhetoric (triads, anaphora, concession and rebuttal, hedged refusal); narrative style (tense and pace, period style, free indirect style); the precise word (near-synonyms, collocations, make/take); work and law (formal and legal register, hedging in reports); the language of knowledge (academic register, hedging, nominalisation); Englishes of the world (only the features the lessons state); humour (puns, deadpan, limericks); sound (rhyme, slant rhyme, alliteration, metre as the lessons count it); argument and the press (fallacies, headlinese, obituary style); spoken English (fillers, like, you know, vague language); tact (condolence, apology versus non-apology, understatement of feeling); English through time and translation (Old to Early Modern English, thou and -eth, word formation, false friends). Each section: one paragraph in Italian that states the point exactly as the lessons do (no new claims, no "always/never" the lessons do not make), ending with "(lezioni N, N, N)"; then a table `| Forma | Esempio | Lezione |` whose examples are quoted exactly from the lessons' texts (dialogue or exercises) with the lesson where they appear. Every example must be findable in that lesson: grep it.

== index
One line per topic, `topic in Italian (English term in brackets where the lessons use it) | lesson numbers`, sorted alphabetically like Perfectionnement's, 80–140 lines. The numbers are the lessons that explain the point in their notes plus the revision lesson that sums it up. Every number must be a lesson that exists (1–100) and really treats the topic: check.

== phrases
`expression | Italian meaning | lessons`, sorted alphabetically, 150–300 lines: the idioms, proverbs, fixed phrases, collocations and phrasal verbs explained in the lessons' notes. The expression in its base form only if that base form appears literally in the text; otherwise write it as it appears. The first lesson number must be a lesson whose dialogue or exercises contain the expression exactly, case aside (the app's test checks this: grep the `- ` lines and `ex1`/`ex2` lines of that file). The Italian is the meaning the notes give, short, idiom in its real sense. Later numbers: other lessons whose texts use it.

Rules: everything in Italian except the English forms; keep technical terms as the lessons write them; lowercase lei; "Fondamenti", "Perfezionamento", "Padronanza", never the English course names, in Italian text; no fact or rule the lessons do not contain. No pronunciation transcriptions. No remarks about originality or copyright.

Strict limits: edit only content/en/appendix-mastery.it.txt. Never touch src/, work/, other content files. Never run git or cargo. You may run grep and read files, and `target/release/lingua_onde --check content/en/appendix-mastery.it.txt` if it accepts the file.

Before finishing: check every table example and every first lesson in `== phrases` with grep against work/en/mastery/NNN.txt. Report briefly: number of grammar sections, index lines, phrases; anything you were unsure of.
