#!/bin/bash
# Writes one week of English Perfectionnement (or, with STAGE=mastery, Mastery)
# lessons with the Cursor agent CLI
# (the user's Cursor subscription), headless. Usage: cursor_week.sh FIRST LAST [MODEL]
# PROMPT_FILE=file runs that prompt instead. Output goes to work/tools/logs/week-FIRST-LAST.log; the reviewer checks every
# lesson afterwards, as for any writer.
set -euo pipefail
REPO=$(cd "$(dirname "$0")/../.." && pwd)  # the repository root
A=$1; B=$2; MODEL=${3:-grok-4.7-medium}
mkdir -p "$REPO/work/tools/logs"
printf -v FA "%03d" "$A"; printf -v FB "%03d" "$B"
PREV=$((A - 7)); printf -v FP "%03d" "$PREV"; printf -v FPE "%03d" $((A - 1))
if [ "${STAGE:-perfectionnement}" = vi-foundation ]; then
PROMPT="You write lessons $A–$B of the Vietnamese Foundation course for Spanish speakers (northern Vietnamese), in the repository $REPO.

Your full instructions are in work/vi/es/writer.md: read it first and follow every rule. The plan is work/vi/es/foundation.md: your lessons are its blocks \"### $A\" to \"### $B\". Lessons already written are in work/vi/es/foundation/: read all of them before writing, to reuse their words and match their format.

Write work/vi/es/foundation/$FA.txt to $FB.txt, one file per lesson. Every 7th lesson is the revision lesson of the six before it. Its dialogue is a NEW little scene between two people, as in 035.txt and 042.txt, never a montage of the week's lines. No Vietnamese words inside Spanish sentences. Nobody says vâng, dạ or ạ to someone younger.

Strict limits: create or edit only those lesson files. Never touch src/, content/, work/tools/ or any other file. Never run git or cargo. The only commands you may run besides reading files and grep are: target/release/lingua_onde --check work/vi/es/foundation/NNN.txt and python3 work/tools/figure_vi.py work/vi/es/foundation/NNN.txt

Work strictly in order, one lesson at a time. Run the checker on each file and fix everything it reports. At the end report briefly as writer.md says."
elif [ "${STAGE:-perfectionnement}" = vi-perfectionnement ]; then
PROMPT="You write lessons $A–$B of the Vietnamese Perfectionnement course for Spanish speakers (northern Vietnamese, B2 to C1), in the repository $REPO.

Your full instructions are in work/vi/es/perf-writer.md: read it first and follow every rule. The plan is work/vi/es/perfectionnement.md: your lessons are its sections \"### $A\" to \"### $B\". Lessons already written and approved are in work/vi/es/perfectionnement/: read all of the previous week ($FP.txt to $FPE.txt, if they exist) to match their voice, length and format exactly.

Write work/vi/es/perfectionnement/$FA.txt to $FB.txt, one file per lesson. The last lesson of the week is the revision lesson of the six before it. Word lessons end their notes with the expression list. Exercise 1 sentences are new, never copies of text lines. No Vietnamese words used as Spanish words inside Spanish sentences. Nobody says vâng, dạ, ạ or thưa to someone younger.

Strict limits: create or edit only those lesson files. Never touch src/, content/, work/tools/ or any other file. Never run git or cargo. The only command you may run besides reading files and grep is: target/release/lingua_onde --check work/vi/es/perfectionnement/NNN.txt

Work strictly in order, one lesson at a time. Run the checker on each file, reread each lesson against the checks in perf-writer.md, and verify every Fundamentos reference and every \"(lección N, nota M)\" reference by opening the cited text. At the end report briefly as perf-writer.md says."
elif [ "${STAGE:-perfectionnement}" = vi-mastery ]; then
PROMPT="You write lessons $A–$B of the Vietnamese Mastery course for Spanish speakers (northern Vietnamese, C1 to C2), in the repository $REPO.

Your full instructions are in work/vi/es/mastery-writer.md: read it first and follow every rule. The plan is work/vi/es/mastery.md: your lessons are the rows $A to $B of its tables. Lessons already written and approved are in work/vi/es/mastery/ (read all of the previous week, $FP.txt to $FPE.txt, if they exist) to match their voice, length and format exactly.

Write work/vi/es/mastery/$FA.txt to $FB.txt, one file per lesson. Every 7th lesson is the revision lesson of the six before it. Exercise 1 sentences are new, never copies of text lines. No Vietnamese words used as Spanish words inside Spanish sentences. Nobody says vâng, dạ, ạ or thưa to someone younger. Notes never mention the plan.

Strict limits: create or edit only those lesson files. Never touch src/, content/, work/tools/ or any other file. Never run git or cargo. The only commands you may run besides reading files and grep are: target/release/lingua_onde --check work/vi/es/mastery/NNN.txt and python3 work/tools/first_seen.py DIR 'pattern'

Work strictly in order, one lesson at a time. Run the checker on each file, reread each lesson against the checks in mastery-writer.md, and verify every reference by opening the cited text. At the end report briefly as mastery-writer.md says."
elif [ "${STAGE:-perfectionnement}" = mastery ]; then
PROMPT="You write lessons $A–$B of the English (British) Mastery course for Italian speakers (C1 to C2), in the repository $REPO.

Your full instructions are in work/en/mastery-writer.md: read it first and follow every rule. The plan is work/en/mastery.md: your lessons are the rows $A to $B of its tables. Lessons already written and approved are in work/en/mastery/ (read all of the previous week, $FP.txt to $FPE.txt, if they exist) to match their voice, length and format exactly.

Write work/en/mastery/$FA.txt to $FB.txt, one file per lesson. Every 7th lesson is the revision lesson of the six before it.

Strict limits: create or edit only those lesson files. Never touch src/, content/, work/tools/ or any other file. Never run git or cargo. The only command you may run besides reading files and grep is: target/release/lingua_onde --check work/en/mastery/NNN.txt

Work strictly in order, one lesson at a time. Run the checker on each file, reread each lesson against the checks in mastery-writer.md, and verify every reference by opening the cited text. At the end report briefly as mastery-writer.md says."
else
PROMPT="You write lessons $A–$B of the English (British) Perfectionnement course for Italian speakers, in the repository $REPO.

Your full instructions are in work/en/perf-writer.md: read it first and follow every rule (the newest rules are near the end of its rules section). Lessons 1–$((A - 1)) are written and approved in work/en/perfectionnement/ (001.txt to $FPE.txt): read several of them, especially $FP.txt to $FPE.txt, to match their voice, length and format exactly.

Write work/en/perfectionnement/$FA.txt to $FB.txt, each to its section in work/en/perfectionnement.md (sections \"### $A\" to \"### $B\"). The last lesson of the week is the revision lesson of the six before it. Word lessons end their notes with the expression list, as in 040.txt and 045.txt.

Strict limits: create or edit only those lesson files. Never touch src/, content/, work/tools/ or any other file. Never run git or cargo. The only command you may run besides reading files is: target/release/lingua_onde --check work/en/perfectionnement/NNN.txt

Work strictly in order, one lesson at a time. Run the checker on each file, reread each lesson against the checks in perf-writer.md, and verify every Fondamenti reference and every \"(lezione N, nota M)\" reference by opening the cited text. At the end report briefly as perf-writer.md says."
fi
# Another task (e.g. the appendix) through the same runner: PROMPT_FILE=brief.md.
LOG="$REPO/work/tools/logs/${STAGE:-perfectionnement}-week-$A-$B.log"
if [ -n "${PROMPT_FILE:-}" ]; then PROMPT=$(cat "$PROMPT_FILE"); LOG="$REPO/work/tools/logs/$(basename "$PROMPT_FILE" .md).log"; fi
cd "$REPO"
run_cursor() { agent -p --model "$MODEL" --force --trust --workspace "$REPO" "$PROMPT" >>"$LOG" 2>&1; }
# Codex (the user's ChatGPT plan) only when Cursor is out of quota and the only
# fix would be waiting for the renewal; a transient failure gets one retry.
LIMIT='out of usage|usage limit|quota|rate limit|out of (requests|credits)|limit reached|exceeded|upgrade your plan|resets? (on|at|in)'
: >"$LOG"
if run_cursor; then exit 0; fi
if ! grep -Eiq "$LIMIT" "$LOG"; then
    sleep 60
    echo "--- cursor retry" >>"$LOG"
    if run_cursor; then exit 0; fi
fi
if grep -Eiq "$LIMIT" "$LOG"; then
    echo "--- cursor out of quota: falling back to codex" >>"$LOG"
    codex exec -C "$REPO" -s workspace-write "$PROMPT" >>"$LOG" 2>&1
else
    echo "--- cursor failed (not a quota limit): see above" >>"$LOG"
    exit 1
fi
