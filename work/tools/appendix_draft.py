"""Raw material for a course's appendix, from its finished lessons.

usage: python3 -I appendix_draft.py <repo> <lang> <stage> <out_dir>

Writes:
- forms.tsv: every word form of the dialogues, the lessons where it appears
  (first one first). A writer groups the forms into headwords (be: am, is,
  are...) and adds the translation; the app's test checks each entry against
  its first lesson.
- grammar.tsv: the numbered sections of the revision lessons' notes, with the
  lessons they name: the skeleton of the grammar index.
"""
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import progression as P  # noqa: E402

repo, lang, stage, out = Path(sys.argv[1]), sys.argv[2], sys.argv[3], Path(sys.argv[4])
out.mkdir(parents=True, exist_ok=True)
files = sorted((repo / "work" / lang / stage).glob("[0-9][0-9][0-9].txt"))

forms = defaultdict(list)
sections = []
for f in files:
    n = int(f.stem)
    text = f.read_text(encoding="utf-8")
    lesson = P.lessons(text)[n]
    for w in P.words(lesson["dialogue"]):
        if n not in forms[w]:
            forms[w].append(n)
    notes_key = re.search(r"(?m)^notes\.(it|es):", text)  # Italian or Spanish base
    if "kind: revision" in text and notes_key:
        notes = text[notes_key.end():]
        for m in re.finditer(r"(?m)^(\d+)\. (.+?)\(((?:lezion[ei]|lecci[oó]n(?:es)?) [^)]*)\)", notes):
            nums = [int(x) for x in re.findall(r"\d+", m.group(3))]
            sections.append((m.group(2).strip(" ."), n, nums))

with open(out / "forms.tsv", "w", encoding="utf-8") as o:
    for w, ls in sorted(forms.items(), key=lambda kv: (kv[1][0], kv[0])):
        o.write(f"{w}\t{' '.join(map(str, ls))}\n")
with open(out / "grammar.tsv", "w", encoding="utf-8") as o:
    for title, rev, nums in sections:
        o.write(f"{title}\t{rev}\t{' '.join(map(str, nums))}\n")
print(f"{len(forms)} forms, {len(sections)} grammar sections from {len(files)} lessons")
