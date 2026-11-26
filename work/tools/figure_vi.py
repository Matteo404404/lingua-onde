#!/usr/bin/env python3
"""Figured pronunciation for Spanish readers, from Vietnamese, by the key in
content/vi/pronunciation.md: one respelled word per syllable, then its tone
arrow, the sentence's punctuation after the arrow.

    figure_vi.py FILE...        rewrite every pron.es line from its dialogue
                                line and print what changed
    figure_vi.py --say TEXT     print the figure of TEXT

Foreign names (any syllable this script cannot read as Vietnamese) are kept
lowercase with →, as the writer's brief says.
"""
import re, sys, unicodedata

TONES = {'̀': '→', '́': '↑', '̉': '✓', '̃': '✓', '̣': '↓'}
ONSETS = [('ngh', 'ng'), ('ng', 'ng'), ('nh', 'ñ'), ('ch', 'ch'), ('tr', 'ch'),
          ('th', 'th'), ('kh', 'j'), ('ph', 'f'), ('gh', 'gu'), ('gi', 'z'),
          ('qu', 'cu'), ('d', 'z'), ('r', 'z'), ('đ', 'd'), ('x', 's'),
          ('s', 's'), ('b', 'b'), ('c', 'c'), ('k', 'c'), ('g', 'g'), ('h', 'h'),
          ('l', 'l'), ('m', 'm'), ('n', 'n'), ('t', 't'), ('v', 'v'), ('p', 'p')]
# Rimes, longest first: written rime -> figured rime (before the final consonant
# is attached). Order matters.
VOWELS = [('uyê', 'uiơ'), ('uya', 'uiơ'), ('iê', 'iơ'), ('yê', 'iơ'), ('ia', 'iơ'), ('ya', 'iơ'), ('uô', 'uơ'),
          ('ươ', 'ươ'), ('ưa', 'ươ'), ('ua', 'uơ'),
          ('ay', 'ăi'), ('au', 'ău'), ('ây', 'ơi'), ('âu', 'ơu'),
          ('oa', 'ua'), ('oă', 'uă'), ('oe', 'uè'), ('uy', 'ui'), ('uâ', 'uơ'),
          ('uê', 'ue'), ('uơ', 'uơ'),
          ('a', 'a'), ('ă', 'ă'), ('â', 'ơ'), ('e', 'è'), ('ê', 'e'), ('i', 'i'),
          ('y', 'i'), ('o', 'ò'), ('ô', 'o'), ('ơ', 'ơ'), ('u', 'u'), ('ư', 'ư')]
FINALS = ['ng', 'nh', 'ch', 'm', 'n', 'p', 't', 'c', 'i', 'y', 'o', 'u']
VI_LETTERS = set('aăâbcdđeêghiklmnoôơpqrstuưvxy')


def split_tone(syl):
    d = unicodedata.normalize('NFD', syl)
    arrow = '→'
    out = []
    for ch in d:
        if ch in TONES:
            arrow = TONES[ch]
        else:
            out.append(ch)
    return unicodedata.normalize('NFC', ''.join(out)), arrow


def figure_syllable(word):
    base, arrow = split_tone(word.lower())
    if not base or any(c not in VI_LETTERS for c in base):
        return None
    onset, rest = '', base
    for w, f in ONSETS:
        if base.startswith(w):
            # gi before a consonant or alone is g + i (gì, gìn)
            if w == 'gi' and (len(base) == 2 or base[2] not in 'aăâeêoôơuưy'):
                onset, rest = 'z', base[1:]
            # qu: the u is a glide, written as u in the figure
            elif w == 'qu':
                onset, rest = 'c', 'u' + base[2:]
            else:
                onset, rest = f, base[len(w):]
            break
    if onset == 'c' and rest[:1] in ('i', 'e', 'ê', 'y'):
        onset = 'qu'
    if onset == 'g' and rest[:1] in ('i', 'e', 'ê'):
        onset = 'gu'
    # qu + a: the glide u then a, not the diphthong ua
    glide = ''
    if base.startswith('qu') and len(rest) > 1:
        glide, rest = 'u', rest[1:]
    final = ''
    for f in FINALS:
        if rest.endswith(f) and len(rest) > len(f):
            final, rest = f, rest[:-len(f)]
            break
    nucleus = rest + (final if final in ('i', 'y', 'o', 'u') else '')
    tail = '' if final in ('i', 'y', 'o', 'u') else final
    fig = ''
    n = nucleus
    while n:
        for w, f in VOWELS:
            if n.startswith(w):
                fig += f
                n = n[len(w):]
                break
        else:
            return None
    # final glides: i/y as i, o/u as u (after the nucleus mapping above)
    if final in ('i', 'y', 'o', 'u') and len(nucleus) > 1:
        last = {'i': 'i', 'y': 'i', 'o': 'u', 'u': 'u'}[final]
        # the nucleus mapping turned the final into a vowel; replace its figure
        fig = fig[:-1] + last if not fig.endswith(('ăi', 'ău', 'ơi', 'ơu')) else fig
    if tail == 'nh':
        if fig.endswith('a'):
            fig, tail = fig + 'i', 'ñ'
        else:
            tail = 'ñ'
    elif tail == 'ch':
        if fig.endswith('a'):
            fig, tail = fig + 'i', 'c'
        else:
            tail = 'c'
    return onset + glide + fig + tail + arrow


def figure(text):
    out = []
    for tok in text.split():
        m = re.match(r'^([«"(¡¿]*)(.*?)([.,!?…:;»")]*)$', tok)
        _, word, post = m.groups()  # the pattern always matches
        if not word:
            continue
        f = figure_syllable(word)
        out.append((f if f else word.lower() + '→') + post)
    return ' '.join(out)


def rewrite(path):
    lines = open(path, encoding='utf-8').read().split('\n')
    last = None
    changed = 0
    for i, line in enumerate(lines):
        m = re.match(r'^- (.*)$', line)
        if m:
            last = m.group(1)
        m = re.match(r'^(\s*)pron\.es: (.*)$', line)
        if m and last is not None:
            new = figure(last)
            if new != m.group(2):
                print(f'{path}:{i + 1}\n  {last}\n  - {m.group(2)}\n  + {new}')
                lines[i] = m.group(1) + 'pron.es: ' + new
                changed += 1
    open(path, 'w', encoding='utf-8').write('\n'.join(lines))
    return changed


def self_test():
    cases = {'chào': 'chau→', 'cô': 'co→', 'người': 'ngươi→', 'nước': 'nươc↑',
             'muốn': 'muơn↑', 'tiếng': 'tiơng↑', 'hoa': 'hua→', 'quá': 'cua↑',
             'xuân': 'suơn→', 'tay': 'tăi→', 'sau': 'său→', 'đâu': 'dơu→',
             'đây': 'dơi→', 'gì': 'zi→', 'giáo': 'zau↑', 'giường': 'zương→',
             'anh': 'aiñ→', 'sách': 'saic↑', 'minh': 'miñ→', 'thích': 'thic↑',
             'ghế': 'gue↑', 'kia': 'quiơ→', 'không': 'jong→', 'phở': 'fơ✓',
             'của': 'cuơ✓', 'nhỏ': 'ñò✓', 'máy': 'măi↑', 'việt': 'viơt↓',
             'có': 'cò↑', 'ngon': 'ngòn→', 'em': 'èm→', 'bảy': 'băi✓',
             'tôi': 'toi→', 'mời': 'mơi→', 'hai': 'hai→', 'mèo': 'mèu→',
             'trong': 'chòng→', 'nhanh': 'ñaiñ→', 'khỏe': 'juè✓', 'thủy': 'thui✓', 'chuyện': 'chuiơn↓'}
    bad = [(k, figure_syllable(k), v) for k, v in cases.items() if figure_syllable(k) != v]
    for k, got, want in bad:
        print(f'self-test: {k} -> {got}, want {want}')
    return not bad


if __name__ == '__main__':
    if not self_test():
        sys.exit(1)
    if sys.argv[1:2] == ['--say']:
        print(figure(' '.join(sys.argv[2:])))
    else:
        n = sum(rewrite(p) for p in sys.argv[1:])
        print(f'{n} pron.es lines changed')
