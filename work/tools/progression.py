"""How a course grows, lesson by lesson: the words each lesson brings for the
first time, and exercise words the learner has not met yet.

usage: python3 -I progression.py <repo> <lang> <course> [from] [to]
Lessons come from content/<lang>/<course>-*.txt, replaced by any written
lesson in work/<lang>/<course>/NNN.txt. A word counts as met once it is in
an earlier dialogue or in the same lesson's dialogue; names, numbers and
words that differ only by case count as the same.
"""
import re
import sys
from pathlib import Path

WORD = re.compile(r"[^\W\d_]+(?:['’][^\W\d_]+)*", re.UNICODE)


def lessons(text):
    """{number: {"dialogue": [...], "ex": [...]}} from a course file."""
    out = {}
    for block in re.split(r"(?m)^(?===\s)", text)[1:]:
        n = int(block.split()[1])
        section, d, ex = None, [], []
        for line in block.split("\n"):
            m = re.match(r"^(dialogue|ex1|ex2|tasks|(?:sounds|notes|culture|aside)\.[a-z]+):\s*$", line)
            if m:
                section = m.group(1)
            elif line.startswith("- ") and section == "dialogue":
                d.append(line[2:])
            elif line.startswith("- ") and section in ("ex1", "ex2"):
                ex.append(line[2:].replace("[", "").replace("]", ""))
        out[n] = {"dialogue": d, "ex": ex}
    return out


def words(lines):
    return {w.lower().replace("’", "'") for l in lines for w in WORD.findall(l)}


def main():
    repo, lang, course = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
    first = int(sys.argv[4]) if len(sys.argv) > 4 else 1
    last = int(sys.argv[5]) if len(sys.argv) > 5 else 10**6
    all_ = {}
    for f in sorted((repo / "content" / lang).glob(f"{course}-*.txt")):
        all_.update(lessons(f.read_text(encoding="utf-8")))
    for f in sorted((repo / "work" / lang / course).glob("[0-9][0-9][0-9].txt")):
        all_.update(lessons(f.read_text(encoding="utf-8")))
    met = set()
    for n in sorted(all_):
        here = words(all_[n]["dialogue"])
        new = sorted(here - met)
        met |= here
        unknown = sorted(words(all_[n]["ex"]) - met)
        if first <= n <= last:
            print(f"lesson {n}: {len(all_[n]['dialogue'])} lines, {len(new)} new words: {' '.join(new)}")
            if unknown:
                print(f"  exercise words not met yet: {' '.join(unknown)}")


if __name__ == "__main__":
    # self-check: words met in an earlier dialogue are not new; exercise words are checked
    t = "== 1 A1\ndialogue:\n- Hello, Anna.\n\nex1:\n- Hello, Bob.\n\n== 2 A1\ndialogue:\n- Hello again.\n"
    ls = lessons(t)
    assert words(ls[2]["dialogue"]) - words(ls[1]["dialogue"]) == {"again"}
    assert words(ls[1]["ex"]) - words(ls[1]["dialogue"]) == {"bob"}
    main()
