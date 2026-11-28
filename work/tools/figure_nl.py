"""Figured pronunciation of Netherlands Dutch for Italian readers.

usage: python3 -I figure_nl.py <lesson file>... [--write]
       python3 -I figure_nl.py --say "<Dutch sentence>"

Fills the missing `pron.it:` lines of Foundation lessons, following
content/nl/pronunciation.md: in lessons 1–28 the whole line (and the
numeral, in every lesson), from lesson 29 only the words new in that lesson,
the rest left out ("…"), as the Assimil books do. Existing lines are kept,
so a hand correction survives. Without --write it prints what it would add.

The sounds come from espeak-ng (its Dutch voice), the copy bundled with the
app's voices in ~/.local/share/lingua-onde/voices/piper. Its stress and a few
sounds are wrong for some words: FIX holds the right ones.
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

FULL = 28  # full_figured() in src/format.rs

# the app's data folder, by its name since the rename to Marea or the old one
PIPER = next((d for d in (Path.home() / ".local/share/marea/voices/piper", Path.home() / ".local/share/lingua-onde/voices/piper") if d.exists()), Path.home() / ".local/share/marea/voices/piper")
CACHE = Path.home() / ".cache/lingua-onde-espeak/espeak-ng"

# espeak-ng's Dutch voice gets these wrong (mostly the stress).
FIX = {
    "mevrouw": "məvrˈʌʊ",
    "meneer": "mənˈeːr",
    "koffie": "kˈɔfi",
    "goedemorgen": "ɣudəmˈɔrɣən",
    "goedemiddag": "ɣudəmˈɪdɑx",
    "goedenavond": "ɣudənˈaːvɔnt",
    "goedenacht": "ɣudənˈɑxt",
    "jullie": "jˈɵli",
    "vakantie": "vaːkˈɑnsi",
    "italiaan": "itaːliˈaːn",
    "italiaans": "itaːliˈaːns",
    "italiaanse": "itaːliˈaːnsə",
    "italië": "itˈaːliə",
    "amsterdam": "ɑmstərdˈɑm",
    "rotterdam": "rɔtərdˈɑm",
    "alsjeblieft": "ɑlʃəblˈift",
    "alstublieft": "ɑlstyblˈift",
    "dankjewel": "dɑŋkjəʋˈɛl",
    "politie": "poːlˈitsi",
    "station": "staːʃˈɔn",
    "informatie": "ɪnfɔrmˈaːtsi",
    "natuurlijk": "naːtˈyrlək",
    "eigenlijk": "ˈɛɪɣənlək",
    "waarschijnlijk": "ʋaːrsxˈɛɪnlək",
    "bologna": "boːlˈɔɲaː",
    "hè": "hˈɛ",
    "perfect": "pɛrfˈɛkt",
    "ophaalbrug": "ˈɔphaːlbrʏx",
    "allebei": "ɑləbˈɛɪ",
    "gisteravond": "ɣɪstərˈaːvɔnt",
    "detective": "ditˈɛktɪf",
    "opstaan": "ˈɔpstaːn",
    "opstaat": "ˈɔpstaːt",
    "kilometer": "kiloːmˈeːtər",
    "appeltaart": "ˈɑpəltaːrt",
    "parkeerplaats": "pɑrkˈeːrplaːts",
    "goedkoop": "ɣˈutkoːp",
    "goedkope": "ɣˈutkoːpə",
    "ingecheckt": "ˈɪnɣətʃɛkt",
    "check": "tʃˈɛk",
    "afstandsbediening": "ˈɑfstɑntsbədinɪŋ",
    "wanneer": "ʋɑnˈeːr",
    "oh": "ˈoː",
    "wc": "ʋeːsˈeː",
    "waarmee": "ʋaːrmˈeː",
    "jawel": "jaːʋˈɛl",
    "precies": "prəsˈis",
    "omdat": "ɔmdˈɑt",
    "modern": "moːdˈɛrn",
    "moderner": "moːdˈɛrnər",
    "ertussenin": "ɛrtɵsənˈɪn",
    "weekend": "ʋˈikɛnt",
    "ernaar": "ɛrnˈaːr",
    "overschakelen": "ˈoːvərsxaːkələ",
    "toerist": "turˈɪst",
    "journaal": "ʃurnˈaːl",
    "aruba": "aːrˈubaː",
    "curaçao": "kyraːsˈʌʊ",
    "bonaire": "boːnˈɛːr",
    "portugees": "pɔrtyɣˈeːs",
    "onafhankelijk": "ɔnɑfhˈɑŋkələk",
    "intelligent": "ɪntɛliɣˈɛnt",
    "zeldzaam": "zˈɛltsaːm",
    "rondvertellen": "rˈɔntfərtɛlə",
    "direct": "dirˈɛkt",
    "overgenomen": "ˈoːvərɣənoːmə",
    "taalgebied": "tˈaːlɣəbit",
    "cookie": "kˈuki",
    "breukelen": "brˈøːkələ",
    "provincie": "proːvˈɪnsi",
    "engelse": "ˈɛŋəlsə",
    "dialecten": "diaːlˈɛktə",
    "dialect": "diaːlˈɛkt",
    "aparte": "aːpˈɑrtə",
    "taalunie": "tˈaːlyni",
    "tweetalig": "tʋˈeːtaːləx",
    "zuiderzee": "zˈœydərzeː",
    "noordzee": "nˈoːrtseː",
    "cornelis": "kɔrnˈeːlɪs",
    "onderhouden": "ɔndərhˈʌʊdə",
    "product": "proːdˈʏkt",
    "onbeleefd": "ˈɔmbəleːft",
    "functioneert": "fʏŋkʃoːnˈeːrt",
    "lawaai": "laːʋˈaːj",
    "overleg": "ˈoːvərlɛx",
    "drumstel": "drˈʏmstɛl",
    "badge": "bˈɛtʃ",
    "makkie": "mˈɑki",
    "ijsselmeer": "ˈɛɪsəlmeːr",
    "waddenzee": "ʋˈɑdənzeː",
    "verven": "vˈɛrvə",
    "keukenschaar": "kˈøːkənsxaːr",
    "tenminste": "tɛnmˈɪnstə",
    "borrel": "bˈɔrəl",
    "stoomboot": "stˈoːmboːt",
    "hardop": "hɑrdˈɔp",
    "surprise": "syrprˈizə",
    "chocoladeletters": "ʃoːkoːlˈaːdəlɛtərs",
    "fietspaden": "fˈitspaːdə",
    "achterkant": "ˈɑxtərkɑnt",
    "zijkant": "zˈɛɪkɑnt",
    "onze": "ˈɔnzə",
    "gastvrij": "ɣɑstfrˈɛɪ",
    "tegenwindfietsen": "ˈteːɣənʋɪntfitsə",
    "online": "ˈɔnlɑin",
    "koningin": "koːnɪŋˈɪn",
    "koninginnedag": "koːnɪŋˈɪnədɑx",
    "terwijl": "tɛrʋˈɛɪl",
    "voordat": "voːrdˈɑt",
    "voorbij": "voːrbˈɛɪ",
    "oranje": "oːrˈɑnjə",
    "miljoenen": "mɪljˈunə",
    "doodmoe": "doːtmˈu",
    "maastricht": "maːstrˈɪxt",
    "thuisgebleven": "tˈœysxəbleːvən",
    "thuisgekomen": "tˈœysxəkoːmən",
    "terugvinden": "tərˈɵxfɪndən",
    "slagroom": "slˈɑxroːm",
    "rabarber": "raːbˈɑrbər",
    "chocolademelk": "ʃoːkoːlˈaːdəmɛlk",
    "naartoe": "naːrtˈu",
    "gewoontes": "ɣəʋˈoːntəs",
    "waarom": "ʋaːrˈɔm",
    "vannacht": "vˈɑnɑxt",
    "schoonmaken": "sxˈoːnmaːkən",
    "hoi": "hˈɔj",
    "hallo": "hɑlˈoː",
    "aangenaam": "aːnɣənˈaːm",
    "milaan": "milˈaːn",
    "stroopwafel": "strˈoːpʋaːfəl",
    "stroopwafels": "strˈoːpʋaːfəls",
    "sorry": "sˈɔri",
    "vriendin": "vrindˈɪn",
    "gefeliciteerd": "ɣəfeːlisitˈeːrt",
    "honderdste": "hˈɔndərtstə",
}

# Weak forms of small words inside a sentence; at the end of a sentence or
# before punctuation the full form stays.
WEAK = {
    "het": "ət",
    "een": "ən",
    "de": "də",
    "te": "tə",
    "je": "jə",
    "ze": "zə",
    "we": "ʋə",
    "me": "mə",
}

# The signs follow Assimil's Italian editions: Italian spelling, the stressed
# syllable in bold (*...*), ‣ after a long vowel, ë for the weak e, è for the
# open e, ü, ø and œ (Il Francese senza sforzo), I for the relaxed short i
# (as Le nouveau néerlandais sans peine writes it), χ for the throaty g / ch
# (the jota of Lo Spagnolo), sc/sci for sh and y for the i of ieri, as in the
# English course for Italians.
VOWELS = [  # IPA, figured
    ("ɛɪ", "èi"), ("yʊ", "üu"), ("ʌʊ", "au"), ("œy", "œi"), ("eʊ", "e‣u"),
    ("aː", "a‣"), ("eː", "e‣"), ("oː", "o‣"), ("øː", "ø‣"), ("ɛː", "è‣"), ("ɔː", "o‣"), ("ɪː", "e‣"), ("ʏː", "ø‣"),
    ("i", "i"), ("u", "u"), ("y", "ü"),
    ("ɪ", "I"), ("ɛ", "è"), ("ɑ", "a"), ("a", "a"), ("ɔ", "o"), ("ɵ", "œ"), ("ʏ", "œ"),
    ("ə", "ë"), ("e", "e"), ("o", "o"),
]
LONG = "‣"
CONS = {
    "p": "p", "b": "b", "t": "t", "d": "d", "k": "k", "f": "f", "v": "v", "m": "m", "n": "n",
    "l": "l", "h": "h", "s": "s", "z": "z", "ʒ": "ž", "r": "r", "ʀ": "r", "ɾ": "r", "ʁ": "r", "ɹ": "r", "j": "y",
    "ʋ": "w", "w": "u", "x": "χ", "ɣ": "χ", "ŋ": "ng", "ɲ": "gn",
}
# Onsets a syllable may start with (IPA), for the bold stressed syllable.
ONSETS = {
    "pl", "bl", "kl", "ɣl", "xl", "fl", "sl", "vl", "pr", "br", "tr", "dr", "kr", "ɣr", "xr",
    "fr", "vr", "ʋr", "sp", "st", "sk", "sm", "sn", "sʋ", "kʋ", "tʋ", "dʋ", "zʋ", "kn",
    "sxr", "spr", "str", "skr", "spl",
}
VOICELESS_END = ("p", "t", "k", "s", "f", "χ")


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
        [espeak(), f"--path={PIPER}", "-v", "nl", "-q", "--ipa"],
        input=word, capture_output=True, text=True, env={"LD_LIBRARY_PATH": str(PIPER)},
    )
    s = "".join(r.stdout.split()).replace(chr(0x200D), "")
    # The j-glide of -tje / -sje is the sh / ch of meisje, katje.
    return s.replace("tʲ", "tʃ").replace("tj", "tʃ").replace("sj", "ʃ").replace("ɡ", "g")


def tokens(s):
    """IPA string -> [(kind, ipa, stressed)] with kind 'v' or 'c'."""
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
            if ch in CONS or ch in "ʃg":
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
    """One word's IPA -> figured spelling, by content/nl/pronunciation.md."""
    if "-" in ipa_word:
        return "-".join(spell(p) for p in ipa_word.split("-"))
    toks = tokens(ipa_word)
    # -en at the end of a longer word loses its n: maken -> ma‣kë.
    if len(toks) > 2 and toks[-1] == ("c", "n", False) and toks[-2][1] == "ə" \
            and sum(t[0] == "v" for t in toks) > 1:
        toks = toks[:-1]
    figured = dict(VOWELS)
    vowels = [n for n, t in enumerate(toks) if t[0] == "v"]
    one_syllable = len(vowels) == 1
    letters = []
    for n, (kind, v, stressed) in enumerate(toks):
        if kind == "v":
            f = figured[v]
            # ie, oe, uu are short, as Le nouveau néerlandais sans peine writes
            # them (ziek "sik"), and long only before r (vier).
            if v in ("i", "u", "y") and n + 1 < len(toks) and toks[n + 1][1] == "r":
                f += LONG
            # Only a stressed syllable (or a word of one) is heard long.
            if not (stressed or one_syllable):
                f = f.replace(LONG, "")
            letters.append(f)
            continue
        nxt = toks[n + 1] if n + 1 < len(toks) else None
        vowel_next = nxt is not None and nxt[0] == "v"
        nf = figured[nxt[1]] if vowel_next else ""
        front = vowel_next and nf[0] in "iIeèë"
        vowel_before = n > 0 and toks[n - 1][0] == "v"
        if v == "tʃ":
            letters.append("c" if front else "ci" if vowel_next else "c'")
        elif v == "dʒ":
            letters.append("g" if front else "gi" if vowel_next else "g'")
        elif v == "ʃ":
            letters.append("sc" if front else "sci" if vowel_next else "sc'")
        elif v == "g":
            letters.append("gh" if front else "g")
        elif v == "ŋ":
            letters.append("n" if nxt and nxt[1] in ("k", "g") else "ng")
        elif v == "w" and vowel_next:
            letters.append("w")  # nieuwe: the u of nieuw opens onto the next vowel
        elif v == "j" and vowel_before and not vowel_next:
            letters.append("i")  # mooi, haai: the glide closes the vowel
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


def assimilate(prev, word):
    """Voicing across a word boundary: ik zie -> ik sie, op vakantie -> op fakantie,
    ik ben -> ig ben (content/nl/pronunciation.md, Le parole che si legano)."""
    bare = word.lstrip("*")
    star = word[: len(word) - len(bare)]
    p = prev.rstrip("*")
    if p.endswith(VOICELESS_END) and bare[:1] in ("v", "z"):
        return prev, star + {"v": "f", "z": "s"}[bare[0]] + bare[1:]
    if bare[:1] in ("b", "d") and p[-1:] in ("k", "p", "t") and not p.endswith(("ss",)):
        tail = prev[len(p):]
        return p[:-1] + {"k": "g", "p": "b", "t": "d"}[p[-1]] + tail, word
    return prev, word


def figure(sentence):
    """A sentence -> (figured pronunciation, [])."""
    items = WORD.findall(sentence.replace("’", "'"))
    out = []
    for n, w in enumerate(items):
        if not w[0].isalpha():
            out.append(w)
            continue
        lw = w.lower()
        nxt = items[n + 1] if n + 1 < len(items) else "."
        if lw in WEAK and nxt[0].isalpha():
            sound = WEAK[lw]
        else:
            sound = ipa(w)
        word = spell(sound)
        if out and out[-1][0].isalpha() or out and out[-1][:1] == "*":
            out[-1], word = assimilate(out[-1], word)
        if out and out[-1][-1:].isalpha() and out[-1][-1] not in "aeiouèëüøœ‣" \
                and word.lstrip("*")[:1] == out[-1][-1]:
            out[-1] = out[-1][:-1]  # kost dat -> kos dat: one consonant, heard once
        out.append(word)
    text = ""
    for t in out:
        text += t if t in ".,;:?!…" or not text else " " + t
    return text, []


def met_before(path, n):
    """Words of the dialogues of the course's lessons before lesson n."""
    p = Path(path).resolve()
    repo = next((d for d in p.parents if (d / "Cargo.toml").exists()), None)
    found = {}
    if repo:
        for f in sorted((repo / "content" / "nl").glob("foundation-*.txt")):
            found.update(progression.lessons(f.read_text(encoding="utf-8")))
    for f in sorted(p.parent.glob("[0-9][0-9][0-9].txt")):
        found.update(progression.lessons(f.read_text(encoding="utf-8")))
    return progression.words(d for k, v in found.items() if k < n for d in v["dialogue"])


def new_words_only(sentence, met):
    """The figured spelling of the words not met yet, the rest left out."""
    full, _ = figure(sentence)
    items = [w for w in WORD.findall(sentence.replace("’", "'"))]
    spelled = re.sub(r"[.,;:?!…]", " ", full).split()
    words = [w for w in items if w[0].isalpha()]
    if len(words) != len(spelled):
        return None
    out, gap = [], False
    for w, s in zip(words, spelled):
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
    out, level, lesson, added, revision, met = [], None, 0, 0, False, set()
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
            if any(b.strip().startswith("pron.it:") for b in block):
                # A line figured on an earlier run still counts as met, so a
                # second run adds nothing for its words further down.
                met.update(w.lower() for w in WORD.findall(line[2:]) if w[0].isalpha())
            else:
                text, _ = figure(line[2:])
                if not full:
                    text = new_words_only(line[2:], met)
                if text is None:
                    out.extend(block)
                    i = j
                    continue
                at = next((k for k, b in enumerate(block) if b.strip().startswith("it:")), len(block) - 1)
                block.insert(at + 1, f"  pron.it: {text}")
                added += 1
            out.extend(block)
            i = j
            continue
        i += 1
    if write:
        Path(path).write_text("\n".join(out))
    print(f"{path}: {added} lines {'added' if write else 'to add'}")
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
        "Goedemorgen, mevrouw. Een kopje koffie, alstublieft.":
            "χudë*mor*χë, më*vrau*. ën *kop*yë *ko*fi, alstü*blift*.",
        "Het meisje zit op de fiets.": "ët *mèi*scë zIt ob dë fits.",
        "Ik zie het. We gaan op vakantie.": "Ik si hët. wë χa‣n op fa*kan*si.",
        "Sorry. Hoeveel kost dat?": "*so*ri. hu*ve‣l* kos dat?",
        "Een broodje kaas, en wat te drinken.": "ën *bro‣*cë ka‣s, èn wa të *drIn*kë.",
        "Mooi weer vandaag. Nieuw huis, nieuwe buren.":
            "mo‣i we‣r van*da‣χ*. niu hœis, *ni*wë *bü‣*rë.",
        "Eerste les": "*e‣r*stë lès",
        "Waar woont u? In Utrecht. Vier uur.": "wa‣r wo‣nt ü? In *œ*trèχt. vi‣r ü‣r.",
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
