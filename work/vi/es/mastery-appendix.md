You write the end of the book of the Vietnamese Mastery course for Spanish speakers ("Maestría" in Spanish text), northern Vietnamese, in this repository. The 100 lessons are finished and approved: work/vi/es/mastery/001.txt to 100.txt (revision lessons 7, 14 ... 98).

Read first:
1. content/vi/appendix-perfectionnement.es.txt, sections `== grammar`, `== index` and `== phrases`: the model for layout, tone and length, in this course's Spanish. Also content/en/appendix-mastery.it.txt `== grammar` for how a Mastery appendix treats style (format only; never copy its English content).
2. content/vi/appendix-mastery.es.txt: it has `== intro` only. Keep the intro unchanged.
3. work/vi/es/mastery.md (the week themes and each lesson's focus) and work/vi/es/mastery-writer.md (the course's rules).
4. All fourteen revision lessons (007, 014, ..., 098) in full: their notes already put each week's devices in order with tables. Then the notes of the other lessons as needed.

Append to content/vi/appendix-mastery.es.txt, after the intro, three sections, in Spanish (tú to the learner, neutral Spanish, no English words):

== grammar
14 to 18 sections `## N Title` (Spanish), one or more per week theme, as style and grammar of C2: register and phatic talk (lời xã giao); saying less (nói giảm nói tránh, litotes, polite irony); persuasion and rhetoric (kính thưa, triads, đành rằng … nhưng, refusing without không); telling a story (order, viewpoint, the corrected tale); the precise word (death verbs, Hán Việt and native pairs, từ láy, false friends); work and the law (contracts, letters, minutes, reports); the language of knowledge (hedging: có thể cho rằng, chưa đủ cơ sở; nominalisation); Vietnamese across regions and abroad (only the features the lessons state: Saigon, Huế, Nghệ, highlands, Việt kiều, teencode); humour (nói lái, chuyện trạng, riddles); the music of Vietnamese (lục bát, song thất lục bát, hát ru, câu đối, tones and melody — the rules exactly as the lessons state them); the press (xã luận, tít, phóng sự, fallacies); real speech (fillers, particles, phone and family rituals); tact (condolence, apology, feeling); Vietnamese through time and between Spanish and Vietnamese. Each section: one paragraph in Spanish that states the point exactly as the lessons do (no new claims, no "siempre / nunca" the lessons do not make), ending with "(lecciones N, N, N)"; then a table `| Forma | Ejemplo | Lección |` whose examples are quoted exactly from the lessons' texts (dialogue or exercises) with the lesson where they appear. Every example must be findable in that lesson: grep it.

== index
One line per topic, `tema en español | lesson numbers`, sorted alphabetically as in the models, 80–140 lines. The numbers are the lessons that explain the point in their notes plus the revision lesson that sums it up. Every number must be a lesson that exists (1–100) and really treats the topic: check.

== phrases
`expresión vietnamita | sentido en español | lessons`, sorted in Vietnamese alphabetical order (a ă â b c d đ e ê g h i k l m n o ô ơ p q r s t u ư v x y, tones ignored, a shorter word before a longer one), 150–300 lines: the idioms (thành ngữ), proverbs (tục ngữ), fixed phrases, formulas and collocations explained in the lessons' notes. The FIRST lesson number must be a lesson whose dialogue or exercises contain the expression exactly as written (the app's test checks this: grep the `- ` lines of that file, which include ex1/ex2 items). The Spanish is the meaning the notes give, short, the idiom in its real sense. Later numbers: other lessons whose texts use it.

Rules: everything in Spanish except the Vietnamese forms; Vietnamese always with its correct tone marks (common print style: hòa, khỏe, thủy); "Fundamentos", "Perfeccionamiento", "Maestría", never the English course names; no fact or rule the lessons do not contain; no figured pronunciation; no remarks about originality, copyright, the plan or the writer.

Strict limits: edit only content/vi/appendix-mastery.es.txt. Never touch src/, work/ (except writing scratch files under work/appendix-skeleton/ if you need them), other content files. Never run git or cargo. You may run grep, read files, and run python3 work/tools/appendix_draft.py . vi/es mastery work/appendix-skeleton to get the grammar skeleton (grammar.tsv: revision sections with the lessons they name).

Before finishing: check every table example and every first lesson in `== phrases` with grep against work/vi/es/mastery/NNN.txt. Report briefly: number of grammar sections, index lines, phrases; anything you were unsure of.
