"""First lesson whose DIALOGUE has a word or phrase (case-insensitive regex).

usage: python3 work/tools/first_seen.py DIR PATTERN...
Prints, per pattern, the first lesson number and the line, as used for the
"(Fundamentos, lección N)" back references.
"""
import re
import sys
from pathlib import Path


def dialogue_lines(text):
    """Target-language lines of the dialogue section (not translations, not exercises)."""
    out, inside = [], False
    for line in text.splitlines():
        if line.startswith("dialogue:"):
            inside = True
        elif re.match(r"^[a-z0-9.]+:", line):
            inside = False
        elif inside and line.startswith("- "):
            out.append(line[2:])
    return out


def main():
    d = Path(sys.argv[1])
    files = sorted(d.glob("[0-9][0-9][0-9].txt"))
    for pat in sys.argv[2:]:
        rx = re.compile(pat, re.I)
        hit = next(((f.stem, l) for f in files for l in dialogue_lines(f.read_text()) if rx.search(l)), None)
        print(f"{pat}: " + (f"{int(hit[0])}  {hit[1][:90]}" if hit else "none"))


if __name__ == "__main__":
    assert dialogue_lines("dialogue:\n- a\n  es: b\nex1:\n- c\n") == ["a"]
    main()
