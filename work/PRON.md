# Figured pronunciation pass

Repository: this one (paths below are relative to its root).

Lingua Onde follows the Assimil method. In the first 50 lessons (A1 and A2)
every dialogue line shows how it sounds, spelled for a reader of each base
language ("prononciation figurée"), so a beginner who knows no phonetics can
read it aloud and get close to the right sounds.

## The lines

Under each dialogue sentence of lessons 1–50, after the translations and any
`pron:` line, add one line per base language:

```
- Hallo! Ich heiße Jonas. Und du?
  it: Ciao! Mi chiamo Jonas. E tu?
  en: Hi! My name is (I am called) Jonas. And you?
  es: ¡Hola! Me llamo Jonas. ¿Y tú?
  pron.it: hallò! ich hàisse iònas. unt du?
  pron.en: ha-LOH! ikh HY-suh YOH-nas. oont doo?
  pron.es: jaló! ij jáise yónas. unt du?
```

(The example shows the shape only; the spellings to use come from the key.)

- `pron.it`, `pron.en`, `pron.es` for every course except Spanish, which
  takes only `pron.it` and `pron.en`.
- The whole sentence, word by word, in the reader's own spelling habits: an
  Italian reads `pron.it` as Italian, an English speaker reads `pron.en` as
  English, a Spanish speaker reads `pron.es` as Spanish. No IPA.
- Follow `work/<lang>/pron-key.md` exactly: it fixes for each reader how every
  sound is spelled, how stress is marked, how words that run together are
  joined, how tones are shown (Chinese, Vietnamese) and the few special
  symbols for sounds the reader's language lacks.
- Spell the real connected speech of the variety the course teaches (French
  liaisons and elisions, Dutch and German final devoicing, Portuguese vowel
  reduction as Brazilians speak, Chinese tone changes of 一 and 不 and third
  tone sandhi...).
- Write nothing else on these lines: no comments, no brackets, no slashes
  between alternatives.

## Pronunciation notes (lessons 1–25)

In lessons 1–25, each `notes.<base>` has at least one numbered note on
pronunciation for that reader: the sounds of this lesson that are new or
tricky for them, compared with their own language, using the key's
spellings. Lessons 1–3 together introduce the key's conventions (stress
mark, special symbols, tones). Add or adapt a note where needed; keep 4–6
notes per language and keep the existing notes' numbering in order.

## Other pronunciation lines

- German: delete every `pron:` line (the figured lines replace those hints).
- Chinese keeps its `pron:` pinyin; Latin keeps its `pron:` with macrons.
- Change nothing else in the lessons: not the text, translations or exercises.

## Check

Run `work/tools/lo-check-full <file>` after editing a file and fix every
problem it lists until it prints `ok`. It lists every A1–A2 dialogue line
that still lacks a `pron.<base>:` line.
