# Lingua Onde

A desktop app for learning languages by assimilation: a short dialogue a day,
heard and repeated aloud, understood through your own language, with notes,
two exercises, review of earlier lessons and, later, an active "second wave".

The built-in courses are original lessons written for this app (Spanish,
French, German, Dutch, Portuguese from Brazil, Mandarin with pinyin, Latin
with vowel lengths, and Vietnamese). Translations and notes are in Italian,
English, and Spanish. Spanish, French, German, Portuguese, Mandarin, Latin
and Vietnamese have 13 lessons so far (8 at A1, then one sample at each
higher level). Dutch's six files contain the course name only. Numbering
follows a full-course plan: A1 1–25, A2 26–50,
B1 51–75, B2 76–150, C1 151–200, C2 201–300. The text has not been checked
by a native speaker.

This is private use only. You can build and run it for your own learning.
You cannot sell it or use it commercially. The full rules are in [LICENSE](LICENSE).

## Install

No programming tools needed. Lessons and fonts are inside the app. Voices
download from the app the first time you need them (that step uses `curl`
and `tar`, which are already on Linux, macOS and Windows 10).

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

For sound, install `mpv` (`sudo apt install mpv`, `sudo dnf install mpv` or
`sudo pacman -S mpv`). The window uses the graphics libraries that come with
a normal desktop (Vulkan, Wayland or X11). The installer names anything that
is missing. On Ubuntu that is usually
`sudo apt install mesa-vulkan-drivers libxkbcommon0 libwayland-client0 libx11-6`.

**Windows** (10 or newer). Download `lingua-onde-windows-x86_64.exe` from
[Releases](https://github.com/Matteo404404/lingua-onde/releases/latest) and
double-click it. If Windows warns about an unknown app, choose "More info",
then "Run anyway". For sound, download the Windows build from
https://mpv.io/installation/ and unzip it so `mpv.exe` and the other files
from that zip sit in the same folder as the app. Playback works. Pause and
rewind work on Linux and macOS.

**macOS (Apple silicon, macOS 11 or newer).** Download
`lingua-onde-macos-arm64.zip` from
[Releases](https://github.com/Matteo404404/lingua-onde/releases/latest),
unzip it and move "Lingua Onde" to Applications. The first time, right-click
it and choose Open. If macOS still blocks it, open System Settings, Privacy
& Security, and choose Open Anyway. Sound works out of the box (`afplay`).
`brew install mpv` adds pause and rewind.

## Build from source

You need Rust 1.95 or newer (`edition = "2024"`). Install the current stable
toolchain from https://rustup.rs if `cargo` is missing. Audio with pause and
rewind needs `mpv`. `ffplay` plays too, without those controls. The first
voice download also uses `curl` and `tar`.

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
publishing them, which is how you check Windows and macOS before the tag.

Progress is stored in `~/.local/share/lingua-onde/` (progress, your own
courses, downloaded voices). `LINGUA_ONDE_HOME` overrides that folder.

`--screenshot <file.png>` opens the window without focus, draws about 120
frames, saves a picture and exits. `--open home|courses|review|settings|study[:step]|editor[:lesson]`
opens one screen directly, and `--size 950x520` sets the window size (a
narrow window shows the icon-only side bar). `--check <file>...` checks
lesson files against the built-in rules and prints what is missing.

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
- Playback uses `mpv` (pause, rewind 5 s and position on Linux and macOS).
  On Windows, `mpv` plays and stops. `ffplay` and `afplay` play without pause.

## Your material

My lessons: create or copy a course, then add lessons (type them, or paste the
dialogue and the translation line by line). Lessons are plain text files. The
format is in `content/FORMAT.md`.

## Where things are

- `src/main.rs` window, `--screenshot`, `--check`. `src/app.rs` navigation,
  home, courses, settings. `src/session.rs` the guided lesson, listening,
  review. `src/editor.rs` My lessons. `src/audio.rs` player, recordings,
  voices. `src/format.rs` lesson files and the built-in lesson rules.
  `src/store.rs` data folder, progress, review schedule. `src/theme.rs`
  colours, fonts and shared widgets.
- `content/<lang>/a1.txt` … `c2.txt` the built-in courses, one file per
  level (the list is `BUILTIN` in `src/store.rs`).
- `packaging/` the launcher entry, icon and `install.sh`; `.github/workflows/release.yml`
  builds the downloads.
- `assets/NotoSans-*.ttf` is Noto Sans (Regular, Medium, Bold), under the SIL
  Open Font License in `assets/NotoSans-OFL.txt`. The app license does not
  replace that font license.

## Tests

`cargo test` checks the lesson format, every built-in course (complete in
Italian, English and Spanish, numbered inside its level's range, pinyin and
Latin vowel lengths where due), answer checking and the review schedule.
With network, mpv, and a muted mpv config
(`MPV_HOME` with `ao=null`):

```
cargo test -- --ignored
```

That also downloads voices, speaks, and runs the listen, pause, repeat sequence.

## License

Personal use only. See [LICENSE](LICENSE). The Noto Sans font stays under
`assets/NotoSans-OFL.txt`.
