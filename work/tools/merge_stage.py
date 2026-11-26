"""Move written lessons from work/<lang>/<course>/NNN.txt into
content/<lang>/<course>-<level>.txt, replacing a lesson with the same number.

usage: python3 -I merge_stage.py <repo> <lang> <course> [--from DIR] [--write]
Without --write it only prints what it would do. Each lesson file must hold
one lesson whose number matches its file name and whose level matches the
course plan.
"""
import re
import sys
from pathlib import Path

PLAN = {
    "foundation": [("A1", 1, 25), ("A2", 26, 50), ("B1", 51, 75), ("B2", 76, 100)],
    "perfectionnement": [("B2", 1, 45), ("C1", 46, 70)],
    "mastery": [("C1", 1, 25), ("C2", 26, 100)],
}


def split(text):
    """(header, {number: block})"""
    head, *rest = re.split(r"(?m)^(?===\s)", text)
    return head, {int(b.split()[1]): b.rstrip() + "\n" for b in rest}


def join(head, blocks):
    if not blocks:  # a level with no lessons yet keeps its header as it was
        return head.rstrip() + "\n"
    return head.rstrip() + "\n\n" + "\n".join(blocks[n] for n in sorted(blocks))


def main():
    repo, lang, course = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
    write = "--write" in sys.argv
    plan = PLAN[course]
    targets = {lvl: repo / "content" / lang / f"{course}-{lvl.lower()}.txt" for lvl, _, _ in plan}
    files = {lvl: split(p.read_text()) for lvl, p in targets.items()}
    moved = []
    # --from DIR: lessons written for one base live in work/<lang>/<base>/<course>.
    src = Path(sys.argv[sys.argv.index("--from") + 1]) if "--from" in sys.argv else repo / "work" / lang / course
    for f in sorted(src.glob("[0-9][0-9][0-9].txt")):
        _, got = split(f.read_text())
        n = int(f.stem)
        assert list(got) == [n], f"{f}: expected one lesson numbered {n}, found {list(got)}"
        lvl = next(l for l, a, b in plan if a <= n <= b)
        assert got[n].split()[2] == lvl, f"{f}: lesson {n} should be {lvl}"
        files[lvl][1][n] = got[n]
        moved.append(n)
    if write:
        for lvl, (head, blocks) in files.items():
            targets[lvl].write_text(join(head, blocks))
    print(("merged" if write else "would merge"), len(moved), "lessons:", moved)


if __name__ == "__main__":
    # self-check: split/join keeps the header and orders lessons by number
    h, b = split("course: x\n\n== 2 A1\nt: b\n\n== 1 A1\nt: a\n")
    assert list(b) == [2, 1] and join(h, b).index("== 1") < join(h, b).index("== 2")
    main()
