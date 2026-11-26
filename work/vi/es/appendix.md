You write the book around the Vietnamese Foundation course for Spanish speakers (northern Vietnamese), in this repository. The 100 lessons are finished and approved: work/vi/es/foundation/001.txt to 100.txt (revision lessons 7, 14 ... 98).

Read first:
1. content/en/appendix-foundation.it.txt in full: the model for every section (layout, tone, length). It is the English course for Italian speakers; yours is the Vietnamese course for Spanish speakers. Copy its format exactly; never copy its English-specific content.
2. content/vi/pronunciation.md: the key (tone arrows → ✓ ↑ ↓, the two rows, respelling for Spanish readers).
3. work/vi/es/writer.md (the course's rules) and work/vi/es/foundation.md (the plan).
4. All fourteen revision lessons (007, 014, ..., 098) in full.
5. The raw skeleton: work/appendix-skeleton/forms.tsv (write it first with: python3 work/tools/appendix_draft.py . vi/es foundation work/appendix-skeleton) (every syllable form of the dialogues with the lessons where it appears, first one first). For the grammar, read the numbered sections of the revision lessons' notes.es directly.

Write content/vi/appendix-foundation.es.txt, starting:

    course: vi
    stage: foundation
    base: es

and then these sections, in Spanish (tú to the learner, neutral Spanish, no English words):

== intro
As in the model: ## Bienvenido (what the course is: 100 lessons, one a day, northern Vietnamese of Hanoi, where the south differs the notes say so), ## Algunos consejos (the model's advice, adapted: half an hour a day, the first lessons are the strangest — six tones and new sounds —, don't learn by heart, speak aloud, go on when something is unclear, listen to Vietnamese outside the course, enjoy the jokes), ## Cómo se estudia una lección (the app's steps, as in the model, in Spanish), ## La pronunciación figurada (the 🔈 line: Spanish respelling, one figured syllable per Vietnamese syllable, the tone arrow after each, high tones on the upper row and low on the lower, full in lessons 1–14 only; then the learner reads the Vietnamese spelling, which is regular; the key is the page "Pronunciación"), ## El repaso (every 7th lesson), ## Las dos olas (from lesson 50, one earlier lesson a day, as lesson 50's aside says), ## Al final del libro (grammar, index, words, expressions).

== grammar
14 to 18 sections `## N Title` (Spanish), grouping by theme what the revision lessons explain: the six tones and the figured pronunciation; address by age and kinship (ông, bà, cô, chú, bác, anh, chị, em, cháu, cụ, tôi, mình; vâng / dạ / ạ); là and adjectives without a verb; questions (ai, gì, đâu, nào, mấy, bao nhiêu, có … không, phải không, à, chưa, sao, bao giờ, khi nào); classifiers and measure words (cái, con, quả, viên, ngôi, cốc, bát, chục); numbers, prices, the clock, dates and years; time markers (đã, đang, sẽ, sắp, rồi, chưa, vẫn, còn, không … nữa); habits and the past (hồi bé, ngày xưa, hay, thường); directional verbs; comparison (hơn, nhất, càng … càng, cũng vậy / cũng không); được and bị; requests and favours (cho, giúp, nhờ, hộ, làm ơn, xin, mượn / cho mượn); conditions and wishes (nếu … thì, miễn là, trừ khi, kẻo, giá mà, ước gì, lẽ ra, đáng lẽ); cause, concession and linking (vì … nên, tại vì, mặc dù, dù, tuy vậy, để, trước khi, sau khi, trước tiên … sau đó … cuối cùng); reported speech (nói là, bảo, hỏi); sentence particles (nhé, nhỉ, đi, thôi, chứ, mà, à, đã, thế); deduction (chắc, có lẽ, không thể); north and south. Each section: one paragraph that states the rule exactly as the lessons do (no new claims, no "siempre / nunca" the lessons do not make), ending with "(lecciones N, N, N)"; then a table `| Forma | Ejemplo | Lección |` whose examples are quoted exactly from the lessons' texts (dialogue or exercises) with the lesson where they appear. Every example must be findable in that lesson: grep it.

== index
One line per topic, `tema en español | lesson numbers`, sorted alphabetically as in the model, 80–140 lines. The numbers are the lessons that explain the point in their notes plus the revision lesson that sums it up. Every number must be a lesson that really treats the topic: check.

== words
`palabra vietnamita | traducción española | lessons`, sorted in Vietnamese alphabetical order (a ă â b c d đ e ê g h i k l m n o ô ơ p q r s t u ư v x y, tones ignored for the order), 500–900 lines, as in the model's `== words`. Use forms.tsv: group syllables into words as Vietnamese writes them (bánh mì, cà phê, xe máy, người ta, bây giờ…), one line per word, with its correct tone marks. The translation is the meaning the lessons give, short. The FIRST lesson number must be a lesson whose dialogue or exercises contain the word (the app's test checks this: grep the `- ` lines of that file). Later numbers: a few other lessons whose texts use it.

== phrases
`expresión vietnamita | sentido en español | lessons`, sorted the same way, 120–250 lines: fixed phrases, idioms, greetings and formulas the notes explain (không có gì, không sao, xin lỗi, cảm ơn, mời … vào, đi thôi, thế thì, dễ như ăn cháo…). Same rule for the first lesson number.

Rules: everything in Spanish except the Vietnamese forms; Vietnamese always with its correct tone marks (common print style: hòa, khỏe, thủy); no fact or rule the lessons do not contain; no figured pronunciation in the tables.

Strict limits: create or edit only content/vi/appendix-foundation.es.txt. Never touch src/, work/, other content files. Never run git or cargo. You may run grep and read files.

Before finishing: check every table example and every first lesson in `== words` and `== phrases` with grep against work/vi/es/foundation/NNN.txt. Report briefly: number of grammar sections, index lines, words, phrases; anything you were unsure of.
