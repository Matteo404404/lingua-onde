"""Extra content checks the format checker cannot do.

usage: python3 -I extra_checks.py <file>...
- Latin: the pron line without macrons equals the sentence; the sentence has no macrons.
- Chinese: pron has tone marks; the sentence has Han characters.
- Any: a lesson title repeated in the same course; a dialogue line repeated in another lesson.
"""
import re
import sys
import unicodedata
from collections import defaultdict

MACRON = str.maketrans("āēīōūȳĀĒĪŌŪȲ", "aeiouyAEIOUY")


def items(block):
    """(section, text, pron) for every sentence of a lesson block."""
    out, section, cur = [], None, None
    for line in block.split("\n"):
        t = line.strip()
        if re.fullmatch(r"(dialogue|ex1|ex2|notes(\.\w+)?):", t):
            section = t[:-1]
            continue
        if section in ("dialogue", "ex1", "ex2"):
            if line.startswith("- "):
                cur = [section, t[2:], ""]
                out.append(cur)
            elif t.startswith("pron:") and cur:
                cur[2] = t[5:].strip()
    return out


def letters(s):
    s = unicodedata.normalize("NFC", s).replace("[", "").replace("]", "")
    return re.sub(r"[^\w]", "", s).lower()


problems = []
titles = defaultdict(list)
lines = defaultdict(list)
for f in sys.argv[1:]:
    text = open(f).read()
    lang = re.search(r"(?m)^course:\s*(\w+)", text).group(1)
    for b in re.split(r"(?m)^(?===\s)", text)[1:]:
        num = b.split()[1]
        at = f"{f} {lang} {num}"
        m = re.search(r"(?m)^title:\s*(.+)$", b)
        if m:
            titles[(lang, m.group(1).strip())].append(num)
        for sec, s, pron in items(b):
            if sec == "dialogue" and len(s) > 25:
                lines[(lang, s)].append(num)
            if lang == "la":
                if s.translate(MACRON) != s:
                    problems.append(f"{at}: Latin text has macrons: {s}")
                if pron and letters(pron.translate(MACRON)) != letters(s):
                    problems.append(f"{at}: pron differs from text:\n    {s}\n    {pron}")
            if lang == "zh":
                if not re.search(r"[一-鿿]", s):
                    problems.append(f"{at}: no Chinese characters: {s}")
                if pron and not re.search(r"[āáǎàēéěèīíǐìōóǒòūúǔùǖǘǚǜ]", pron):
                    problems.append(f"{at}: pinyin without tone marks: {pron}")
for (lang, t), nums in titles.items():
    if len(nums) > 1:
        problems.append(f"{lang}: title '{t}' used by lessons {nums}")
for (lang, s), nums in lines.items():
    if len(set(nums)) > 1:
        problems.append(f"{lang}: line repeated in lessons {sorted(set(nums))}: {s[:70]}")
print("\n".join(problems))
print(f"{len(problems)} problems", file=sys.stderr)
