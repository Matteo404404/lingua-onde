"""Figured pronunciation of British English for Italian readers.

usage: python3 -I figure_en.py <lesson file>... [--write]
       python3 -I figure_en.py --say "<English sentence>"

Fills the missing `pron.it:` lines of Foundation lessons, following
content/en/pronunciation.md: in lessons 1–28 the whole line (and the
numeral, in every lesson), from lesson 29 only the words new in that lesson, the rest left
out ("…"), as the Assimil books do. Existing lines are kept,
so a hand correction survives. Without --write it prints what it would add.
Words whose sound depends on their meaning (read, live, close...) are listed
for a human check.

The sounds come from espeak-ng (its RP British voice), the copy bundled with the
app's voices in ~/.local/share/lingua-onde/voices/piper.
"""
import re
import shutil
import stat
import subprocess
import sys
from functools import lru_cache
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import progression  # noqa: E402

FULL = 28  # FULL_FIGURED in src/format.rs

PIPER = Path.home() / ".local/share/lingua-onde/voices/piper"
CACHE = Path.home() / ".cache/lingua-onde-espeak/espeak-ng"

# espeak-ng's British voice gets these wrong or old-fashioned.
FIX = {
    "one": "wˈʌn",
    "once": "wˈʌns",
    "someone": "sˈʌmwʌn",
    "anyone": "ˈɛniwʌn",
    "everyone": "ˈɛvɹiwʌn",
    "giulia": "dʒˈuːliə",
    "usually": "jˈuːʒuəli",
    "perfect": "pˈɜːfɪkt",
    "husband": "hˈʌzbənd",
    "husband's": "hˈʌzbəndz",
    "café": "kˈafeɪ",
    "cafe": "kˈafeɪ",
    "thirteen": "θɜːtˈiːn",
    "fourteen": "fɔːtˈiːn",
    "fifteen": "fɪftˈiːn",
    "sixteen": "sɪkstˈiːn",
    "seventeen": "sɛvəntˈiːn",
    "eighteen": "eɪtˈiːn",
    "nineteen": "naɪntˈiːn",
    "recipe": "ɹˈɛsəpi",
    "bologna": "bəlˈəʊnjə",
    "welcome": "wˈɛlkəm",
    "nobody": "nˈəʊbədi",
    "quiet": "kwˈaɪət",
    "they're": "ðˈeə",
    "nobody's": "nˈəʊbədiz",
    "baths": "bˈɑːðz",
    "ones": "wˈʌnz",
    "pasta": "pˈastə",
    "innings": "ˈɪnɪŋz",
    "edinburgh": "ˈɛdɪnbɹə",
    "pants": "pˈants",
    "juliet": "dʒˈuːliət",
    "craic": "kɹˈak",
    "hundred": "hˈʌndɹəd",
    "someone's": "sˈʌmwʌnz",
}

# Weak forms of small words inside a sentence: (before a consonant, before a
# vowel). At the end of a sentence or before punctuation the strong form stays.
WEAK = {
    "a": ("ə", "ə"),
    "an": ("ən", "ən"),
    "the": ("ðə", "ði"),
    "of": ("əv", "əv"),
    "to": ("tə", "tu"),
    "and": ("ənd", "ənd"),
    "for": ("fə", "fəɹ"),
    "from": ("fɹəm", "fɹəm"),
    "at": ("ət", "ət"),
    "as": ("əz", "əz"),
    "can": ("kən", "kən"),
    "was": ("wəz", "wəz"),
    "were": ("wə", "wəɹ"),
    "are": ("ə", "əɹ"),
    "them": ("ðəm", "ðəm"),
    "her": ("hə", "həɹ"),
    "than": ("ðən", "ðən"),
    "you": ("ju", "ju"),
    "does": ("dəz", "dəz"),
}

# A homograph settled by the word after it.
BEFORE = {
    ("excuse", "me"): "ɪkskjˈuːz", ("excuse", "us"): "ɪkskjˈuːz",
    ("close", "the"): "klˈəʊz", ("close", "your"): "klˈəʊz", ("close", "it"): "klˈəʊz",
    ("close", "my"): "klˈəʊz", ("close", "at"): "klˈəʊz",
    ("used", "to"): "jˈuːst", ("use", "to"): "jˈuːs",
}

# Verbs that open a question keep their full form there ("Are you...?").
STRONG_FIRST = {"are", "was", "were"}

# Words whose sound depends on what they mean here.
HOMOGRAPHS = {
    "read", "live", "lives", "lead", "wind", "close", "closes", "use", "uses", "used",
    "record", "present", "tear", "tears", "wound", "minute", "object", "bow", "row",
    "content", "desert", "produce", "refuse", "project", "increase", "permit", "subject",
    "wind", "invalid", "polish", "house", "houses", "excuse", "separate", "estimate",
    "graduate", "moderate", "alternate", "associate", "advocate", "perfect", "conduct",
}

# The signs follow Assimil's Italian editions: Italian spelling, the stressed
# syllable in bold (*...*), ‣ after a long vowel and ë for the weak e
# (Il Francese senza sforzo), è for the open e, θ for th (Lo Spagnolo),
# ž for the s of usually, sc/sci for sh, y for the i of yes.
VOWELS = [  # IPA, figured
    ("aɪə", "aië"), ("aʊə", "auë"),
    ("iː", "i‣"), ("uː", "u‣"), ("ɑː", "a‣"), ("ɔː", "o‣"), ("ɜː", "ë‣"),
    ("eɪ", "ei"), ("aɪ", "ai"), ("ɔɪ", "oi"), ("aʊ", "au"), ("əʊ", "ou"),
    ("ɪə", "ië"), ("iə", "ië"), ("eə", "eë"), ("ɛə", "eë"), ("ʊə", "uë"),
    ("ɪ", "i"), ("i", "i"), ("ᵻ", "i"), ("ɛ", "e"), ("e", "e"),
    ("a", "è"), ("æ", "è"), ("ɐ", "ë"), ("ʌ", "a"), ("ɒ", "o"), ("ɔ", "o"),
    ("ʊ", "u"), ("u", "u"), ("ə", "ë"), ("o", "o"),
]
CONS = {
    "p": "p", "b": "b", "t": "t", "d": "d", "k": "k", "f": "f", "v": "v", "m": "m", "n": "n",
    "l": "l", "h": "h", "s": "s", "z": "z", "θ": "θ", "ð": "dh", "ʒ": "ž",
    "ɹ": "r", "r": "r", "j": "y", "w": "w", "x": "kh", "ŋ": "ng",
}
# Onsets a syllable may start with (IPA), for the bold stressed syllable.
ONSETS = {
    "pl", "bl", "kl", "ɡl", "fl", "sl", "pɹ", "bɹ", "tɹ", "dɹ", "kɹ", "ɡɹ", "fɹ", "θɹ", "ʃɹ",
    "tw", "dw", "kw", "ɡw", "sw", "θw", "sp", "st", "sk", "sm", "sn", "sf",
    "pj", "bj", "tj", "dj", "kj", "ɡj", "mj", "nj", "fj", "vj", "hj", "lj", "sj",
    "spl", "spɹ", "stɹ", "skɹ", "skw", "spj", "stj", "skj",
}


def espeak():
    if not CACHE.exists():
        CACHE.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(PIPER / "espeak-ng", CACHE)
        CACHE.chmod(CACHE.stat().st_mode | stat.S_IXUSR)
    return str(CACHE)


@lru_cache(maxsize=None)
def ipa(word):
    if word.lower() in FIX:
        return FIX[word.lower()]
    if "-" in word.strip("-"):
        return "-".join(ipa(p) for p in word.split("-") if p)
    r = subprocess.run(
        [espeak(), f"--path={PIPER}", "-v", "en-gb-x-rp", "-q", "--ipa"],
        input=word, capture_output=True, text=True, env={"LD_LIBRARY_PATH": str(PIPER)},
    )
    return "".join(r.stdout.split()).replace(chr(0x200D), "")


@lru_cache(maxsize=None)
def in_context(prev, word):
    """The sound of `word` said after `prev`, else said alone."""
    r = subprocess.run(
        [espeak(), f"--path={PIPER}", "-v", "en-gb-x-rp", "-q", "--ipa"],
        input=f"{prev} {word}", capture_output=True, text=True, env={"LD_LIBRARY_PATH": str(PIPER)},
    )
    parts = r.stdout.replace(chr(0x200D), "").split()
    return parts[-1] if len(parts) == 2 else ipa(word)


def tokens(s):
    """IPA string -> [(kind, ipa, stressed)] with kind 'v' or 'c'."""
    # A syllabic consonant (wasn't: wˈɒzn + U+0329 + t) is written with ë before it, as in isn't.
    s = re.sub("(.)" + chr(0x0329), r"ə\1", s)
    out, stress, i = [], False, 0
    while i < len(s):
        ch = s[i]
        if ch == "ˈ":
            stress = True
            i += 1
            continue
        if ch in "ˌʔ":
            i += 1
            continue
        if s.startswith("tʃ", i) or s.startswith("dʒ", i):
            out.append(("c", s[i : i + 2], False))
            i += 2
            continue
        for v, _ in VOWELS:
            if s.startswith(v, i):
                out.append(("v", v, stress))
                stress = False
                i += len(v)
                break
        else:
            if ch in ("ɡ", "g"):
                out.append(("c", "ɡ", False))
            elif ch in CONS or ch == "ʃ":
                out.append(("c", ch, False))
            else:
                raise ValueError(f"unknown sound {ch!r} in {s!r}")
            i += 1
    return out


def syllable_of(toks, vi):
    """(first, last) token index of the syllable whose vowel is toks[vi]."""
    vowels = [n for n, t in enumerate(toks) if t[0] == "v"]
    k = vowels.index(vi)

    def onset_start(v_at, prev_v):
        cons = [toks[n][1] for n in range(prev_v + 1, v_at)]
        for size in range(min(3, len(cons)), 0, -1):
            tail = cons[-size:]
            if size == 1 and tail[0] != "ŋ" or "".join(tail) in ONSETS:
                return v_at - size
        return v_at

    first = onset_start(vi, vowels[k - 1]) if k else 0
    last = onset_start(vowels[k + 1], vi) - 1 if k + 1 < len(vowels) else len(toks) - 1
    return first, last


def spell(ipa_word):
    """One word's IPA -> figured spelling, by content/en/pronunciation.md."""
    if "-" in ipa_word:
        return "-".join(spell(p) for p in ipa_word.split("-"))
    toks = tokens(ipa_word)
    figured = dict(VOWELS)
    vowels = [n for n, t in enumerate(toks) if t[0] == "v"]
    letters = []
    for n, (kind, v, stressed) in enumerate(toks):
        if kind == "v":
            letters.append("a" if v == "ɐ" and stressed else figured[v])
            continue
        nxt = toks[n + 1] if n + 1 < len(toks) else None
        vowel_next = nxt is not None and nxt[0] == "v"
        nf = figured[nxt[1]] if vowel_next else ""
        front = vowel_next and nf[0] in "ieèë" and not (nxt[1] == "ɐ" and nxt[2])
        vowel_before = n > 0 and toks[n - 1][0] == "v"
        if v == "tʃ":
            letters.append("c" if front else "ci" if vowel_next else "c'")
        elif v == "dʒ":
            letters.append("g" if front else "gi" if vowel_next else "g'")
        elif v == "ʃ":
            letters.append("sc" if front else "sci" if vowel_next else "sc'")
        elif v == "ɡ":
            letters.append("gh" if front else "g")
        elif v == "k":
            letters.append("k")
        elif v == "ŋ":
            letters.append("n" if nxt and nxt[1] in ("k", "ɡ") else "ng")
        elif v == "s" and vowel_before and vowel_next:
            letters.append("ss")  # as in Lo Spagnolo: a single s would sound like rosa
        else:
            letters.append(CONS[v])
    stressed = [n for n in vowels if toks[n][2]]
    if len(vowels) > 1 and stressed:
        first, last = syllable_of(toks, stressed[0])
        letters[first] = "*" + letters[first]
        letters[last] = letters[last] + "*"
    return "".join(letters)


WORD = re.compile(r"[^\W\d_](?:[^\W\d_]|['’-])*|[.,;:?!…]")
VOWEL_START = re.compile(r"^[ˈˌ]?(?:[aeiouæɑɒɔəɛɪʊʌɜɐᵻ])")


def figure(sentence):
    """A sentence -> (figured pronunciation, homographs met)."""
    items = WORD.findall(sentence.replace("’", "'"))
    out, seen = [], []
    for n, w in enumerate(items):
        if not w[0].isalpha():
            out.append(w)
            continue
        lw = w.lower()
        nxt = items[n + 1] if n + 1 < len(items) else "."
        nxt_ipa = ipa(nxt) if nxt[0].isalpha() else ""
        before_vowel = bool(VOWEL_START.match(nxt_ipa))
        first = n == 0 or not items[n - 1][0].isalpha()
        if lw in WEAK and nxt[0].isalpha() and not (first and lw in STRONG_FIRST):
            sound = WEAK[lw][1 if before_vowel else 0]
        elif (lw, nxt.lower()) in BEFORE:
            sound = BEFORE[(lw, nxt.lower())]
        elif lw in HOMOGRAPHS and n and items[n - 1][0].isalpha():
            # The word before often settles it ("I live" against "a live show").
            sound = in_context(items[n - 1], w)
        else:
            sound = ipa(w)
        word = spell(sound)
        # Linking r: a written final r is heard before a vowel.
        if before_vowel and re.search(r"(r|re)$", lw) and not word.endswith("r") and lw not in WEAK:
            word += "r"
        if lw in HOMOGRAPHS:
            seen.append(w)
        out.append(word)
    text = ""
    for t in out:
        text += t if t in ".,;:?!…" or not text else " " + t
    return text, seen


def met_before(path, n):
    """Words of the dialogues of the course's lessons before lesson n."""
    p = Path(path).resolve()
    repo = next((d for d in p.parents if (d / "Cargo.toml").exists()), None)
    found = {}
    if repo:
        for f in sorted((repo / "content" / "en").glob("foundation-*.txt")):
            found.update(progression.lessons(f.read_text(encoding="utf-8")))
    for f in sorted(p.parent.glob("[0-9][0-9][0-9].txt")):
        found.update(progression.lessons(f.read_text(encoding="utf-8")))
    return progression.words(d for k, v in found.items() if k < n for d in v["dialogue"])


def new_words_only(sentence, met):
    """The figured spelling of the words not met yet, the rest left out."""
    full, _ = figure(sentence)
    items = [w for w in WORD.findall(sentence.replace("’", "'"))]
    spelled = [t for t in full.replace(".", " ").replace(",", " ").replace("?", " ").replace("!", " ")
               .replace(";", " ").replace(":", " ").replace("…", " ").split()]
    words = [w for w in items if w[0].isalpha()]
    if len(words) != len(spelled):
        return None
    out, gap = [], False
    for w, s in zip(words, spelled):
        # A hyphenated number (forty-two) is met when its parts are.
        parts = w.lower().split("-")
        if w.lower() in met or (len(parts) > 1 and all(p in met for p in parts)):
            gap = True
            continue
        if gap or not out and w is not words[0]:
            out.append("…")
        out.append(s)
        met.add(w.lower())
        met.update(parts)
        gap = False
    if not out:
        return None
    if gap:
        out.append("…")
    return " ".join(out)


def fill(path, write):
    lines = Path(path).read_text().split("\n")
    out, level, lesson, added, flags, revision = [], None, 0, 0, [], False
    i = 0
    while i < len(lines):
        line = lines[i]
        out.append(line)
        m = re.match(r"^== (\d+) (A1|A2|B1|B2|C1|C2)", line)
        if m:
            level, lesson = m.group(2), int(m.group(1))
        early = level in ("A1", "A2", "B1", "B2")
        full = lesson <= FULL
        if m:
            met = met_before(path, lesson) if not full else set()
            revision = False
        # After lesson 28 a revision dialogue has no new word and no pron line.
        if line.strip() == "kind: revision":
            revision = True
        # The numeral is figured in full in every lesson, as in the books' headers.
        if early and line.startswith("numeral: ") and not any(l.startswith("numeral.pron.it:") for l in lines[i + 1 : i + 4]):
            text, _ = figure(line[len("numeral: "):])
            out.append(f"numeral.pron.it: {text}")
            added += 1
        if early and line.startswith("- ") and in_section(lines, i, "dialogue:") and not (revision and not full):
            j = i + 1
            block = []
            while j < len(lines) and lines[j].startswith("  "):
                block.append(lines[j])
                j += 1
            if not any(b.strip().startswith("pron.it:") for b in block):
                text, seen = figure(line[2:])
                if not full:
                    text = new_words_only(line[2:], met)
                if text is None:
                    out.extend(block)
                    i = j
                    continue
                at = next((k for k, b in enumerate(block) if b.strip().startswith("it:")), len(block) - 1)
                block.insert(at + 1, f"  pron.it: {text}")
                added += 1
                if seen:
                    flags.append(f"lesson {lesson}: check {', '.join(seen)}: {line[2:]}")
            out.extend(block)
            i = j
            continue
        i += 1
    if write:
        Path(path).write_text("\n".join(out))
    print(f"{path}: {added} lines {'added' if write else 'to add'}")
    for f in flags:
        print("  " + f)
    if not write:
        print("\n".join(out))


def in_section(lines, i, name):
    for k in range(i, -1, -1):
        l = lines[k]
        if re.match(r"^(dialogue|ex1|ex2|tasks|(sounds|notes|culture|aside)\.[a-z]+):\s*$", l) or l.startswith("== "):
            return l.strip() == name
    return False


def selfcheck():
    cases = {
        "Hello, my name is Marco.": "hë*lou*, mai neim iz *ma‣*kou.",
        "I live on the third floor.": "ai liv on dhë θë‣d flo‣.",
        "A glass of water after class.": "ë gla‣s ëv *wo‣*tër *a‣f*të kla‣s.",
        "Lesson Twenty-One": "*le*ssën *twen*ti-wan",
        "A cup of tea, please.": "ë kap ëv ti‣, pli‣z.",
        "The apple and the orange.": "dhi *è*pël ënd dhi *o*ring'.",
        "Where are you from?": "weër ë yu from?",
        "Are you Italian? Can I help?": "a‣ yu i*tè*liën? kën ai help?",
        "Thank you very much.": "θènk yu *ve*ri mac'.",
        "One, two, three.": "wan, tu‣, θri‣.",
        "It wasn't her dog. Invite her.": "it *wo*zënt hë dog. in*vait* hë‣.",
        "You're welcome. Usually she goes shopping.": "yo‣ *wel*këm. *yu‣*žuëli sci‣ gouz *scio*ping.",
    }
    bad = [(s, figure(s)[0], want) for s, want in cases.items() if figure(s)[0] != want]
    for s, got, want in bad:
        print(f"selfcheck: {s!r} gives {got!r}, expected {want!r}")
    return not bad


if __name__ == "__main__":
    if "--say" in sys.argv:
        for s in sys.argv[sys.argv.index("--say") + 1 :]:
            print(figure(s)[0])
        sys.exit(0)
    if not selfcheck():
        sys.exit(1)
    for p in [a for a in sys.argv[1:] if not a.startswith("--")]:
        fill(p, "--write" in sys.argv)
