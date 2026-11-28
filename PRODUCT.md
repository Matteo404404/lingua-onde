# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(The app is a cross-platform desktop program, Rust eframe/egui 0.36, drawn
by the app itself on Linux, Windows and macOS. No native OS design language
applies; the closest record value is `web`.)

## Users

Adults learning a language on their own at home, about 20–30 minutes a day,
through their own language (Italian, English or Spanish interface). They
want a calm, serious, beautiful tool, closer to a good book than to a game.
(Confirmed by the user, 2026-11-28.)

## Product Purpose

One short dialogue a day, heard in one go and repeated aloud, understood
through the learner's language with figured pronunciation, notes beside the
dialogue and two exercises; a revision lesson every seven; from lesson 50 a
"second wave" that brings back earlier lessons to rebuild from the learner's
language. Three courses per language: Foundation (to B2, 100 lessons),
Perfectionnement (B2→C1, 70), Mastery (C1→C2, 100). Success is a daily
habit that carries a learner from first words to fluent reading and
speaking.

## Positioning

A slow, book-like daily method with original dialogues and figured
pronunciation for the learner's own language, offline voices, an offline
pronunciation check after the last listening, and a CEFR level test that
starts the learner where they are. Free, no account, no streak pressure
beyond a quiet study calendar.

## Operating Context

Desktop window (default 1280×820, minimum 900×600), light and dark themes
following the system or the learner's setting. Daily loop: home → today's
lesson (10 steps: listen, read, notes, exercises, final listening with the
pronunciation check) → review cards. Also: course picker, the book
(appendix), the learner's own lessons editor, settings and voices.

## Capabilities and Constraints

- UI strings live in content/ui/it.txt and content/ui/es.txt (English in code).
- Fonts are bundled (Noto Sans); Chinese uses a system CJK font.
- Self-updater, installers (Windows Inno Setup, macOS dmg, Linux script).
- Progress must survive updates: never rename course ids or the installer
  AppId; keep shipping the lingua-onde-* updater files in every release.

## Brand Commitments

- Name: **Marea** (from 2026-11-28; previously "Lingua Onde"). "Tide" in
  Italian and Spanish, maré in Portuguese: one lesson a day, and the second
  wave brings lessons back like a tide.
- The rename is complete (2026-11-28, version 1.11): repository
  Matteo404404/marea, binary and data folders marea / Marea. Copies from
  before update through old-named release files (lingua-onde-*) and move
  their data folder on first start (src/store.rs). The installer AppId is
  kept so Windows upgrades the old install in place.
- The course does not lean on the name of any commercial method.

## Evidence on Hand

Real content: the courses in content/ (Italian→English, Spanish→Vietnamese,
Italian→Dutch, English→Spanish complete; Italian→Portuguese in progress).
No testimonials, user numbers or press: none may be invented. The text has
not been checked by native speakers.

## Product Principles

1. The lesson is the product: the dialogue and the learner's voice come
   first, chrome recedes.
2. Calm daily rhythm over pressure: progress is shown, never nagged.
3. One screen, one task: each step says what to do now.
4. Nothing lost: progress and courses survive every update.

## Accessibility & Inclusion

Readable text sizes for long reading, both themes with sufficient
contrast, every action reachable by keyboard where egui allows.
