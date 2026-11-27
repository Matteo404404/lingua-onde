# Lingua Onde

A desktop app for learning languages by assimilation, following the method of
the Assimil courses: a short dialogue a day, heard and repeated aloud,
understood through your own language, with figured pronunciation, notes, two
exercises, a revision lesson every seven, and from lesson 50 an active wave
that rebuilds earlier lessons from your language. How it works is in
[METHOD.md](METHOD.md).

Each language has three courses in a row: Foundation (first words to B2, 100
lessons), Perfectionnement (B2 to C1, 70 lessons) and Mastery (C1 to C2, 100
lessons). The built-in courses are original lessons written for this app
(English, Spanish, French, German, Dutch, Portuguese from Brazil, Mandarin
with pinyin, Latin with vowel lengths, and Vietnamese). They are being written
one pair of languages at a time: English for Italian speakers first
(Foundation is complete: 100 lessons with translations and notes in Italian,
an introduction, and an appendix with the grammar, a grammar index, the words
both ways and the expressions; Perfectionnement is complete too: 70 lessons,
an introduction, and an appendix with the grammar, a grammar index and the
expressions; Mastery is complete too: 100 lessons with open tasks, an
introduction, and an appendix with style and grammar, an index and the
expressions),
then Vietnamese for Spanish speakers (Foundation is complete: 100 lessons of
northern Vietnamese with translations and notes in Spanish, figured
pronunciation respelled for Spanish readers with a tone arrow after each
syllable, high tones drawn on the upper row and low ones below, as in
Assimil's *Le Vietnamien sans peine*, in lessons 1–14; an introduction and an
appendix; the interface is in Spanish too; Perfectionnement is complete too:
70 lessons, an introduction, and an appendix with the grammar, a grammar index
and the expressions; Mastery is complete too: 100 lessons with open tasks, an
introduction, and an appendix with style and grammar, an index and the
expressions), then Dutch for Italian speakers (Foundation is complete: 100
lessons of the Dutch of the Netherlands with translations and notes in
Italian, figured pronunciation respelled for Italian readers, an introduction
and an appendix with the grammar, a grammar index, the words and the
expressions; Perfectionnement is complete too: 70 lessons, an introduction,
and an appendix with the grammar, a grammar index and the expressions).
Spanish for Italian speakers has
its first week, lessons 1–7. The other languages have 13 sample
lessons (lessons 1–8, 26, 51 and 76 of Foundation, one lesson each of
Perfectionnement and Mastery), with translations and notes in Italian, English
and Spanish. The text has not been checked by a native
speaker.

This is private use only. You can build and run it for your own learning.
You cannot sell it or use it commercially. The full rules are in [LICENSE](LICENSE).

## Install

No programming tools needed. Lessons and fonts are inside the app. Voices
download from the app the first time you need them (that step uses `curl`
and `tar`, which are already on Linux, macOS and Windows 10).

Updates install themselves (from v1.4.0): when a new version is out, the app
says so at the top of the screen; "Update now" downloads it, puts it in place
of the old one and restarts. Your progress is kept. Settings, Updates shows
the version and checks on request. Versions before v1.4.0 have to be
downloaded by hand once.

The downloads below exist once a version is published on
[Releases](https://github.com/Matteo404404/lingua-onde/releases). Until that
first release, use [Build from source](#build-from-source).

**Linux** (x86_64, Ubuntu 22.04 or newer, or a distro from about 2022 onward).
Paste this in a terminal, then open "Lingua Onde" from the app menu:

```
curl -fsSL https://raw.githubusercontent.com/Matteo404404/lingua-onde/main/packaging/install.sh | sh
```

The same command updates the app. To remove it and keep your progress:
`curl -fsSL https://raw.githubusercontent.com/Matteo404404/lingua-onde/main/packaging/install.sh | sh -s -- --uninstall`.

Sound works out of the box (PipeWire, PulseAudio or ALSA). Installing `mpv`
(`sudo apt install mpv`, `sudo dnf install mpv` or `sudo pacman -S mpv`) adds
pause and rewind and plays your own MP3 recordings. The window uses the graphics libraries that come with
a normal desktop (Vulkan, Wayland or X11). The installer names anything that
is missing. On Ubuntu that is usually
`sudo apt install mesa-vulkan-drivers libxkbcommon0 libwayland-client0 libx11-6`.

**Windows** (10 or newer). Download `lingua-onde-windows-x86_64.exe` from
[Releases](https://github.com/Matteo404404/lingua-onde/releases/latest) and
double-click it. If Windows warns about an unknown app, choose "More info",
then "Run anyway". The voices play out of the box through Windows' own sound
player. To play your own MP3 recordings, download the Windows build from
https://mpv.io/installation/ and unzip it so `mpv.exe` and the other files
from that zip sit in the same folder as the app.

**macOS (macOS 11 or newer).** Download `lingua-onde-macos-arm64.zip` for
Apple silicon (M1 and later) or `lingua-onde-macos-x86_64.zip` for Intel Macs
(Apple menu, About This Mac: "Chip" means Apple silicon, "Processor" Intel) from
[Releases](https://github.com/Matteo404404/lingua-onde/releases/latest),
unzip it and move "Lingua Onde" to Applications. The first time, right-click
it and choose Open. If macOS still blocks it, open System Settings, Privacy
& Security, and choose Open Anyway. Sound works out of the box (`afplay`).
`brew install mpv` adds pause and rewind.

## Build from source

You need Rust 1.95 or newer (`edition = "2024"`). Install the current stable
toolchain from https://rustup.rs if `cargo` is missing. The voices play
through the system's own player; `mpv`, if installed, adds pause and rewind.
The first voice download uses `curl` and `tar`.

```
git clone https://github.com/Matteo404404/lingua-onde
cd lingua-onde
cargo run --release
```

The first release build takes a few minutes. Later starts are the binary in
`target/release/lingua_onde`. On Linux, `./packaging/install.sh` builds it
and adds it to the app launcher (`~/.local/bin/lingua-onde`, a menu entry and
an icon); `--uninstall` removes those three files and keeps your progress.

Push `main` first, then tag it. The installer script is read from `main`, and
the downloads are built from the tag:

```
git push origin main
git tag v1.0.0
git push origin v1.0.0
```

That runs `.github/workflows/release.yml`. The Linux build runs the tests
first. The release page gets the three downloads after all three builds
finish. "Run workflow" on the Actions tab builds the same files without
publishing them. "Check platforms" (also run by hand) downloads the voices
and speaks on Linux, Windows and macOS (Apple silicon and Intel): run it before the tag.

Progress is stored in `~/.local/share/lingua-onde/` (progress, your own
courses, downloaded voices). `LINGUA_ONDE_HOME` overrides that folder. Only languages whose Foundation is complete are offered; `LINGUA_ONDE_DRAFTS=1` shows the drafts too.

`--screenshot <file.png>` opens the window without focus, draws about 120
frames, saves a picture and exits. `--open home|courses|settings|guide|study[:step]|editor[:lesson]|review[:tab[:search]]|appendix[:page]` (0–6: introduction, pronunciation, lessons, grammar, index, words, expressions)
opens one screen directly (`guide` is the pronunciation guide), and `--size 950x520` sets the window size (a
narrow window shows the icon-only side bar). `--check <file>...` checks
lesson files against the built-in rules and prints what is missing.

## The daily lesson

The full method, course by course, is in [METHOD.md](METHOD.md). In the app,
every lesson goes:

1. Listen without the text, repeating each sentence aloud in the pause.
2. The same, reading the translation in your language.
3. The same, reading the original.
4. Read the dialogue with the translation (literal sense in brackets) and,
   the figured pronunciation for your language: the whole sentence in
   lessons 1–28, then the new words (the guide is under Notes,
   "Pronunciation guide").
5. Pronunciation observations, notes and, in some lessons, a cultural note.
6. Exercise 1: translate. Exercise 2: fill in the missing words (one dot per
   letter; checked, accents forgiven with a warning). Mastery adds open tasks.
7. Read it again with the translation hidden.
8. Listen once more without the text, repeating.
9. Read and listen to the two previous dialogues.
10. From lesson 50, the active wave: the lesson 49 places back (50 brings
    back 1), its dialogue and exercise 1 in your language only, to say (and
    optionally type) in the language you learn, then compare.

Every 7th lesson is a revision lesson: a revision dialogue whose lines name
the lessons they come from, studied the same way, with the grammar of the
six lessons before in order in place of the notes, and no exercises.
At the end of a lesson you can choose to go over it again the next day.
One lesson a day is the rhythm: once it is done, Home says you are done for
today and offers the next lesson only through a quieter link. Review holds the
active wave and Search (any sentence of the language); a spaced daily
sentence review, not part of the Assimil method, can be turned on in Settings. The active
wave of lesson *n* comes due when you finish lesson *n + 49* (Settings,
"Second wave starts"), and after the last lesson the rest come back one a day,
also alongside the next course of the language, until every lesson you studied
has been rebuilt.

The book (side menu) is laid out like an Assimil volume: before the lessons
the introduction (how to study, the two waves) and, in Foundation, the
pronunciation key; then the list of lessons; and at the end a grammar summary,
a grammar index, the words in both directions (Foundation only) and the
expressions, each with the lessons where they are met. A new learner is taken through the
introduction before lesson 1 (with a link to skip it). The course at the
top of the side bar opens a list of every course, to switch from any screen;
Courses shows each language's three courses side by side. Light and dark
follow the system, or Settings, Appearance fixes one. The synthetic voice
is slower in Foundation's first lessons and reaches the natural pace at lesson
50, as the books' recordings do (Settings, "Gradual pace"). On a day without
time, Home offers the last dialogue to listen to and repeat.

## Audio

- Your recordings: Courses, "Recordings…" links a folder. Files are matched
  to lessons by the last number in their name (`Leccion 12.mp3`, `L012.mp3`).
  A lesson's own `audio:` line wins. For your own course, "A lesson for each
  recording" creates text-less lessons so you can study with a printed book
  you already own. Do not put someone else's course recordings into this repo.
- Voices: Settings and voices, "Download" fetches the free offline Piper
  engine (about 25 MB, once) and one voice per language (about 60 MB). Latin
  is read by the Italian voice with Latin pronunciation rules.
- The voices need nothing installed: they play through the system's own
  player (Windows' sound player, `afplay` on macOS, PipeWire, PulseAudio or
  ALSA on Linux), and their speed, slower in Foundation's first lessons, is
  made by the voice engine itself. `mpv`, if present, is used first: it adds
  pause, rewind 5 s and position (Linux and macOS) and plays your own MP3
  recordings; `ffplay` also plays them, without pause.
- Before a release, the "Check platforms" workflow (run by hand in GitHub
  Actions) downloads the voices and speaks on Linux, Windows and macOS.

## Your material

My lessons: create or copy a course, then add lessons (type them, or paste the
dialogue and the translation line by line). Lessons are plain text files. The
format is in `content/FORMAT.md`.

## Where things are

- `src/main.rs` window, `--screenshot`, `--check`. `src/app.rs` navigation,
  home, courses, settings. `src/session.rs` the guided lesson, listening,
  review. `src/practice.rs` the active wave, Mastery's open tasks and the
  answer check. `src/editor.rs` My lessons. `src/audio.rs` player, recordings,
  voices. `src/format.rs` lesson files and the built-in lesson rules.
  `src/store.rs` data folder, progress, review schedule. `src/theme.rs`
  colours, fonts and shared widgets. `src/i18n.rs` the interface in the
  learner's language, from `content/ui/<base>.txt` (Italian so far).
- `content/<lang>/<course>-<level>.txt` the built-in courses
  (`es/foundation-a1.txt` … `es/mastery-c2.txt`; the list is `BUILTIN` in
  `src/store.rs`), and `content/<lang>/pronunciation.md` the figured
  pronunciation key where one exists, `content/<lang>/appendix-<stage>.<base>.txt`
  a course's appendix (`src/appendix.rs`, "The book" in the app). `content/FORMAT.md` is the file format,
  `METHOD.md` the method.
- `AGENTS.md` how the courses are written with AI agents, one week at a
  time; `work/` holds its briefs, course plans, writer instructions and
  tools (`work/tools/cursor_week.sh`, `merge_stage.py`, `first_seen.py`…).
- `progress.json` in the data folder: the learner's progress, keyed by course
  id and lesson number, so updates that add lessons, courses or languages
  keep it. Never rename a course id (`Stage::course_id`). The last good file
  is copied to `progress.backup.json`; a file that cannot be read is set
  aside as `progress.unreadable-<time>.json`, never overwritten. Files from
  v1.0.0 (one 300-lesson course per language) are converted on load.
- `packaging/` the launcher entry, icon and `install.sh`; `.github/workflows/release.yml`
  builds the downloads, `.github/workflows/check-platforms.yml` checks the
  voices and sound on each system.
- `assets/NotoSans-*.ttf` is Noto Sans (Regular, Medium, Bold), under the SIL
  Open Font License in `assets/NotoSans-OFL.txt`. The app license does not
  replace that font license.

## Tests

`cargo test` checks the lesson format, every built-in course (complete in
Italian, English and Spanish, numbered inside its level's range, pinyin and
Latin vowel lengths where due), answer checking, the review schedule, and
that every interface text has its Italian translation.
With network (and, for the listening test, mpv with a muted config:
`MPV_HOME` with `ao=null`):

```
cargo test -- --ignored
```

That also downloads voices, speaks, and runs the listen, pause, repeat sequence.

## License

Personal use only. See [LICENSE](LICENSE). The Noto Sans font stays under
`assets/NotoSans-OFL.txt`.
