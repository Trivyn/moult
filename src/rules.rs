//! The suffix rules, with no data. Every function here is pure: the caller
//! supplies the dictionary check, so the same code runs in the crate and in
//! the data generator that audits it.

use alloc::string::String;
use alloc::vec::Vec;

/// The fewest characters a singular candidate may have. It keeps `os` from
/// becoming `o` and `pas` from becoming `pa`.
const MIN_SINGULAR_LEN: usize = 3;

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// The regular plural of `lemma`: consonant + `y` becomes `ies`; a word
/// ending in `s`, `x`, `z`, `ch` or `sh` takes `es`; anything else takes `s`.
/// Words in `-o` and `-f` take a plain `s`; `-oes` and `-ves` plurals come
/// only from WordNet's form list.
pub(crate) fn regular_plural(lemma: &str) -> String {
    let mut out = String::from(lemma);
    let mut rev = lemma.chars().rev();
    match (rev.next(), rev.next()) {
        (Some('y'), Some(before)) if !is_vowel(before) => {
            out.pop();
            out.push_str("ies");
        }
        (Some('s' | 'x' | 'z'), _) | (Some('h'), Some('c' | 's')) => out.push_str("es"),
        _ => out.push('s'),
    }
    out
}

/// The third person singular of `lemma`: consonant + `y` becomes `ies`;
/// `s`, `x`, `z`, `ch` or `sh` takes `es`; consonant + `o` takes `es` when
/// `o_es` is set; anything else takes `s`.
pub(crate) fn third_person(lemma: &str, o_es: bool) -> String {
    if o_es {
        if let Some(stem) = lemma.strip_suffix('o') {
            if stem.chars().last().is_some_and(|c| !is_vowel(c)) {
                return String::from(lemma) + "es";
            }
        }
    }
    regular_plural(lemma)
}

/// The fewest characters a word ending in `-sis` needs for the `-ses` plural.
/// It leaves `sis` itself and short words like `isis` to the regular rule.
const MIN_SIS_LEN: usize = 5;

/// Whether the `-sis` to `-ses` plural applies to `lemma`.
pub(crate) fn is_sis_word(lemma: &str) -> bool {
    lemma.len() >= MIN_SIS_LEN && lemma.ends_with("sis")
}

/// Whether `key` ends in `s` but not `ss`, and is not a `-sis` word the plural
/// rules handle.
pub(crate) fn ends_in_bare_s(key: &str) -> bool {
    key.ends_with('s') && !key.ends_with("ss") && !is_sis_word(key)
}

/// Whether `key` has the shape of a plural noun: it ends in a bare `s` (see
/// [`ends_in_bare_s`]) after at least one character, anything but `a`, `i`, `o`, `u` or `y`. The
/// excluded endings are the Latin and Greek singulars of "bus", "status",
/// "iris", "canvas" and "-itis"; after a consonant, `e`, a digit or an
/// apostrophe the `s` is nearly always a plural ending ("thanks", "clothes",
/// "1990s").
pub(crate) fn has_plural_shape(key: &str) -> bool {
    ends_in_bare_s(key)
        && key
            .strip_suffix('s')
            .is_some_and(|stem| !stem.is_empty() && !stem.ends_with(['a', 'i', 'o', 'u', 'y']))
}

/// The plural of `lemma` when no list and no WordNet form decides it, in rule
/// order:
///
/// 1. `-man` to `-men` (policeman, fireman) for a known noun that is not in
///    `man_regular`, the nouns whose `man` is not the word (human, shaman)
/// 2. `-ch` to `-chs` (stomachs, monarchs) for a word in `ch_regular`, the
///    nouns whose `ch` is not the sound in church
/// 3. `-sis` to `-ses` (mitosis, apoptosis) for any word of at least five
///    characters, known or not
/// 4. [`regular_plural`]
pub(crate) fn plural(
    lemma: &str,
    is_known_noun: impl Fn(&str) -> bool,
    man_regular: impl Fn(&str) -> bool,
    ch_regular: impl Fn(&str) -> bool,
) -> String {
    if let Some(stem) = lemma.strip_suffix("man") {
        if is_known_noun(lemma) && !man_regular(lemma) {
            return String::from(stem) + "men";
        }
    }
    if lemma.ends_with("ch") && ch_regular(lemma) {
        return String::from(lemma) + "s";
    }
    if is_sis_word(lemma) {
        return String::from(&lemma[..lemma.len() - 3]) + "ses";
    }
    regular_plural(lemma)
}

/// The singular candidates for `word`, in rule order. Every candidate has at
/// least three characters. Nothing here checks a dictionary:
///
/// 1. `-ies` to `-ie` (cookies, movies)
/// 2. `-ies` to `-y` (cities)
/// 3. `-ses`, `-xes`, `-zes`, `-ches`, `-shes` to the stem (boxes, churches)
/// 4. `-s` to the stem, except after `ss` (cats)
/// 5. `-oes` to `-o` (heroes)
/// 6. `-ves` to `-f`, then to `-fe` (wolves, knives)
/// 7. `-men` to `-man` (policemen, women)
/// 8. `-ses` to `-sis` (mitoses, theses)
///
/// The ordinary strips come first, so a word both rules reach resolves to the
/// plain strip ("ellipses" to "ellipse"); the rarer rewrites 7 and 8 come
/// last. Where two nouns do compete, the policy lists settle the word before
/// the rules run. Rules 7 and 8 need a non-empty stem, so `men` and `ses`
/// alone never fire.
#[allow(dead_code)] // the generator and the tests use the Vec form
pub(crate) fn singular_candidates(word: &str) -> Vec<String> {
    let mut out = Vec::new();
    singular_stems(word, &mut |c| out.push(String::from(c)));
    out
}

/// [`singular_candidates`] without the allocation: each candidate is built in
/// a stack buffer and handed to `f`, in rule order. A candidate longer than the
/// buffer (a word of more than 64 bytes) is built on the heap.
pub(crate) fn singular_stems(word: &str, f: &mut dyn FnMut(&str)) {
    let mut push = |stem: &str, tail: &str| {
        if stem.chars().count() + tail.len() < MIN_SINGULAR_LEN {
            return;
        }
        let len = stem.len() + tail.len();
        if len > MAX_STEM_WORD + 2 {
            let mut s = String::from(stem);
            s.push_str(tail);
            f(&s);
            return;
        }
        let mut buf = [0u8; MAX_STEM_WORD + 2];
        buf[..stem.len()].copy_from_slice(stem.as_bytes());
        buf[stem.len()..len].copy_from_slice(tail.as_bytes());
        // Both parts are valid UTF-8 and the cut is between them.
        if let Ok(s) = core::str::from_utf8(&buf[..len]) {
            f(s);
        }
    };
    if let Some(stem) = word.strip_suffix("ies") {
        push(stem, "ie");
        push(stem, "y");
    }
    for tail in ["ses", "xes", "zes", "ches", "shes"] {
        if word.ends_with(tail) {
            push(&word[..word.len() - 2], "");
        }
    }
    if word.ends_with('s') && !word.ends_with("ss") {
        push(&word[..word.len() - 1], "");
    }
    if let Some(stem) = word.strip_suffix("oes") {
        push(stem, "o");
    }
    if let Some(stem) = word.strip_suffix("ves") {
        push(stem, "f");
        push(stem, "fe");
    }
    if let Some(stem) = word.strip_suffix("men") {
        if !stem.is_empty() {
            push(stem, "man");
        }
    }
    if let Some(stem) = word.strip_suffix("ses") {
        if !stem.is_empty() {
            push(stem, "sis");
        }
    }
}

/// The first candidate of [`singular_stems`] that `is_noun` accepts.
/// A `-men` word never reduces to a noun in `man_regular`: those nouns take
/// `-s`, so "dolmen" is not the plural of "dolman" and "limen" is not the
/// plural of "liman".
pub(crate) fn checked_singular(
    word: &str,
    is_noun: impl Fn(&str) -> bool,
    man_regular: impl Fn(&str) -> bool,
) -> Option<String> {
    let men = word.ends_with("men");
    let mut found = None;
    singular_stems(word, &mut |c| {
        if found.is_none() && is_noun(c) && !(men && man_regular(c)) {
            found = Some(String::from(c));
        }
    });
    found
}

/// The longest word, in bytes, the stem rules below accept. It matches the
/// stack buffer they build candidates in; a longer word, or one that is not
/// ASCII, yields no stems.
const MAX_STEM_WORD: usize = 64;

/// The fewest characters a verb lemma candidate may have. The noun and
/// adjective rules use [`MIN_SINGULAR_LEN`]; the verbs "be", "do" and "go"
/// are two letters long and have no listed form for "being", "doing",
/// "going" or "does".
const MIN_VERB_STEM_LEN: usize = 2;

/// Whether `stem` ends in the same consonant letter twice ("stopp", "runn").
fn ends_doubled_consonant(stem: &str) -> bool {
    match stem.as_bytes() {
        [.., a, b] => a == b && a.is_ascii_lowercase() && !is_vowel(char::from(*a)),
        _ => false,
    }
}

/// Hand `f` the candidate `stem` + `tail` when it has at least
/// [`MIN_SINGULAR_LEN`] characters. Both parts are ASCII.
fn emit(min: usize, stem: &str, tail: &str, f: &mut dyn FnMut(&str)) {
    let len = stem.len() + tail.len();
    if !(min..=MAX_STEM_WORD + 2).contains(&len) {
        return;
    }
    let mut buf = [0u8; MAX_STEM_WORD + 2];
    buf[..stem.len()].copy_from_slice(stem.as_bytes());
    buf[stem.len()..len].copy_from_slice(tail.as_bytes());
    if let Ok(s) = core::str::from_utf8(&buf[..len]) {
        f(s);
    }
}

/// The fewest letters the stripped stem needs before an `e` is restored
/// (`-es`, `-ed`, `-ing` to `-e` for verbs; `-er`, `-est` to `-e` for
/// adjectives). A one-letter stem would only invent "be" from "bed" or
/// "bing". The `-ying` to `-ie` rule is exempt: "dying", "tying" and "vying"
/// have a one-letter stem and are the point of it.
const MIN_E_RESTORING_STEM_LEN: usize = 2;

/// Emit the verb candidate `stem` + `e`, unless the stem is shorter than
/// [`MIN_E_RESTORING_STEM_LEN`].
fn emit_e(stem: &str, f: &mut dyn FnMut(&str)) {
    if stem.len() >= MIN_E_RESTORING_STEM_LEN {
        emit(MIN_VERB_STEM_LEN, stem, "e", f);
    }
}

/// Emit `stem` as the bare stem, then, when it ends in a doubled consonant,
/// the stem with the double removed ("stopp" then "stop").
fn emit_bare(min: usize, stem: &str, f: &mut dyn FnMut(&str)) {
    emit(min, stem, "", f);
    if ends_doubled_consonant(stem) {
        emit(min, &stem[..stem.len() - 1], "", f);
    }
}

/// The verb lemma candidates for `word`, passed to `f` in rule order. Every
/// candidate has at least two characters, so that "going", "doing" and
/// "being" reach "go", "do" and "be"; the one spurious two-letter reading
/// this admits is blocked in the e-restoring rules (2, 4 and 5), which need a
/// stem of at least [`MIN_E_RESTORING_STEM_LEN`] letters before the `e`, so
/// "bed" and "bing" do not reach "be" while "being" does (by the bare stem).
/// Nothing here checks a dictionary:
///
/// 1. `-ies` to `-y` (carries), `-ied` to `-y` (carried)
/// 2. `-es` to `-e` (hates), `-es` to the stem (watches)
/// 3. `-s` to the stem, except after `ss` (runs)
/// 4. `-ed` to `-e` (hated), `-ed` to the stem (wanted), then the stem with
///    a doubled final consonant undoubled (stopped to stop)
/// 5. `-ing` to `-e` (hating), `-ing` to the stem (wanting), then the
///    undoubled stem (running to run)
/// 6. `-ying` to `-ie` (dying)
///
/// A word that is not ASCII, or longer than 64 bytes, yields nothing.
pub(crate) fn verb_stems(word: &str, f: &mut dyn FnMut(&str)) {
    if !word.is_ascii() || word.len() > MAX_STEM_WORD {
        return;
    }
    if let Some(stem) = word.strip_suffix("ies") {
        emit(MIN_VERB_STEM_LEN, stem, "y", f);
    }
    if let Some(stem) = word.strip_suffix("ied") {
        emit(MIN_VERB_STEM_LEN, stem, "y", f);
    }
    if let Some(stem) = word.strip_suffix("es") {
        emit_e(stem, f);
        emit(MIN_VERB_STEM_LEN, stem, "", f);
    }
    if word.ends_with('s') && !word.ends_with("ss") {
        emit(MIN_VERB_STEM_LEN, &word[..word.len() - 1], "", f);
    }
    if let Some(stem) = word.strip_suffix("ed") {
        emit_e(stem, f);
        emit_bare(MIN_VERB_STEM_LEN, stem, f);
    }
    if let Some(stem) = word.strip_suffix("ing") {
        emit_e(stem, f);
        emit_bare(MIN_VERB_STEM_LEN, stem, f);
    }
    if let Some(stem) = word.strip_suffix("ying") {
        emit(MIN_VERB_STEM_LEN, stem, "ie", f);
    }
}

/// The adjective lemma candidates for `word`, passed to `f` in rule order.
/// Every candidate has at least three characters, and nothing here checks a
/// dictionary:
///
/// 1. `-ier` and `-iest` to `-y` (happier, happiest)
/// 2. `-er` and `-est` to `-e` (later, latest), for a stem of at least
///    [`MIN_E_RESTORING_STEM_LEN`] letters (three characters in all, so the
///    stem rule of the shared minimum already covers it)
/// 3. `-er` and `-est` to the stem (taller, tallest), then the stem with a
///    doubled final consonant undoubled (bigger to big)
///
/// A word that is not ASCII, or longer than 64 bytes, yields nothing.
pub(crate) fn adjective_stems(word: &str, f: &mut dyn FnMut(&str)) {
    if !word.is_ascii() || word.len() > MAX_STEM_WORD {
        return;
    }
    if let Some(stem) = word.strip_suffix("ier") {
        emit(MIN_SINGULAR_LEN, stem, "y", f);
    }
    if let Some(stem) = word.strip_suffix("iest") {
        emit(MIN_SINGULAR_LEN, stem, "y", f);
    }
    if let Some(stem) = word.strip_suffix("er") {
        if stem.len() >= MIN_E_RESTORING_STEM_LEN {
            emit(MIN_SINGULAR_LEN, stem, "e", f);
        }
        emit_bare(MIN_SINGULAR_LEN, stem, f);
    }
    if let Some(stem) = word.strip_suffix("est") {
        if stem.len() >= MIN_E_RESTORING_STEM_LEN {
            emit(MIN_SINGULAR_LEN, stem, "e", f);
        }
        emit_bare(MIN_SINGULAR_LEN, stem, f);
    }
}
