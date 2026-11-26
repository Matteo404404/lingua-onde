You write the end of the book of the Vietnamese Perfectionnement course for Spanish speakers (northern Vietnamese), in this repository. The 70 lessons are finished and approved: work/vi/es/perfectionnement/001.txt to 070.txt (revision lessons 7, 14 ... 70).

Read first:
1. content/en/appendix-perfectionnement.it.txt, sections `== grammar`, `== index` and `== phrases`: the model for layout, tone and length (it is the English course for Italian speakers; copy its format, never its English-specific content). Also content/vi/appendix-foundation.es.txt, `== grammar` and `== phrases`, for the Spanish style of this course.
2. content/vi/appendix-perfectionnement.es.txt: it has `== intro` only. Keep the intro unchanged.
3. work/vi/es/perf-writer.md (the course's rules) and work/vi/es/perfectionnement.md (the plan).
4. All ten revision lessons (007, 014, ..., 070) in full, and the word lessons (001 ăn, 009 chơi, 015 mặt, 025 đi, 033 lòng, 040 bụng, 045 làm, 053 ra, 060 nói) with their expression lists at the end of the notes.
5. The raw skeleton: work/appendix-skeleton/grammar.tsv (write it first with: python3 work/tools/appendix_draft.py . vi/es perfectionnement work/appendix-skeleton) (revision section title, revision lesson, lessons it names).

Append to content/vi/appendix-perfectionnement.es.txt, after the intro, three sections, in Spanish (tú to the learner, neutral Spanish, no English words):

== grammar
14 to 18 sections `## N Title` (Spanish), grouping by theme the finer points of the revision lessons: address and kinship in the North (bác, chú, cô, cậu, dì; anh / em between cousins; không dám, dạ, thưa going up the ladder); politeness and face (thể diện, refusing and thanking, euphemisms for death, age and money); sentence particles and their attitude; time and aspect in narration (vừa, mới, đã, rồi, bèn, liền, chợt); cause, result and concession linkers; được and bị in narration and in the news; the language of the news and of offices (Hán Việt compounds, formulas of letters and reports); Hán Việt and native doublets and word order; classifiers and measure words beyond Fundamentos; reduplication and sound words; comparison and the figures of proverbs; idioms with body words (mặt, lòng, bụng); verse (lục bát); the word families as one table pointing to the word lessons. Use only what the revision lessons and notes contain; drop a theme they do not treat. Each section: one paragraph in Spanish that states the rule exactly as the lessons do (no new claims, no "siempre / nunca" the lessons do not make), ending with "(lecciones N, N, N)"; then a table `| Forma | Ejemplo | Lección |` whose examples are quoted exactly from the lessons' texts (dialogue or exercises) with the lesson where they appear. Every example must be findable in that lesson: grep it.

== index
One line per topic, `tema en español | lesson numbers`, sorted alphabetically as in the models, 60–120 lines. The numbers are the lessons that explain the point in their notes plus the revision lesson that sums it up. Every number must be a lesson that exists (1–70) and really treats the topic: check.

== phrases
`expresión vietnamita | sentido en español | lessons`, sorted in Vietnamese alphabetical order (a ă â b c d đ e ê g h i k l m n o ô ơ p q r s t u ư v x y, tones ignored for the order), 150–300 lines: the expressions of the word lessons' lists, and the idioms (thành ngữ), proverbs (tục ngữ), fixed phrases and formulas explained in the other lessons' notes. The FIRST lesson number must be a lesson whose dialogue or exercises contain the expression exactly as written (the app's test checks this: grep the `- ` lines of that file, which include ex1/ex2 items). The Spanish is the meaning the notes give, short, the idiom in its real sense. Later numbers: other lessons whose texts use it.

Rules: everything in Spanish except the Vietnamese forms; Vietnamese always with its correct tone marks (common print style: hòa, khỏe, thủy); "Fundamentos", never "Foundation", in Spanish text; no fact or rule the lessons do not contain; no figured pronunciation in the tables.

Strict limits: edit only content/vi/appendix-perfectionnement.es.txt. Never touch src/, work/ (except writing the skeleton above), other content files. Never run git or cargo. You may run grep and read files.

Before finishing: check every table example and every first lesson in `== phrases` with grep against work/vi/es/perfectionnement/NNN.txt. Report briefly: number of grammar sections, index lines, phrases; anything you were unsure of.
