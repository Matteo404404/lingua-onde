"""Figured pronunciation of Spanish (Spain, Madrid) for English readers.

usage: python3 -I figure_es.py <lesson file>... [--write]
       python3 -I figure_es.py --say "<Spanish sentence>"
       python3 -I figure_es.py --test

Rewrites every `pron.en:` line (and `numeral.pron.en:`) of Foundation lessons
from the Spanish line, following the English readers' key in
content/es/pronunciation.md, which takes its signs from Assimil's Spanish with
Ease: é for every e, ou for u, ss for an s inside a word, H for the jota, c and
ç for the th of think, gn for ñ, tch for ch, ï for the i that closes a
diphthong, the stressed syllable in bold (*...*). Lessons 1–28 get the whole
line, later lessons only the words new in the lesson ("…" for the rest), as the
Assimil books do. Spanish spelling is regular, so the rules below do the work;
FIX holds the words whose spelling does not give the sound. Correct a word
there and rerun, never by hand on a pron line: the next run rewrites it.
Without --write it prints the file it would write.
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import progression  # noqa: E402

FULL = 28  # full_figured() in src/format.rs
BASE = "en"

# Words whose spelling does not give the sound (x said as the jota, loans).
FIX = {
    "méxico": "*mé*Hico",
    "mexicano": "méHi*ca*no",
    "mexicana": "méHi*ca*na",
    "texas": "*té*Hass",
    "oaxaca": "oa*Ha*ca",
    "wifi": "*oui*fi",
    "subrayar": "soubra*yar*",
    "psicólogo": "ssi*co*logo",
    "psicóloga": "ssi*co*loga",
    "ninot": "ni*not*",
    "ninots": "ni*nots*",
    "hemingway": "*Hé*minguéï",
}

# Words ending in -mente that are not adverbs in -mente (two stresses).
NOT_ADVERB = {"clemente", "implemente", "complemente", "suplemente", "demente"}

VOWELS = "aeiouáéíóúü"
ACCENTED = {"á": "a", "é": "e", "í": "i", "ó": "o", "ú": "u"}
FRONT = "eiéí"
# Two consonants that start a syllable together (by sound, after the units below).
ONSETS = {("p", "l"), ("p", "r"), ("b", "l"), ("b", "r"), ("f", "l"), ("f", "r"),
          ("k", "l"), ("k", "r"), ("g", "l"), ("g", "r"), ("t", "r"), ("d", "r")}


def units(word):
    """Spanish spelling -> [(kind, sound, accented)]: kind 'v' (a e i o u) or 'c'.

    Consonant sounds: b k th ch d f g gh (hard g before e, i) H l m n gn p r rr
    s t w y, and k + xs for x between vowels; h is dropped."""
    w = word.lower()
    at = lambda k: w[k] if 0 <= k < len(w) else ""  # noqa: E731
    out, i = [], 0
    while i < len(w):
        ch, nx = w[i], at(i + 1)
        if ch in ACCENTED or ch in "aeiou":
            out.append(("v", ACCENTED.get(ch, ch), ch in ACCENTED))
            i += 1
            continue
        if ch == "ü":
            out.append(("v", "u", False))
            i += 1
            continue
        two = w[i:i + 2]
        if two in ("ch", "ll", "rr"):
            out.append(("c", {"ch": "ch", "ll": "y", "rr": "rr"}[two], False))
            i += 2
            continue
        if ch == "q":
            out.append(("c", "k", False))
            i += 2 if nx == "u" else 1
            continue
        if ch == "g" and nx == "u" and at(i + 2) != "" and at(i + 2) in FRONT:
            out.append(("c", "gh", False))
            i += 2
            continue
        if ch == "h":
            # hielo, hierba: initial hi + vowel is the consonant y
            if i == 0 and nx == "i" and at(i + 2) != "" and at(i + 2) in VOWELS:
                out.append(("c", "y", False))
                i += 2
            else:
                i += 1
            continue
        if ch == "y":
            out.append(("c", "y", False) if nx != "" and nx in VOWELS else ("v", "i", False))
        elif ch == "c":
            out.append(("c", "th" if nx != "" and nx in FRONT else "k", False))
        elif ch == "z":
            out.append(("c", "th", False))
        elif ch == "g":
            out.append(("c", "H" if nx != "" and nx in FRONT else "g", False))
        elif ch == "j":
            out.append(("c", "H", False))
        elif ch in "bv":
            out.append(("c", "b", False))
        elif ch == "ñ":
            out.append(("c", "gn", False))
        elif ch == "x":
            if i == 0 or (nx != "" and nx not in VOWELS):
                out.append(("c", "s", False))
            else:
                out += [("c", "k", False), ("c", "xs", False)]  # ta-xi: k ends one syllable, s starts the next
        elif ch in "dfklmnprstw":
            out.append(("c", ch, False))
        else:
            raise ValueError(f"no rule for {ch!r} in {word!r}")
        i += 1
    return out


def _strong(u):
    return u[1] in "aeo"


def _joins(group, us, n):
    """Whether vowel us[n] belongs to the same syllable as the vowels in group."""
    last, v = us[group[-1]], us[n]
    if last[1] == v[1]:
        return False  # leer, chiita
    if last[1] in "iu" and v[1] in "iu":
        return True  # cuidado, ciudad, cuídate: always one syllable
    if (last[1] in "iu" and last[2]) or (v[1] in "iu" and v[2]):
        return False  # día, país: the written accent splits them
    if _strong(last) and _strong(v):
        return False  # teatro, creo
    if _strong(v) and any(_strong(us[k]) for k in group):
        return False
    return True


def syllables(us):
    """Units -> list of syllables (each a list of units)."""
    groups = []
    for n, u in enumerate(us):
        if u[0] != "v":
            continue
        if groups and groups[-1][-1] == n - 1 and _joins(groups[-1], us, n):
            groups[-1].append(n)
        else:
            groups.append([n])
    if not groups:
        return [us]
    cuts = [0]
    for g1, g2 in zip(groups, groups[1:]):
        cons = list(range(g1[-1] + 1, g2[0]))
        if not cons:
            cuts.append(g2[0])
        elif len(cons) == 1:
            cuts.append(cons[0])
        elif (us[cons[-2]][1], us[cons[-1]][1]) in ONSETS:
            cuts.append(cons[-2])  # ha-blo, en-tre, ins-tru-ir
        else:
            cuts.append(cons[-1])  # ten-go, ins-tan-te
    cuts.append(len(us))
    return [us[a:b] for a, b in zip(cuts, cuts[1:])]


def stressed(word, syls):
    """Index of the stressed syllable: the written accent, else the default rule."""
    for n, s in enumerate(syls):
        if any(u[0] == "v" and u[2] for u in s):
            return n
    last = word.lower()[-1]
    if len(syls) > 1 and (last in "aeiouns"):
        return len(syls) - 2
    return len(syls) - 1


def spell_syllable(s, first, word_start):
    """One syllable's units -> figured letters."""
    vowels = [k for k, u in enumerate(s) if u[0] == "v"]
    peak = next((k for k in vowels if _strong(s[k])), vowels[-1] if vowels else None)
    out = []
    for k, (kind, snd, _) in enumerate(s):
        nxt = s[k + 1] if k + 1 < len(s) else None
        if kind == "v":
            if k == peak or peak is None:
                out.append({"a": "a", "e": "é", "i": "i", "o": "o", "u": "ou"}[snd])
            elif k < peak:
                out.append("i" if snd == "i" else "ou")  # bién, bouéno
            else:
                out.append("ï" if snd == "i" else "ou")  # oï, séïss, aouto
            continue
        if snd == "s":
            out.append("s" if first and k == 0 and word_start else "ss")
        elif snd == "th":
            front = nxt is not None and nxt[0] == "v" and nxt[1] in "ei"
            out.append("c" if front else "ç")
        elif snd == "k":
            front = nxt is not None and nxt[0] == "v" and nxt[1] in "ei"
            out.append("k" if front else "c")
        elif snd == "xs":
            out.append("s")
        elif snd == "gh":
            out.append("gu")
        elif snd == "ch":
            out.append("tch")
        else:
            out.append(snd)
    return "".join(out)


def figure_word(word, force=False):
    """One Spanish word -> its figured spelling, the stressed syllable in *...*."""
    lw = word.lower()
    if lw in FIX:
        return FIX[lw]
    if lw.endswith("mente") and len(lw) > 7 and lw[-6] not in "oiu" and lw not in NOT_ADVERB:
        # rápidamente: two stresses; the hyphen keeps the two bold syllables apart
        return figure_word(lw[:-5], force=True) + "-*mén*té"
    us = units(lw)
    syls = syllables(us)
    # The k of cc (acción) and of x (taxi) is written k: lékcion, *tak*si.
    letters = []
    for n, s in enumerate(syls):
        text = spell_syllable(s, n == 0, True)
        nxt = syls[n + 1] if n + 1 < len(syls) else []
        if text.endswith("c") and nxt and nxt[0][1] in ("th", "xs"):
            text = text[:-1] + "k"
        letters.append(text)
    if letters and letters[-1].endswith("c"):
        letters[-1] = letters[-1][:-1] + "k"  # coñac, bistec
    if (len(syls) > 1 or force) and any(u[0] == "v" for u in us):
        k = stressed(lw, syls)
        letters[k] = "*" + letters[k] + "*"
    return "".join(letters)


TOKEN = re.compile(r"[^\W\d_]+|[¿¡.,;:?!…]|\d")


def figure_tokens(sentence):
    """[(token, figured)] for a sentence; punctuation is kept, quotes and dashes dropped."""
    out = []
    for t in TOKEN.findall(sentence):
        if t.isdigit():
            raise ValueError(f"write numbers in words: {sentence!r}")
        out.append((t, t if not t[0].isalpha() else figure_word(t)))
    return out


def join(figured):
    text = ""
    for f in figured:
        if not text or text.endswith(("¿", "¡")) or f in ".,;:?!…":
            text += f
        else:
            text += " " + f
    return text


def figure(sentence):
    return join(f for _, f in figure_tokens(sentence))


def met_before(path, n):
    """Words of the dialogues of the course's lessons before lesson n."""
    p = Path(path).resolve()
    repo = next((d for d in p.parents if (d / "Cargo.toml").exists()), None)
    found = {}
    if repo:
        for f in sorted((repo / "content" / "es").glob("foundation-*.txt")):
            found.update(progression.lessons(f.read_text(encoding="utf-8")))
    for f in sorted(p.parent.glob("[0-9][0-9][0-9].txt")):
        found.update(progression.lessons(f.read_text(encoding="utf-8")))
    return progression.words(d for k, v in found.items() if k < n for d in v["dialogue"])


def new_words_only(sentence, met):
    """The figured spelling of the words not met yet, the rest left out ("…")."""
    out, gap = [], False
    for t, f in figure_tokens(sentence):
        if not t[0].isalpha():
            continue
        if t.lower() in met:
            gap = True
            continue
        if gap or (not out and t != next(x for x in TOKEN.findall(sentence) if x[0].isalpha())):
            out.append("…")
        out.append(f)
        met.add(t.lower())
        gap = False
    if not out:
        return None
    if gap:
        out.append("…")
    return " ".join(out)


def in_section(lines, i, name):
    for k in range(i, -1, -1):
        l = lines[k]
        if re.match(r"^(dialogue|ex1|ex2|tasks|(sounds|notes|culture|aside)\.[a-z]+):\s*$", l) or l.startswith("== "):
            return l.strip() == name
    return False


def fill(path, write):
    """Rewrite the pron.en lines of a lesson file (all of them, so they follow the Spanish)."""
    src = Path(path).read_text(encoding="utf-8").split("\n")
    lines = [l for l in src if not l.startswith(f"numeral.pron.{BASE}:") and not l.strip().startswith(f"pron.{BASE}:")]
    out, lesson, level, revision, met, added = [], 0, None, False, set(), 0
    i = 0
    while i < len(lines):
        line = lines[i]
        out.append(line)
        m = re.match(r"^== (\d+) (A1|A2|B1|B2|C1|C2)", line)
        if m:
            lesson, level, revision = int(m.group(1)), m.group(2), False
            met = met_before(path, lesson) if lesson > FULL else set()
        foundation = level in ("A1", "A2", "B1", "B2")
        full = lesson <= FULL
        if line.strip() == "kind: revision":
            revision = True
        if foundation and line.startswith("numeral: "):
            out.append(f"numeral.pron.{BASE}: {figure(line[len('numeral: '):])}")
            added += 1
        if foundation and line.startswith("- ") and in_section(lines, i, "dialogue:") and not (revision and not full):
            j = i + 1
            block = []
            while j < len(lines) and lines[j].startswith("  "):
                block.append(lines[j])
                j += 1
            # A line said in English (its translation is the line itself) gets no Spanish figure.
            english = any(b.strip() == f"{BASE}: {line[2:]}" for b in block)
            text = None if english else figure(line[2:]) if full else new_words_only(line[2:], met)
            if text is not None:
                at = next((k for k, b in enumerate(block) if b.strip().startswith(f"{BASE}:")), len(block) - 1)
                block.insert(at + 1, f"  pron.{BASE}: {text}")
                added += 1
            out.extend(block)
            i = j
            continue
        i += 1
    if write:
        Path(path).write_text("\n".join(out), encoding="utf-8")
    print(f"{path}: {added} pron.{BASE} lines {'written' if write else 'to write'}")
    if not write:
        print("\n".join(out))


# My own examples, one or more for every rule of the key.
CASES = {
    "casa": "*ca*ssa",
    "mesa": "*mé*ssa",
    "sí": "si",
    "todo": "*to*do",
    "uno": "*ou*no",
    "baile": "*baï*lé",
    "hay": "aï",
    "seis": "séïss",
    "hoy": "oï",
    "estoy": "éss*toï*",
    "muy": "moui",
    "auto": "*aou*to",
    "euro": "*éou*ro",
    "bien": "bién",
    "bueno": "*boué*no",
    "cuatro": "*coua*tro",
    "agua": "*a*goua",
    "ciudad": "ciou*dad*",
    "cuidado": "coui*da*do",
    "vives": "*bi*béss",
    "aquí": "a*ki*",
    "queso": "*ké*sso",
    "cena": "*cé*na",
    "zapato": "ça*pa*to",
    "luz": "louç",
    "gracias": "*gra*ciass",
    "lección": "lék*cion*",
    "doctor": "doc*tor*",
    "mucho": "*mou*tcho",
    "guerra": "*gué*rra",
    "pingüino": "pin*goui*no",
    "gente": "*Hén*té",
    "jamón": "Ha*mon*",
    "josé": "Ho*ssé*",
    "hola": "*o*la",
    "ahora": "a*o*ra",
    "ahí": "a*i*",
    "hielo": "*yé*lo",
    "huevo": "*oué*bo",
    "llamo": "*ya*mo",
    "yo": "yo",
    "ayer": "a*yér*",
    "españa": "éss*pa*gna",
    "pero": "*pé*ro",
    "perro": "*pé*rro",
    "taxi": "*tak*si",
    "examen": "ék*sa*mén",
    "extranjero": "ésstran*Hé*ro",
    "hablo": "*a*blo",
    "tengo": "*tén*go",
    "entre": "*én*tré",
    "día": "*di*a",
    "lucía": "lou*ci*a",
    "país": "pa*iss*",
    "teatro": "té*a*tro",
    "madrid": "ma*drid*",
    "usted": "ouss*téd*",
    "italiano": "ita*lia*no",
    "bolonia": "bo*lo*nia",
    "rápidamente": "*ra*pida-*mén*té",
    "normalmente": "nor*mal*-*mén*té",
    "méxico": "*mé*Hico",
    "Buenos días. Perdona, ¿vives aquí?": "*boué*noss *di*ass. pér*do*na, ¿*bi*béss a*ki*?",
    "¡Un café con leche, por favor!": "¡oun ca*fé* con *lé*tché, por fa*bor*!",
    "Lección primera": "lék*cion* pri*mé*ra",
    "Lección veintiuna": "lék*cion* béïn*tiou*na",
}


def selfcheck():
    bad = [(s, figure(s), want) for s, want in CASES.items() if figure(s) != want]
    for s, got, want in bad:
        print(f"selfcheck: {s!r} gives {got!r}, expected {want!r}")
    return not bad


if __name__ == "__main__":
    if not selfcheck():
        sys.exit(1)
    if "--test" in sys.argv:
        print(f"figure_es.py: {len(CASES)} cases ok")
        sys.exit(0)
    if "--say" in sys.argv:
        for s in sys.argv[sys.argv.index("--say") + 1:]:
            print(figure(s))
        sys.exit(0)
    for p in [a for a in sys.argv[1:] if not a.startswith("--")]:
        fill(p, "--write" in sys.argv)
