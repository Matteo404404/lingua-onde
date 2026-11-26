You write the end of the book of the English (British) Perfectionnement course for Italian speakers, in this repository. The 70 lessons are finished and approved: work/en/perfectionnement/001.txt to 070.txt (revision lessons 7, 14 ... 70).

Read first:
1. content/en/appendix-foundation.it.txt, sections `== grammar`, `== index` and `== phrases`: the model for layout, tone and length. Copy its format exactly.
2. content/en/appendix-perfectionnement.it.txt: it has `== intro` only. Keep the intro unchanged.
3. work/en/perfectionnement.md, section "The book around the lessons".
4. All ten revision lessons (007, 014, ..., 070) in full, and the word lessons (001 take, 009 put, 015 go, 025 come, 033 turn, 040 run, 045 break, 053 keep, 060 mind) with their expression lists at the end of the notes.
5. The raw skeleton: work/appendix-skeleton/grammar.tsv (write it first with: python3 work/tools/appendix_draft.py . en perfectionnement work/appendix-skeleton) (revision section title, revision lesson, lessons it names).

Append to content/en/appendix-perfectionnement.it.txt, after the intro, three sections:

== grammar
16 to 20 sections `## N Title` (Italian), grouping the finer points of the revision lessons by theme (tenses of narrative and the future in the past; would, used to and the past; modals of probability, deduction, needn't have; conditionals, mixed and inverted; reporting and passive "it is said"; participle clauses and nominalisation; relatives with prepositions, where, whose; articles with abstract nouns; -ing or to; emphasis, inversion, do; ellipsis and substitution; so and such, compound nouns; hedging; indirect questions; discourse markers; register: formal letters, jargon, euphemism and understatement; word formation; the verb families as one table pointing to the word lessons). Each section: one paragraph in Italian that states the rule exactly as the lessons do (no new claims, no "always/never" the lessons do not make), ending with "(lezioni N, N, N)"; then a table `| Forma | Esempio | Lezione |` whose examples are quoted exactly from the lessons' texts (dialogue or exercises) with the lesson where they appear. Every example must be findable in that lesson: grep it.

== index
One line per topic, `topic in Italian (English term in brackets where the lessons use it) | lesson numbers`, sorted alphabetically like Foundation's, 60–120 lines. The numbers are the lessons that explain the point in their notes plus the revision lesson that sums it up. Every number must be a lesson that exists (1–70) and really treats the topic: check.

== phrases
`expression | Italian meaning | lessons`, sorted alphabetically, 150–300 lines: the expressions of the word lessons' lists, and the idioms, proverbs, fixed phrases and phrasal verbs explained in the other lessons' notes. The first lesson number must be a lesson whose dialogue or exercises contain the expression (the app's test checks this: grep the `- ` lines and `ex1`/`ex2` lines of that file). The Italian is the meaning the notes give, short, idiom in its real sense. Later numbers: other lessons whose texts use it.

Rules: everything in Italian except the English forms; keep technical terms as the lessons write them; lowercase lei; "Fondamenti", never "Foundation", in Italian text; no fact or rule the lessons do not contain. No pronunciation transcriptions.

Strict limits: edit only content/en/appendix-perfectionnement.it.txt. Never touch src/, work/, other content files. Never run git or cargo. You may run grep and read files.

Before finishing: check every table example and every first lesson in `== phrases` with grep against work/en/perfectionnement/NNN.txt. Report briefly: number of grammar sections, index lines, phrases; anything you were unsure of.
