"""NLTK differential for moult (dev-only; see DIFFERENCES.md).

Run from the repository root:

    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py [--guess | --lemma] [--check]

Compares moult's `singularize` (via `cargo xtask-inflect`) with
`WordNetLemmatizer().lemmatize(w, "n")` over Princeton WordNet 3.0, buckets every
disagreement by the ordered rules in `bucket()`, and writes expected.tsv (the
default mode, which never guesses), expected-guess.tsv (the opt-in
`guess_unknown` mode) and DIFFERENCES.md. With --guess only expected-guess.tsv
is written. With --lemma only expected-lemma.tsv is written: moult's
`lemmatize(w, Verb)` and `lemmatize(w, Adjective)` against NLTK's
`lemmatize(w, "v")` and `(w, "a")` (the default run writes it too, and the
lemma section of DIFFERENCES.md). With --check nothing is written; the exit
status is non-zero when a fresh file differs from the committed one (--check
alone: all three files, expected.tsv, expected-guess.tsv and expected-lemma.tsv;
--guess --check: the two noun files; --lemma --check: expected-lemma.tsv).
"""
import hashlib
import os
import random
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CACHE = HERE / ".cache"
DICT = Path("/usr/share/dict/words")
NLTK_PIN = "3.9.2"
DICT_HEADER = "# dict-sha256: "


def read_lines(p):
    return [l.rstrip("\r\n") for l in p.read_text(encoding="utf-8").splitlines() if l.strip()]


def rust_block(src, name):
    m = re.search(r"static %s: &\[[^\]]*\] = &\[\n(.*?)\n\];" % name, src, re.S)
    if not m:
        sys.exit("cannot find %s" % name)
    return m.group(1)


def first_strings(block):
    """First string literal of every non-comment entry line."""
    out = []
    for line in block.splitlines():
        m = re.match(r'\s*\(?\s*"((?:[^"\\]|\\.)*)"(?:\s*,\s*"((?:[^"\\]|\\.)*)")?', line)
        if m:
            out.append((m.group(1), m.group(2)))
    return out


def inflect(words, guess=False):
    r = subprocess.run(
        ["cargo", "run", "-q", "-p", "xtask", "--release", "--features", "inflect", "--", "inflect"]
        + (["--guess"] if guess else []),
        cwd=ROOT, input="\n".join(words) + "\n", capture_output=True, text=True, check=True,
    )
    res = {}
    for line in r.stdout.split("\n"):
        if line:
            w, s, p, n = line.split("\t")
            res[w] = (s, p, n == "true")
    return res


def inflect_lemma(words, candidates=False):
    """moult's lemmatize as verb and adjective, or all candidates (lists), for each word."""
    r = subprocess.run(
        ["cargo", "run", "-q", "-p", "xtask", "--release", "--features", "inflect", "--", "inflect",
         "--lemma-candidates" if candidates else "--lemma"],
        cwd=ROOT, input="\n".join(words) + "\n", capture_output=True, text=True, check=True,
    )
    split = (lambda x: x.split(",") if x else []) if candidates else (lambda x: x)
    res = {}
    for line in r.stdout.split("\n"):
        if line:
            w, v, a = line.split("\t")
            res[w] = (split(v), split(a))
    return res


def load_dict(check):
    """The system word list and its hash; --check refuses a list other than the recorded one."""
    if not DICT.exists():
        sys.exit("%s is absent; the differential's word list is pinned to the macOS dictionary (sha256 recorded in expected.tsv), so it cannot run here" % DICT)
    dict_bytes = DICT.read_bytes()
    dict_sha = hashlib.sha256(dict_bytes).hexdigest()
    if check:
        exp = HERE / "expected.tsv"
        first = exp.read_text(encoding="utf-8").split("\n", 1)[0] if exp.exists() else ""
        want = first[len(DICT_HEADER):] if first.startswith(DICT_HEADER) else None
        if want != dict_sha:
            sys.exit("%s has sha256 %s but expected.tsv was produced with %s; refusing to compare a different word list" % (DICT, dict_sha, want))
    words = sorted({l.strip().lower() for l in dict_bytes.decode("utf-8", errors="replace").splitlines() if l.strip()})
    return dict_sha, words


def main():
    args = sys.argv[1:]
    check = "--check" in args
    guess_only = "--guess" in args
    lemma_only = "--lemma" in args
    import nltk

    if nltk.__version__ != NLTK_PIN:
        sys.exit("expected nltk==%s, got %s" % (NLTK_PIN, nltk.__version__))
    CACHE.mkdir(exist_ok=True)
    nltk.data.path[:] = [str(CACHE)]
    if not (CACHE / "corpora" / "wordnet").exists():
        if not nltk.download("wordnet", download_dir=str(CACHE), quiet=True):
            sys.exit("could not download wordnet")
        z = CACHE / "corpora" / "wordnet.zip"
        if z.exists() and not (CACHE / "corpora" / "wordnet").exists():
            import zipfile
            with zipfile.ZipFile(z) as zf:
                zf.extractall(CACHE / "corpora")
    from nltk.corpus import wordnet as wn
    from nltk.stem import WordNetLemmatizer

    assert wn.get_version() == "3.0", wn.get_version()
    lem = WordNetLemmatizer()

    if lemma_only:
        dict_sha, dict_words = load_dict(check)
        rows, _ = lemma_diff(wn, lem, dict_words)
        tsv = lemma_tsv(rows, dict_sha)
        exp_l = HERE / "expected-lemma.tsv"
        if check:
            old = exp_l.read_text(encoding="utf-8") if exp_l.exists() else ""
            if old != tsv:
                print("%s differs from a fresh run" % exp_l.name, file=sys.stderr)
                sys.exit(1)
            print("%s is up to date (%d rows)" % (exp_l.name, len(rows)))
            sys.exit(0)
        exp_l.write_text(tsv, encoding="utf-8")
        print("wrote %s (%d rows)" % (exp_l, len(rows)))
        return

    nouns = set(read_lines(ROOT / "src/nouns.txt"))
    names = set(read_lines(ROOT / "src/names.txt"))
    # NLTK lower-cases index keys, so read the cased lemma names from the synsets:
    # pwn = common (all-lower-case) lemmas, pwn_names = lemmas written with capitals only.
    cased = {l.name() for ss in wn.all_synsets("n") for l in ss.lemmas()}
    pwn = {n for n in cased if n == n.lower()}
    pwn_names = {n.lower() for n in cased} - pwn
    pol = (ROOT / "src/policy.rs").read_text(encoding="utf-8")
    invariant = {a for a, _ in first_strings(rust_block(pol, "INVARIANT"))}
    choice = dict(first_strings(rust_block(pol, "SINGULAR_CHOICE")))
    man_regular = {a for a, _ in first_strings(rust_block(pol, "MAN_REGULAR"))}
    data = (ROOT / "src/data.rs").read_text(encoding="utf-8")
    form_pairs = first_strings(rust_block(data, "FORM_TO_LEMMA"))
    forms = [a for a, _ in form_pairs]
    irregular = dict(form_pairs)
    for lemma, plural in first_strings(rust_block(pol, "IRREGULAR")):
        irregular[plural] = lemma

    # Word list
    src_a = sorted(nouns)
    res0 = inflect(src_a)
    src_b = sorted({res0[w][1] for w in src_a})
    src_c = sorted(set(forms))
    # The system word list is an input of the committed expected.tsv, so its
    # hash is recorded there and --check refuses a different list.
    exp = HERE / "expected.tsv"
    if not DICT.exists():
        sys.exit("%s is absent; the differential's word list is pinned to the macOS dictionary (sha256 recorded in expected.tsv), so it cannot run here" % DICT)
    dict_bytes = DICT.read_bytes()
    dict_sha = hashlib.sha256(dict_bytes).hexdigest()
    if check:
        first = exp.read_text(encoding="utf-8").split("\n", 1)[0] if exp.exists() else ""
        want = first[len(DICT_HEADER):] if first.startswith(DICT_HEADER) else None
        if want != dict_sha:
            sys.exit("%s has sha256 %s but expected.tsv was produced with %s; refusing to compare a different word list" % (DICT, dict_sha, want))
    src_d = sorted({l.strip().lower() for l in dict_bytes.decode("utf-8", errors="replace").splitlines() if l.strip()})
    allw = sorted(set(src_a) | set(src_b) | set(src_c) | set(src_d))
    words = [w for w in allw if w == w.lower() and "\t" not in w and " " not in w]
    res = inflect(words)
    res_guess = inflect(words, guess=True)

    def nltk_s(w):
        return lem.lemmatize(w, "n")

    def bucket(w, m, n, is_noun):
        """Ordered rules; the first that matches wins. 'bug' is the residue."""
        # 1. invariant: the word is on moult's closed invariant list.
        if w in invariant:
            return "invariant"
        # 2. policy-choice: curated SINGULAR_CHOICE entry or a MAN_REGULAR noun.
        if w in choice or w in man_regular:
            return "policy-choice"
        # 3. irregular-form: a WordNet irregular form (or IRREGULAR supplement)
        #    that moult maps to its lemma and NLTK's exception list lacks.
        if irregular.get(w) == m:
            return "irregular-form"
        # 4. names: in moult's names list and not a common noun.
        if w in names and w not in nouns:
            return "names"
        # 5. numeric: decade or number plurals ("80s"); moult only guesses on letters.
        if re.fullmatch(r"\d+s", w):
            return "numeric"
        # 6. extra-rule: the checked -men -> -man and -ses -> -sis rules, which
        #    NLTK's detachment rules do not have.
        if (w.endswith("men") and m.endswith("man")) or (w.endswith("ses") and m.endswith("sis")):
            return "extra-rule"
        # 7. min-stem: moult never outputs fewer than 3 characters.
        if len(m) < 3 or len(n) < 3:
            return "min-stem"
        # 8. data-version: membership differs between OEWN 2025 and PWN 3.0
        #    for the word or either answer.
        if (w in nouns) != (w in pwn) or (m in nouns) != (m in pwn) or (n in nouns) != (n in pwn):
            # NLTK answering with a name only PWN writes with capitals is a morphy artefact.
            if n in pwn_names and n not in pwn:
                return "nltk-proper-name"
            return "data-version"
        # 9. class-A strip-wins: w and its stripped form are both common nouns;
        #    moult strips, NLTK keeps w.
        if w in nouns and m != w and n == w and m in nouns:
            return "class-A-strip-wins"
        # 10. nltk-shortest-candidate: w is a noun, moult keeps it, NLTK strips
        #     because lemmatize() returns the shortest candidate.
        if w in nouns and m == w and n != w and n in pwn:
            return "nltk-shortest-candidate"
        # 11. fallback: w unknown to both dictionaries and moult guessed.
        if w not in nouns and w not in pwn and not is_noun:
            if m != w and n == w:
                # Which unchecked rule fired.
                if w.endswith("ies") and m == w[:-3] + "y":
                    return "fallback-ies"
                if w.endswith("es") and m == w[:-2]:
                    return "fallback-es"
                if w[-2:-1] == "e":
                    return "fallback-s-after-e"
                return "fallback-s"
            if m == w and n != w and n in pwn:
                # NLTK's exception list (or a detachment rule) reaches a PWN noun
                # that moult's dictionary and rules do not.
                return "nltk-exception-only"
            return "fallback-other"
        # 12. nltk-morphy-quirk: NLTK's output is not a common WordNet noun.
        #     Also: NLTK leaves a word unchanged although the regular stem is a PWN noun.
        if (n not in pwn and n != w) or (n == w and m != w and m in pwn):
            return "nltk-morphy-quirk"
        # 13. different-lemma: both outputs are nouns but differ.
        if m in nouns and n in pwn and m != n and m != w and n != w:
            return "different-lemma"
        return "bug"

    def diff_rows(results):
        rows = []
        for w in words:
            m = results[w][0]
            n = nltk_s(w)
            if m != n:
                rows.append((w, m, n, bucket(w, m, n, results[w][2])))
        rows.sort()
        return rows

    def to_tsv(rows):
        return DICT_HEADER + dict_sha + "\nword\tmoult\tnltk\tbucket\n" + "".join("\t".join(r) + "\n" for r in rows)

    rows = diff_rows(res)
    rows_guess = diff_rows(res_guess)
    tsv, tsv_guess = to_tsv(rows), to_tsv(rows_guess)
    exp_guess = HERE / "expected-guess.tsv"
    if guess_only and not check:
        rows_lemma, info, tsv_lemma = [], None, ""
    else:
        rows_lemma, info = lemma_diff(wn, lem, src_d)
        tsv_lemma = lemma_tsv(rows_lemma, dict_sha)
    if check:
        ok = True
        checked = [(exp, tsv, len(rows)), (exp_guess, tsv_guess, len(rows_guess))]
        if not guess_only:
            checked.append((HERE / "expected-lemma.tsv", tsv_lemma, len(rows_lemma)))
        for path, fresh, n in checked:
            old = path.read_text(encoding="utf-8") if path.exists() else ""
            if old != fresh:
                print("%s differs from a fresh run" % path.name, file=sys.stderr)
                ok = False
            else:
                print("%s is up to date (%d rows)" % (path.name, n))
        sys.exit(0 if ok else 1)
    exp_guess.write_text(tsv_guess, encoding="utf-8")
    if guess_only:
        print("wrote %s (%d rows)" % (exp_guess, len(rows_guess)))
        return
    exp.write_text(tsv, encoding="utf-8")

    total = len(words)
    md = []
    md.append(JUSTIFY_HEAD)
    md.append("## Word list\n")
    md.append("| source | words |\n|---|---|")
    md.append("| src/nouns.txt | %d |" % len(src_a))
    md.append("| moult plural of each noun (unique) | %d |" % len(src_b))
    md.append("| FORM_TO_LEMMA forms | %d |" % len(src_c))
    md.append("| /usr/share/dict/words, lower-cased (sha256 `%s`) | %d |" % (dict_sha, len(src_d)))
    md.append("| union | %d |" % len(allw))
    md.append("| union, lower-case words compared | %d |\n" % total)
    md.append("The committed expected.tsv and expected-guess.tsv were produced on macOS's `/usr/share/dict/words` (sha256 `%s`). `--check` refuses, with a message, when that file is absent or its hash differs, rather than comparing a different word set.\n" % dict_sha)
    md.append("## Result: default mode\n")
    md.append("The default mode is the guarantee: `singularize` never guesses, so a word that no dictionary-checked step resolves is unchanged. The `fallback-*` buckets can then hold only rows where NLTK moves a word moult leaves alone.\n")
    md.extend(render(rows, total, shown_in_full=True))
    md.append("## Result: opt-in `guess_unknown(true)`\n")
    md.append("The same comparison with the unknown-word fallback on (`cargo xtask-inflect --guess`; rows in `expected-guess.tsv`). Every row that differs from the default mode is a word the fallback changed. These are best-effort guesses for words WordNet lacks, and the suspected defects below are the known residual of the opt-in.\n")
    md.extend(render(rows_guess, total, shown_in_full=False))
    md.append(TRIAGE)
    (HERE / "expected-lemma.tsv").write_text(tsv_lemma, encoding="utf-8")
    md.extend(render_lemma(rows_lemma, info, dict_sha))
    (HERE / "DIFFERENCES.md").write_text("\n".join(md), encoding="utf-8")
    print("wrote %s (%d rows), %s (%d rows), expected-lemma.tsv (%d rows), DIFFERENCES.md" % (exp, len(rows), exp_guess, len(rows_guess), len(rows_lemma)))


LEMMA_BUCKET_ORDER = ["capitalised", "curated", "data-version", "order", "min-stem", "nltk-exception",
                      "extra-rule", "bug"]


def naive_forms(lemma, pos):
    """Regular inflections of a lemma, spelled the way English does it (no dictionary)."""
    out = set()
    v = "aeiou"
    # Doubling is regular English only for a one-syllable stem ("stop" -> "stopped");
    # a longer stem doubles by stress ("benefit" -> "benefited"), which no list captures.
    cvc = (len(lemma) >= 3 and lemma[-1] not in v + "wxy" and lemma[-2] in v and lemma[-3] not in v
           and len(re.findall(r"[aeiouy]+", lemma)) == 1)
    if pos == "v":
        if lemma.endswith(("s", "x", "z", "ch", "sh", "o")):
            out.add(lemma + "es")
        elif lemma.endswith("y") and len(lemma) > 1 and lemma[-2] not in v:
            out.add(lemma[:-1] + "ies")
        else:
            out.add(lemma + "s")
        suffixes = (("ed", "ing"),)
    else:
        suffixes = (("er", "est"),)
    for suf in suffixes[0]:
        if lemma.endswith("e") and suf[0] == "e" and pos == "a":
            out.add(lemma + suf[1:])
        if lemma.endswith("e") and suf[0] in "ei":
            out.add(lemma[:-1] + suf)
            if suf[0] == "e":
                out.add(lemma + suf[1:])
        if lemma.endswith("y") and len(lemma) > 1 and lemma[-2] not in v:
            out.add(lemma[:-1] + "i" + suf)
        if cvc:
            out.add(lemma + lemma[-1] + suf)
        out.add(lemma + suf)
    return out


def lemma_words(wn, verbs, adjs):
    """The word list of the lemma mode and its sources, for the report."""
    data = (ROOT / "src/data.rs").read_text(encoding="utf-8")
    vf = [a for a, _ in first_strings(rust_block(data, "VERB_FORMS"))]
    af = [a for a, _ in first_strings(rust_block(data, "ADJECTIVE_FORMS"))]
    nv = set()
    for w in verbs:
        nv |= naive_forms(w, "v")
    na = set()
    for w in adjs:
        na |= naive_forms(w, "a")
    return vf, af, nv, na


def extra_rule_shape(w, m, pos):
    """Whether moult's answer m for w is what one of the rules morphy lacks gives:
    -ied / -ying (verbs), -ier / -iest (adjectives), or an undoubled final
    consonant before -ed / -ing (verbs) or -er / -est (adjectives)."""
    sufs = ("ed", "ing") if pos == "v" else ("er", "est")
    if pos == "v" and w.endswith(("ied", "ying")):
        return True
    if pos == "a" and w.endswith(("ier", "iest")):
        return True
    for suf in sufs:
        if w.endswith(suf):
            stem = w[: -len(suf)]
            if len(stem) >= 2 and stem[-1] == stem[-2] and stem[-1] not in "aeiou" and m == stem[:-1]:
                return True
    return False


def lemma_diff(wn, lem, dict_words):
    """Every word where moult's and NLTK's lemma differ, as (word, pos, moult, nltk, bucket)."""
    verbs = read_lines(ROOT / "src/verbs.txt")
    adjs = read_lines(ROOT / "src/adjectives.txt")
    verb_names = set(read_lines(ROOT / "src/verb_names.txt"))
    adj_names = set(read_lines(ROOT / "src/adjective_names.txt"))
    pol = (ROOT / "src/policy.rs").read_text(encoding="utf-8")
    choice = {}
    for f, p, l in re.findall(r'\("([^"]+)", PartOfSpeech::(\w+), "([^"]+)"\)', rust_block_any(pol, "LEMMA_CHOICE")):
        choice[(f, p)] = l
    vf, af, nv, na = lemma_words(wn, verbs, adjs)
    srcs = [verbs, adjs, vf, af, sorted(nv), sorted(na), dict_words]
    allw = set()
    for src in srcs:
        allw |= set(src)
    words = sorted(w for w in allw if w == w.lower() and "\t" not in w and " " not in w)
    mv = inflect_lemma(words)
    cand = inflect_lemma(words, candidates=True)

    pwn = {}
    pwn_names = {}
    for pos, key in (("v", "v"), ("a", "a")):
        cased = {l.name() for ss in wn.all_synsets(key) for l in ss.lemmas()}
        common = {n for n in cased if n == n.lower()}
        pwn[pos] = common
        pwn_names[pos] = {n.lower() for n in cased} - common
    wn.ensure_loaded()
    exceptions = wn._exception_map
    moult_common = {"v": set(verbs), "a": set(adjs)}
    moult_names = {"v": verb_names, "a": adj_names}

    def bucket(w, pos, m, n, cands):
        ptag = "Verb" if pos == "v" else "Adjective"
        # 1. capitalised: a lemma written only with capitals (known, never a rule target).
        for x in (w, m, n):
            if (x in moult_names[pos] and x not in moult_common[pos]) or (x in pwn_names[pos] and x not in pwn[pos]):
                return "capitalised"
        # 2. curated: LEMMA_CHOICE decides the form.
        if (w, ptag) in choice:
            return "curated"
        # 3. data-version: membership of the word or an answer differs between the lists.
        for x in (w, m, n):
            if (x in moult_common[pos]) != (x in pwn[pos]):
                return "data-version"
        # 4. order: both answers are lemmas of the word and NLTK takes the shortest
        #    (hating: hate or hat; canvass: canvass or canvas); moult follows its
        #    stated order.
        if n in cands or (m in pwn[pos] and n in pwn[pos] and len(n) < len(m)):
            return "order"
        # 5. min-stem: NLTK reaches a lemma of fewer than three characters, which
        #    moult's rules never emit.
        if len(n) < 3 and n != w:
            return "min-stem"
        # 6. nltk-exception: the word is on NLTK's exception list for the part of
        #    speech (163 adjectives map to themselves), which stops its rules,
        #    and moult's answer is a lemma of at least three characters (a
        #    shorter stem is an accidental reading, not evidence).
        if n == w and w in exceptions[pos] and len(m) >= 3 and m != w:
            return "nltk-exception"
        # 7. extra-rule: moult's rules (-ied, -ier/-iest, -ying, a doubled consonant)
        #    find a lemma that morphy's rule set misses. The word must have the
        #    shape of one of those rules and moult's answer the matching stem;
        #    anything else is unexplained and stays in bug.
        if n == w and m != w and extra_rule_shape(w, m, pos):
            return "extra-rule"
        return "bug"

    rows = []
    for w in words:
        (mvv, mva), (cv, ca) = mv[w], cand[w]
        for pos, m, cands in (("v", mvv, cv), ("a", mva, ca)):
            n = lem.lemmatize(w, pos)
            if m != n:
                rows.append((w, pos, m, n, bucket(w, pos, m, n, cands)))
    rows.sort()
    names = ["src/verbs.txt", "src/adjectives.txt", "VERB_FORMS forms", "ADJECTIVE_FORMS forms",
             "naive inflections of each verb lemma (-s/-es, -ed, -ing)", "naive inflections of each adjective lemma (-er, -est)",
             "/usr/share/dict/words, lower-cased"]
    return rows, dict(sources=list(zip(names, [len(set(x)) for x in srcs])), total=len(words))


def rust_block_any(src, name):
    m = re.search(r"static %s: &\[[^\n]*\] = &\[\n(.*?)\n\];" % name, src, re.S)
    if not m:
        sys.exit("cannot find %s" % name)
    return m.group(1)


def lemma_tsv(rows, dict_sha):
    return DICT_HEADER + dict_sha + "\nword\tpos\tmoult\tnltk\tbucket\n" + "".join("\t".join(r) + "\n" for r in rows)


LEMMA_JUSTIFY = {
    "capitalised": "One of the word, moult's answer or NLTK's answer is a lemma written only with capitals (\"french\", \"irish\", \"romance\" as a name). moult knows such a lemma and lists it for itself, but no suffix rule ever reaches it; NLTK lower-cases its index and strips freely.",
    "curated": "The form is in LEMMA_CHOICE (src/policy.rs): a form with more than one reading where the crate states the answer (\"appalled\" -> \"appall\", \"fell\" -> \"fall\", \"outer\" -> \"outer\", \"best\" -> \"good\"). NLTK has no such choice and takes the shortest candidate or its exception list.",
    "data-version": "The word or an answer is a lemma in only one dictionary: Open English WordNet 2025 (moult) versus Princeton WordNet 3.0 (NLTK). Mostly rule results on words only OEWN has (\"twerk\", \"favorite\"), or the reverse. Neither is wrong; the dictionaries differ.",
    "order": "Both answers are lemmas of the word (moult lists NLTK's among its candidates, or both are PWN lemmas and NLTK's is shorter). NLTK returns the shortest of morphy's lemmas (\"hating\" -> \"hat\"); moult follows its stated order, the -e restoring rule before the bare stem (\"hating\" -> \"hate\"), and a word that is itself a lemma before a shorter stem (\"canvass\").",
    "min-stem": "NLTK reaches a lemma of one or two characters through a rule (\"98est\" -> \"98\", \"cest\" -> \"c\"); moult's adjective rules never emit fewer than three characters. These are spurious readings of non-words.",
    "nltk-exception": "The word is on NLTK's exception list for that part of speech (many adjectives map to themselves, e.g. \"archer\"), which stops morphy applying its rules, while moult's rules find a lemma of at least three characters (\"co-opted\" -> \"co-opt\", \"archer\" -> \"arch\"). The exception-list entry is the evidence; a shorter answer is not accepted here and lands in bug. The differences are in moult's favour in the sense that the stem is a lemma of the part of speech.",
    "extra-rule": "moult's rules that morphy lacks find a lemma and NLTK finds none: -ied (\"autopsied\" -> \"autopsy\"), -ying (\"retying\" -> \"retie\"), a doubled final consonant (\"stopped\" -> \"stop\", \"bigger\" -> \"big\"), and -ier / -iest for adjectives (\"happier\" -> \"happy\"). Most rows are adjectives because the word list holds the naive -ier / -iest of every -y adjective, many of them non-words (\"accessarier\"): the rule is checked against the dictionary, the generator is not.",
}


def render_lemma(rows, info, dict_sha):
    """Markdown for the lemma mode: word list, bucket counts, a sample per bucket."""
    by = defaultdict(list)
    for r in rows:
        by[r[4]].append(r)
    total = info["total"]
    md = ["## Lemma mode (`--lemma`)\n"]
    md.append("moult's `lemmatize(w, Verb)` and `lemmatize(w, Adjective)` (`cargo xtask-inflect --lemma`) against `WordNetLemmatizer().lemmatize(w, \"v\")` and `(w, \"a\")`. NLTK returns the shortest of morphy's lemmas, or the word when it finds none. Each word is compared once per part of speech; a disagreement is one row of `expected-lemma.tsv` (`word, pos, moult, nltk, bucket`, `v` or `a`). Adverbs are not compared: NLTK's morphy has no adverb rules. The nouns are `singularize` and are covered above.\n")
    md.append("| source | words |\n|---|---|")
    for name, n in info["sources"]:
        md.append("| %s | %d |" % (name, n))
    md.append("| union, lower-case words compared | %d |\n" % total)
    md.append("The naive inflections add the regular -s/-es, -ed, -ing (and -er, -est) forms of every lemma, dropping a final e, changing y to i, and doubling the final consonant of a one-syllable consonant-vowel-consonant stem. Many are non-words; they probe rule behaviour, not usage. The word list includes `/usr/share/dict/words` (sha256 `%s`), recorded in `expected-lemma.tsv` as for the noun modes.\n" % dict_sha)
    agree = 2 * total - len(rows)
    md.append("Agreement: %d / %d = %.2f%% of word and part-of-speech pairs. Disagreements: %d (verbs %d, adjectives %d).\n" % (agree, 2 * total, 100.0 * agree / (2 * total), len(rows), sum(1 for r in rows if r[1] == "v"), sum(1 for r in rows if r[1] == "a")))
    md.append("| bucket | verbs | adjectives | total |\n|---|---|---|---|")
    order = [b for b in LEMMA_BUCKET_ORDER if b in by] + sorted(b for b in by if b not in LEMMA_BUCKET_ORDER)
    for b in LEMMA_BUCKET_ORDER:
        v = sum(1 for r in by.get(b, []) if r[1] == "v")
        md.append("| %s | %d | %d | %d |" % (b, v, len(by.get(b, [])) - v, len(by.get(b, []))))
    md.append("")
    md.append("The rules apply in this order (`bucket()` in `lemma_diff`): capitalised, curated, data-version, order, min-stem, nltk-exception, extra-rule; the first match wins and `bug` is the residue.\n")
    rng = random.Random(0)
    for b in order:
        if b == "bug":
            continue
        md.append("#### %s (%d)\n" % (b, len(by[b])))
        md.append(LEMMA_JUSTIFY.get(b, "") + "\n")
        sample = sorted(rng.sample(by[b], min(10, len(by[b]))))
        md.append("| word | pos | moult | nltk |\n|---|---|---|---|")
        for w, p, m, n, _ in sample:
            md.append("| %s | %s | %s | %s |" % (w, p, m, n))
        md.append("")
    md.append("#### bug (%d)\n" % len(by["bug"]))
    md.append("Unexplained residue: every row, none judged intentional.\n")
    md.append("| word | pos | moult | nltk |\n|---|---|---|---|")
    for w, p, m, n, _ in by["bug"]:
        md.append("| %s | %s | %s | %s |" % (w, p, m, n))
    md.append("")
    return md


def render(rows, total, shown_in_full):
    """Markdown for one mode: totals, a bucket table, a sample per bucket, and the bug residue."""
    by = defaultdict(list)
    for r in rows:
        by[r[3]].append(r)
    agree = total - len(rows)
    rng = random.Random(0)
    md = []
    md.append("NLTK %s, WordNet 3.0. Agreement: %d / %d = %.2f%%. Disagreements: %d.\n" % (NLTK_PIN, agree, total, 100.0 * agree / total, len(rows)))
    md.append("| bucket | count |\n|---|---|")
    order = [b for b in BUCKET_ORDER if b in by] + sorted(b for b in by if b not in BUCKET_ORDER)
    for b in order:
        md.append("| %s | %d |" % (b, len(by[b])))
    md.append("")
    for b in order:
        if b == "bug":
            continue
        md.append("#### %s (%d)\n" % (b, len(by[b])))
        if shown_in_full:
            md.append(JUSTIFY.get(b, "") + "\n")
        sample = sorted(rng.sample(by[b], min(10, len(by[b]))))
        md.append("| word | moult | nltk |\n|---|---|---|")
        for w, m, n, _ in sample:
            md.append("| %s | %s | %s |" % (w, m, n))
        md.append("")
    md.append("#### bug (%d)\n" % len(by["bug"]))
    md.append("Unexplained residue: every row, none judged intentional. The ordered rules are broad, so an empty bucket does not mean every row is right.\n")
    md.append("| word | moult | nltk |\n|---|---|---|")
    for w, m, n, _ in by["bug"]:
        md.append("| %s | %s | %s |" % (w, m, n))
    md.append("")
    return md


TRIAGE = """## Suspected defects in the opt-in fallback (known residual)

These apply only with `guess_unknown(true)`; the default mode never guesses, so
none of them can occur there (the default-mode `fallback-*` buckets for the
unchecked rules are empty). Rows where the opt-in fallback produces a string that
is not an OEWN noun are bucketed as `fallback-*`, because the opt-in is
documented policy.

The tightened fallback (it now refuses `-as -is -us -ss -os -ys -ics -ities -wards
-selves -ops`, a vowel + `-ses`, `-sses`, `-etes`, the taxon endings `-ales -iformes
-ides`, "facies", and compounds of an invariant noun) removed 1280 rows from the
previous run (49 + 28 + 1512 + 927 before; 30 + 14 + 727 + 465 now in the four
fallback buckets). Fixed, and no longer listed:

- `-ics`, `-ops`, `-wards` words ("backwards", "afterwards", "echinops").
- `-os` and `-ys` Greek singulars and genus names ("tholos", "xyrichthys").
- `-ales`, `-iformes`, `-ides` orders and taxa ("agaricales").
- `-ities` mass nouns ("calvities", "nigrities"), "facies" and compounds of
  "species" and "series" ("superspecies", "subseries").
- Names in `-ses` and `-sses` ("moses", "ulysses", "rameses"), and `-etes` ("kubernetes").

Cost of the tightening: about 1280 unknown words that NLTK also leaves alone are
now left alone by moult too. Reading the removed rows, nearly all were non-plurals
(taxa, Greek and Latin singulars, adverbs); the genuine regular plurals now
unchanged are a handful of compounds such as "blooddrops", "beechdrops" and
"pinedrops". The 8 new rows (a word NLTK maps to a name or a PWN noun, moult now
keeps) are "broadways", "demotics", "galways", "germanics", "saccos", "semitics",
"thos" and "oses".

The known-head check (a fallback reduction is accepted only when the shortened
word ends in an OEWN noun of at least three letters, the whole word or a proper
suffix) then removed a further 634 rows from the four fallback buckets (30 + 14 +
727 + 465 before; 4 + 5 + 318 + 275 now) and fixed the non-words that were listed
here: "abies" -> "aby", "colluvies", "balbuties", "hirsuties", "antirabies",
"consocies", "macies" (the Latin `-ies` singulars); "lichenes", "gekkones",
"ascolichenes", "clavicornes", "dicotyledones" (Latin `-es` plurals);
"laches" -> "lach", "natchitoches", "albanenses". Nine rows moved into
`nltk-exception-only` and `fallback-other` (moult now leaves a word unchanged
that NLTK reduces).

The head check was then tightened again: a reduction is accepted only when the
whole result is an OEWN noun, or a proper suffix of at least four letters is a
noun with at least two letters before it (a plausible compound boundary). This
removed the short-head false positives ("elaps" -> "elap" via "lap", "perhaps"
-> "perhap" via "hap") and moved the four fallback buckets from 4 + 5 + 313 +
269 to 2 + 4 + 208 + 150 rows.

Still suspected, and not fixed (the opt-in is best-effort for words WordNet lacks):

- A compound whose last part is a noun reduces to a non-word: "afterhours" ->
  "afterhour", "belowstairs" -> "belowstair". The default mode leaves both alone
  (`tests/cases.rs`); the opt-in output is pinned as a known limitation in
  `tests/guess.rs`.
- The head check is weak where a longer noun happens to end the word: the
  reduction keeps a non-word whose last four or more letters are a noun
  ("alabastrites" -> "alabastrite" via "rite"), so `fallback-s-after-e` can
  still hold a Latin plural or a mineral name whose reduction ends in a
  four-letter noun. A sample of 45 of the 422 remaining fallback rows shows
  about a dozen non-words of that shape ("aerogene", "raptore", "traditore",
  "carnifice", "perates", "helode", "impenne", "johnadream", "chanking").
- A handful of `-ies` words ("andries" -> "andry", "manyplies" -> "manyply",
  "outboundaries", "sherryvallies") and `-ches` / `-shes` compounds
  ("carritches", "gamashes") reduce to a string that is not a noun but ends
  in one.
- `min-stem` blocks reductions to real two-letter nouns ("ads", "ids", "pis")
  because of the 3-letter floor: a deliberate cost of the floor.
"""

BUCKET_ORDER = ["invariant", "policy-choice", "irregular-form", "names", "numeric",
                "extra-rule", "min-stem", "nltk-proper-name", "data-version",
                "class-A-strip-wins", "nltk-shortest-candidate", "fallback-ies", "fallback-es", "fallback-s-after-e", "fallback-s",
                "nltk-exception-only", "fallback-other", "nltk-morphy-quirk", "different-lemma", "bug"]

JUSTIFY_HEAD = """# NLTK differential

Dev-only comparison of moult's `singularize` against NLTK's
`WordNetLemmatizer().lemmatize(w, "n")` on Princeton WordNet 3.0. It exists to
find moult bugs, not to match NLTK: NLTK's lemmatizer applies detachment rules and
returns the shortest candidate that is in WordNet, with no policy.

## How to run

From the repository root (the first run downloads WordNet 3.0 into the
git-ignored `tools/nltk-diff/.cache/`):

    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py            # rewrite expected.tsv, expected-guess.tsv, expected-lemma.tsv and this file
    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --guess    # rewrite only expected-guess.tsv
    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --check    # fail if any of the three expected files would change
    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --guess --check
    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --lemma    # rewrite only expected-lemma.tsv (verbs and adjectives)
    uv run --with nltk==3.9.2 python -I tools/nltk-diff/diff.py --lemma --check

moult is called through `cargo xtask-inflect` (words on stdin, TSV on stdout),
and with `--guess` for the opt-in mode. `expected.tsv` is the default mode, the
guarantee: `singularize` never guesses. `expected-guess.tsv` is
`Inflector::builder().guess_unknown(true)`, so the opt-in residual is
reproducible and reviewable.
Only lower-case words are compared. Each disagreement lands in exactly one
bucket; the rules apply in the order the buckets are listed below
(`bucket()` in `diff.py`). `expected.tsv` holds every disagreement as
`word, moult, nltk, bucket`; a change in moult behaviour shows up as a diff
there. The buckets below are described once, under the default mode; the opt-in
section lists the same buckets plus the `fallback-*` ones.
"""

JUSTIFY = {
    "invariant": "The word is on moult's closed INVARIANT list (zero plurals, mass nouns, fields of study in -ics). Policy is to leave it alone; NLTK strips the final s whenever the result is in WordNet.",
    "policy-choice": "A word with more than one reading where the crate states a policy: SINGULAR_CHOICE (\"leaves\" -> \"leaf\", \"cookies\" -> \"cookie\"), MAN_REGULAR nouns (\"human\" is not a -man plural), and the checked -men/-man and -ses/-sis rules NLTK does not have. NLTK has no such choice and takes the shortest candidate.",
    "names": "Words in moult's names list that are not common nouns. moult treats them as already singular names; NLTK has no name list and may strip a final s.",
    "min-stem": "moult refuses a rule output shorter than three characters (\"as\" is not the plural of \"a\"); NLTK has no such floor.",
    "data-version": "(Compared against common lower-case lemmas only; capitalised PWN lemmas are tracked separately.) One side's answer exists only in one dictionary: Open English WordNet 2025 (moult) versus Princeton WordNet 3.0 (NLTK). Neither is wrong; the dictionaries differ.",
    "class-A-strip-wins": "The word is itself an OEWN noun but is also the regular plural of another noun (\"glasses\"-style ambiguity); moult strips when the stripped form is a noun, by the crate's resolution order, NLTK keeps the word.",
    "nltk-shortest-candidate": "The word is a noun and moult keeps it, but its stripped form is also a noun and NLTK's lemmatize() returns the shortest candidate. moult only strips through checked rules after the curated lists; keeping a known word is the conservative result.",
    "fallback-ies": "Opt-in mode only. Unknown word ending in -ies after a consonant (and not -ities): moult's conservative unchecked rule gives -y; NLTK leaves unknown words unchanged. Right for regular plurals, wrong for the remaining Latin -ies singulars (see the triage list).",
    "fallback-es": "Opt-in mode only. Unknown word ending in s/x/z/ch/sh + es: moult drops the es; NLTK leaves it unchanged.",
    "fallback-s-after-e": "Opt-in mode only. Unknown word ending in -s after e: moult drops the s; NLTK leaves it unchanged. Mostly regular plurals of words WordNet lacks (\"microservices\") but also some Latin plurals; the rule is documented policy (the unknown-word step of the crate docs) and the main remaining source of non-word outputs.",
    "fallback-s": "Opt-in mode only. Unknown word ending in -s after a consonant: moult drops the s; NLTK leaves it unchanged. The documented opt-in fallback.",
    "nltk-exception-only": "NLTK reaches a PWN noun through its exception list (or a detachment chain) that moult's dictionary and rules do not; moult leaves the unknown word unchanged, the conservative result.",
    "fallback-other": "Remaining disagreements on words moult's dictionary does not settle: moult keeps the word (or, in the opt-in mode, lightly strips it), NLTK answers with a PWN name or an exception-list lemma.",
    "irregular-form": "A Latin/Greek or other irregular plural that OEWN lists as a form of a lemma (or the IRREGULAR supplement: person/people, bacterium/bacteria); NLTK's exception list lacks the pair and returns the word unchanged.",
    "numeric": "Number plurals such as \"80s\": moult leaves a token with digits unchanged (its fallback needs letters); NLTK strips the s.",
    "extra-rule": "The checked -men -> -man and -ses -> -sis rules (\"businessmen\", \"fibroses\"); NLTK has a -men rule only through its exception list and no -ses -> -sis rule.",
    "nltk-proper-name": "NLTK answers with a lemma that PWN writes only with capitals; lemmatize() lower-cases and so returns a name.",
    "fallback": "The word is in neither dictionary. moult applies its conservative unchecked rules (-ies after a consonant, -s after a consonant, ...); NLTK returns unknown words unchanged or applies detachment rules that need a dictionary hit.",
    "nltk-morphy-quirk": "NLTK's output is not a WordNet noun or is an artefact of its detachment rules and exception lists (it returns the word's own entry when no candidate hits). moult only outputs dictionary-checked nouns.",
    "different-lemma": "Both outputs are WordNet nouns but different ones; the word has more than one plausible reduction and moult's order of rules differs from morphy's shortest-candidate choice.",
}

if __name__ == "__main__":
    main()
