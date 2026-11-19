# Lingua Onde

A desktop app for learning languages by assimilation: a short dialogue a day,
heard and repeated aloud, understood through your own language, with notes,
two exercises, review of earlier lessons and, later, an active "second wave".

The built-in courses are original lessons written for this app (Spanish,
French, German, Portuguese from Brazil, Mandarin with pinyin, Latin with vowel
lengths, and Vietnamese). Translations and notes are in Italian, English, and
Spanish. Each language has 13 lessons so far (8 at A1, then one sample at each
higher level), numbered on a full-course plan: A1 1–25, A2 26–50, B1 51–75,
B2 76–150, C1 151–200, C2 201–300. The text has not been checked by a native
speaker.

This is private use only. You can build and run it for your own learning.
You cannot sell it or use it commercially. The full rules are in [LICENSE](LICENSE).

## Run

You need Rust 1.85 or newer (`edition = "2024"`). Install it from
https://rustup.rs if `cargo` is missing. Audio with pause and rewind needs
`mpv`. `ffplay` plays too, without those controls. The first voice download
also uses `curl` and `tar`.

```
git clone https://github.com/Matteo404404/lingua-onde
cd lingua-onde
cargo run --release
```

The first release build takes a few minutes. Later starts are the binary in
`target/release/lingua_onde`.

Progress is stored in `~/.local/share/lingua-onde/` (progress, your own
courses, downloaded voices). `LINGUA_ONDE_HOME` overrides that folder.

`--screenshot <file.png>` opens the window without focus, draws about 120
frames, saves a picture and exits. `--open home|courses|review|settings|study[:step]|editor[:lesson]`
opens one screen directly.

## The daily lesson

1. Blind listening: no text, repeat each sentence aloud in the pause.
2. Listen while reading the translation in your language.
3. Listen while reading the original text.
4. Read the dialogue with the translation (literal sense in brackets).
5. Notes.
6. Exercise 1: translate.
7. Exercise 2: fill in the missing words (checked, accents forgiven with a warning).
8. Read it again (translation hidden).
9. Last blind listening.
10. Read and listen to the two previous dialogues.
11. Second wave (when due): an old lesson, from your language back to the original, aloud.

Finished dialogues go into a spaced daily sentence review (Review, Daily
sentences). The second wave of lesson *n* comes due when you finish lesson
*n + 49* (Settings, "Second wave starts").

## Audio

- Your recordings: Courses, "Recordings…" links a folder. Files are matched
  to lessons by the last number in their name (`Leccion 12.mp3`, `L012.mp3`).
  A lesson's own `audio:` line wins. For your own course, "A lesson for each
  recording" creates text-less lessons so you can study with a printed book
  you already own. Do not put someone else's course recordings into this repo.
- Voices: Settings and voices, "Download" fetches the free offline Piper
  engine (about 25 MB, once) and one voice per language (about 60 MB). Latin
  is read by the Italian voice with Latin pronunciation rules.
- Playback uses `mpv` (pause, rewind 5 s, position). `ffplay` and `afplay`
  work without pause.

## Your material

My lessons: create or copy a course, then add lessons (type them, or paste the
dialogue and the translation line by line). Lessons are plain text files. The
format is in `content/FORMAT.md`.

## Where things are

- `src/main.rs` window and `--screenshot`. `src/app.rs` navigation, progress,
  courses, settings. `src/session.rs` the guided lesson, listening, review.
  `src/editor.rs` My lessons. `src/audio.rs` player, recordings, voices.
  `src/format.rs` lesson files. `src/store.rs` data folder, progress, review schedule.
- `content/*.txt` the built-in courses (`<lang>.txt` plus `<lang>-part2.txt`,
  merged per language; the list is `BUILTIN` in `src/store.rs`).
- `assets/NotoSans-Regular.ttf` is Noto Sans, under the SIL Open Font License
  in `assets/NotoSans-OFL.txt`. The app license does not replace that font license.

## Tests

`cargo test` checks the lesson format, every built-in course (complete in
Italian, English and Spanish, numbered inside its level's range), answer
checking and the review schedule. With network, mpv, and a muted mpv config
(`MPV_HOME` with `ao=null`):

```
cargo test -- --ignored
```

That also downloads voices, speaks, and runs the listen, pause, repeat sequence.

## License

Personal use only. See [LICENSE](LICENSE). The Noto Sans font stays under
`assets/NotoSans-OFL.txt`.
