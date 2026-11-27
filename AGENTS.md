# Writing courses with AI agents

Lingua Onde's built-in courses (English for Italian speakers, Vietnamese for
Spanish speakers, more to come) are written by AI agents under one reviewing
agent, one week of lessons at a time. This file explains the loop and where
its pieces are, so you can run it with your own agents: a new language pair,
a new course, or fixes to an existing one.

## Roles

- **Planner and reviewer**: a strong model in a coding agent (here Claude
  Code). It reads the method (`METHOD.md`, `work/PATTERNS.md`), writes each
  course's plan, writes the writer's brief, reads and fixes every lesson the
  writer produces, merges and releases. It never hands review to the writer.
- **Writer**: a cheaper headless agent that writes one week (seven lessons)
  per run from the plan and the brief. Here it is the Cursor CLI on its Auto
  model, run by `work/tools/cursor_week.sh`; any agent that can read files,
  write files and run one command will do.

## The loop, one week at a time

1. **Plan** the course: one block or table row per lesson, with its kind, the
   scene and its ending, the grammar or style focus, the tasks. Examples:
   `work/vi/es/foundation.md`, `work/vi/es/perfectionnement.md`,
   `work/vi/es/mastery.md`, `work/nl/it/foundation.md`, `work/en/*.md`.
2. **Brief** the writer: file format, length, note style, the language's
   pitfalls, and the checks to run (`work/BRIEF.md` for every course, then
   one writer file per course: `work/vi/es/writer.md`, `perf-writer.md`,
   `mastery-writer.md`, `work/en/perf-writer.md`, `mastery-writer.md`).
3. **Write** one week:
   `STAGE=vi-perfectionnement work/tools/cursor_week.sh 8 14 auto`
   (stages: `vi-foundation`, `vi-perfectionnement`, `vi-mastery`, the
   default English `perfectionnement`, `mastery`; the third argument is the
   model). The writer edits only `work/<lang>[/<base>]/<stage>/NNN.txt`
   and may run only the checker. A log goes to `work/tools/logs/`.
4. **Check** every file: `target/release/lingua_onde --check FILE` (format,
   levels, tone marks of figured pronunciation, gaps). Then the reviewer
   reads every lesson in full: the checker cannot see a wrong fact, an
   unnatural line, a joke that does not work, a blank with two right
   answers, a reference to the wrong lesson.
   `python3 work/tools/first_seen.py work/vi/es/foundation 'word'` finds the
   first lesson whose dialogue has a word, for "(Fundamentos, lección N)"
   references. `work/tools/figure_vi.py` regenerates the Vietnamese figured
   pronunciation from the key; `figure_en.py` does the English one, and
   `figure_nl.py` the Dutch one (espeak plus a FIX table of words whose
   stress or sound espeak gets wrong: add to it rather than hand-editing
   lines).
5. **Teach the brief**: every error type found goes into the writer's brief
   as a rule ("Rules added after week N" in `work/en/mastery-writer.md` is
   the longest example). The next week's writer reads it.
6. **Merge**: `python3 -I work/tools/merge_stage.py . vi perfectionnement
   --from work/vi/es/perfectionnement --write` moves the approved lessons
   into `content/vi/perfectionnement-<level>.txt`. Then the next week starts.
   Weeks are strictly sequential: each lesson builds on the ones before.
7. **The book**: after the last lesson, `work/tools/appendix_draft.py`
   extracts the raw material (every word form with its first lesson, the
   revision sections) and a writer brief (`work/vi/es/appendix.md`,
   `work/en/perf-appendix.md`, `work/en/mastery-appendix.md`) turns it into
   `content/<lang>/appendix-<stage>.<base>.txt`. `cargo test --release`
   checks that every word and expression of the book is in the lesson it
   cites.
8. **Release**: `cargo test --release --locked`, commit, push a new tag; the
   workflow in `.github/workflows` builds Linux, Windows and macOS.

## Rules that keep it honest

- Everything is original. The method follows the Assimil books' structure
  (measured in `work/PATTERNS.md`), never their text; writers never open the
  books.
- Real places, people and history: only well-established facts, legends
  marked as legends.
- A native speaker has not yet reviewed the Vietnamese; reports of errors are
  welcome as issues.

## Not in the repository

Lesson drafts (`work/*/…/NNN.txt`, merged into `content/` once approved),
run logs, one-off fix prompts, old pre-method drafts.
