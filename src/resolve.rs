//! The resolution steps shared by the free functions and [`Inflector`]. Each
//! function takes the lower-cased key of a word, and sometimes the word as
//! written, because the acronym and fallback rules read its case.
//!
//! [`Inflector`]: crate::Inflector

use alloc::borrow::Cow;
use alloc::string::String;

use crate::case::Origin;
use crate::lookup::{contains, rows, rows_by_lemma};
use crate::{data, policy, rules};

/// The outcome of the dictionary steps of singularize.
pub(crate) enum Sing {
    /// The word is its own singular.
    Keep,
    /// The word's singular, in lower case.
    To(Cow<'static, str>),
    /// WordNet lists the word only as another part of speech ("perhaps"):
    /// known, not a noun, and left unchanged.
    Other,
    /// No list, form, rule or table knows the word.
    Unknown,
}

pub(crate) fn is_noun_key(key: &str) -> bool {
    contains(data::NOUNS, key)
}

fn is_known(key: &str) -> bool {
    is_noun_key(key) || contains(data::NAMES, key)
}

pub(crate) fn is_invariant(key: &str) -> bool {
    policy::INVARIANT.binary_search(&key).is_ok()
}

fn is_excluded(lemma: &str, form: &str) -> bool {
    policy::EXCLUDED_FORMS.binary_search(&(lemma, form)).is_ok()
}

fn is_man_regular(word: &str) -> bool {
    policy::MAN_REGULAR.binary_search(&word).is_ok()
}

fn is_ch_regular(word: &str) -> bool {
    policy::CH_REGULAR.binary_search(&word).is_ok()
}

fn is_s_regular(key: &str) -> bool {
    policy::S_REGULAR.binary_search(&key).is_ok()
}

fn is_other(key: &str) -> bool {
    contains(data::OTHERS, key)
}

/// Singularize steps 3 to 7 for the lower-case `key`: the invariant list, the
/// irregular forms, the words WordNet lists only as another part of speech
/// (unchanged unless the plural of the noun a checked rule finds), the checked
/// rules, and the known words.
pub(crate) fn resolve(key: &str) -> Sing {
    if is_invariant(key) {
        return Sing::Keep;
    }
    // IRREGULAR holds a handful of pairs, so a scan by plural is cheaper than
    // a second sorted view.
    if let Some(&(singular, _)) = policy::IRREGULAR.iter().find(|&&(_, p)| p == key) {
        return Sing::To(Cow::Borrowed(singular));
    }
    if let Ok(i) = policy::SINGULAR_CHOICE.binary_search_by(|&(k, _)| k.cmp(key)) {
        let out = policy::SINGULAR_CHOICE[i].1;
        return if out == key {
            Sing::Keep
        } else {
            Sing::To(Cow::Borrowed(out))
        };
    }
    // Rows are (form, lemma). The generator guarantees that the policy lists
    // settle a form with several lemmas, so the survivors agree.
    let mut lemmas = rows(data::FORM_TO_LEMMA, key)
        .iter()
        .filter(|&&(form, lemma)| !is_excluded(lemma, form))
        .map(|&(_, lemma)| lemma);
    if let Some(lemma) = lemmas.next() {
        debug_assert!(lemmas.all(|l| l == lemma), "unsettled form {key:?}");
        return if lemma == key {
            Sing::Keep
        } else {
            Sing::To(Cow::Borrowed(lemma))
        };
    }
    let reduced = rules::checked_singular(key, is_noun_key, is_man_regular);
    // A word WordNet lists only as another part of speech stays unchanged,
    // unless it is the plural of the noun a checked rule finds ("bats" of
    // "bat", but "alas" is not the plural of "ala").
    if is_other(key) && !is_known(key) {
        return match reduced {
            Some(noun) if plural(&noun).0 == key => Sing::To(Cow::Owned(noun)),
            _ => Sing::Other,
        };
    }
    if let Some(lemma) = reduced {
        return Sing::To(Cow::Owned(lemma));
    }
    if is_known(key) {
        Sing::Keep
    } else {
        Sing::Unknown
    }
}

/// The length of the all-caps stem of an acronym plural, or `None`: an
/// upper-case ASCII stem of at least two characters with at least one
/// letter (digits allowed), then a lower-case `s`, or `es` after a stem
/// ending in `S`, `X`, `Z`, `CH` or `SH` ("IDs", "SKUs", "OSes"). The
/// two-character minimum keeps "As", "Is" and "Us" out.
pub(crate) fn acronym_stem(word: &str) -> Option<usize> {
    let is_stem = |s: &str| {
        s.len() >= 2
            && s.bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            && s.bytes().any(|b| b.is_ascii_uppercase())
    };
    if let Some(stem) = word.strip_suffix("es") {
        let sibilant =
            stem.ends_with(['S', 'X', 'Z']) || stem.ends_with("CH") || stem.ends_with("SH");
        if sibilant && is_stem(stem) {
            return Some(stem.len());
        }
    }
    let stem = word.strip_suffix('s')?;
    is_stem(stem).then_some(stem.len())
}

/// Endings that mark a word the fallback never strips: Latin and Greek
/// singulars (`-as`, `-is`, `-us`, `-os`, `-ys`), `-ss`, field names (`-ics`),
/// abstract nouns (`-ities`), adverbs and pronouns (`-wards`, `-selves`),
/// `-ops`, and the shapes of names and taxa ("Moses", "Ulysses",
/// "Kubernetes", `-ales`, `-iformes`, `-ides`), and "facies", the one Latin
/// singular in `-cies` that WordNet lacks.
const FALLBACK_REFUSED: &[&str] = &[
    "as", "is", "us", "ss", "os", "ys", "ics", "ities", "wards", "selves", "ops", "sses", "etes",
    "ales", "iformes", "ides", "facies",
];

/// Whether the fallback leaves the unknown, lower-case `word` unchanged:
/// it ends in a [`FALLBACK_REFUSED`] ending, in a vowel plus `ses`, or in an
/// invariant noun of at least four letters ("superspecies", "subseries").
fn fallback_refuses(word: &str) -> bool {
    if FALLBACK_REFUSED.iter().any(|t| word.ends_with(t)) {
        return true;
    }
    if let Some(stem) = word.strip_suffix("ses") {
        if stem.ends_with(['a', 'e', 'i', 'o', 'u']) {
            return true;
        }
    }
    policy::INVARIANT
        .iter()
        .any(|inv| inv.len() >= 4 && word.ends_with(inv))
}

/// Whether the lower-case ASCII `reduced` is evidence of a real word: it is a
/// WordNet noun, or it ends in a WordNet noun of at least four letters after
/// at least two letters of a plausible compound boundary. "microservice"
/// ends in "service" after "micro"; "elap" ends in "lap" (too short) and
/// "lach" ends in no noun, so both are refused.
fn has_known_head(reduced: &str) -> bool {
    is_noun_key(reduced)
        || (2..=reduced.len().saturating_sub(4)).any(|i| is_noun_key(&reduced[i..]))
}

/// The conservative rules for a word the dictionary does not know. They fire
/// only on a word of at least four lower-case ASCII letters that
/// [`fallback_refuses`] does not reject, and only when the reduced word passes
/// [`has_known_head`] (it is a noun, or compounds a noun of four letters or
/// more); otherwise the word is unchanged:
///
/// - `-ies` after a consonant becomes `-y`
/// - `s`, `x`, `z`, `ch` or `sh` followed by `es` loses the `es`
/// - `-s` is stripped after a consonant or after `e`
pub(crate) fn fallback_singular(word: &str) -> Option<String> {
    let b = word.as_bytes();
    if b.len() < 4 || !b.iter().all(u8::is_ascii_lowercase) || fallback_refuses(word) {
        return None;
    }
    let reduced = reduce(word)?;
    has_known_head(&reduced).then_some(reduced)
}

fn reduce(word: &str) -> Option<String> {
    let is_vowel = |c: u8| matches!(c, b'a' | b'e' | b'i' | b'o' | b'u');
    if let Some(stem) = word.strip_suffix("ies") {
        if stem.len() >= 2 && !is_vowel(stem.as_bytes()[stem.len() - 1]) {
            return Some(String::from(stem) + "y");
        }
    }
    for tail in ["ses", "xes", "zes", "ches", "shes"] {
        if word.ends_with(tail) {
            return Some(String::from(&word[..word.len() - 2]));
        }
    }
    let stem = word.strip_suffix('s')?;
    match stem.as_bytes()[stem.len() - 1] {
        b'a' | b'i' | b'o' | b'u' | b'y' => None,
        _ => Some(String::from(stem)),
    }
}

/// Whether `word` already reads as a plural, so pluralize leaves it alone:
///
/// - an acronym plural ("IDs");
/// - a word the dictionary steps or the unknown-word fallback reduce to a
///   different word ("glasses", "men", "gizmos"); or
/// - a word WordNet lists only as another part of speech ("perhaps"); or
/// - a word that is not a lower-case WordNet noun, ends in `s` but not `ss`,
///   and is not a `-sis` word the plural rules handle ("Kubernetes",
///   "Postgres", "Athens", "facies"). Such a word is unknown or only a name,
///   and adding `es` would invent a non-word. This holds whether or not the
///   unknown-word fallback strips the `s`, so a word the fallback refuses
///   ("kubernetes", "moses") is also left alone by pluralize; or
/// - a WordNet noun that singularize keeps and that has the shape of a plural
///   (see [`rules::has_plural_shape`]: `s` after a consonant, `e`, a digit
///   or an apostrophe), when WordNet, the policy lists and [`S_REGULAR`] give
///   it no plural ("thanks", "clothes", "1990s"). Nearly all are listed only as
///   plural-only nouns, so `es` would invent a non-word.
pub(crate) fn reads_as_plural(word: &str, key: &str) -> bool {
    if acronym_stem(word).is_some() {
        return true;
    }
    match resolve(key) {
        Sing::To(_) | Sing::Other => true,
        Sing::Keep if is_noun_key(key) => {
            rules::has_plural_shape(key) && !is_s_regular(key) && listed_plural(key).is_none()
        }
        // A name such as "Kansas" is known, but not a noun.
        Sing::Keep | Sing::Unknown => rules::ends_in_bare_s(key),
    }
}

/// The [`Origin`] of the lower-case `plural` of `key`. A plural that only adds
/// "s" or "es" is an appended suffix, however it was found ("ISBN" to "ISBNs",
/// "OS" to "OSes", an override "SKU" to "SKUs"); any other plural is a listed
/// form that follows the word's case ("CHILD" to "CHILDREN").
pub(crate) fn plural_origin(key: &str, plural: &str) -> Origin {
    match plural.strip_prefix(key) {
        Some("s" | "es") => Origin::Rule,
        _ => Origin::Form,
    }
}

/// Pluralize step 4 for the lower-case `key`: the irregular pairs, the plural
/// choices and WordNet's forms, with where the answer came from, or `None`
/// when none of them lists a plural.
fn listed_plural(key: &str) -> Option<(Cow<'static, str>, Origin)> {
    if let Some(&(_, plural)) = policy::IRREGULAR.iter().find(|&&(s, _)| s == key) {
        return Some((Cow::Borrowed(plural), plural_origin(key, plural)));
    }
    if let Ok(i) = policy::PLURAL_CHOICE.binary_search_by(|&(k, _)| k.cmp(key)) {
        let plural = policy::PLURAL_CHOICE[i].1;
        return Some((Cow::Borrowed(plural), plural_origin(key, plural)));
    }
    // Rows are (form, lemma). The generator guarantees that the policy lists
    // settle a lemma with several forms.
    let mut forms = rows_by_lemma(key)
        .filter(|&(form, lemma)| !is_excluded(lemma, form))
        .map(|(form, _)| form);
    if let Some(form) = forms.next() {
        debug_assert!(forms.all(|f| f == form), "unsettled lemma {key:?}");
        return Some((Cow::Borrowed(form), plural_origin(key, form)));
    }
    None
}

/// Pluralize steps 4 and 5 for the lower-case `key`: the listed plurals and
/// the plural rules, with where the answer came from. The key is
/// returned unchanged when that is the plural ("forceps").
pub(crate) fn plural(key: &str) -> (Cow<'static, str>, Origin) {
    listed_plural(key).unwrap_or_else(|| {
        (
            Cow::Owned(rules::plural(
                key,
                is_noun_key,
                is_man_regular,
                is_ch_regular,
            )),
            Origin::Rule,
        )
    })
}
