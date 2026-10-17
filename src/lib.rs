//! English noun inflection checked against a dictionary.
//!
//! `moult` turns an English noun into its singular or plural form. Suffix
//! rules alone over-fire ("waves" → "wafe", "police" → "polouse") or
//! under-fire ("analysis" → "analysi"). When singularizing, `moult` accepts a
//! suffix rule's output only when it is a noun [Open English WordNet] knows,
//! and it reads irregular forms from WordNet's own form list. Plural rules
//! form a plural from the word without such a check, since a plural is not a
//! lemma ("cat" → "cats"). A word WordNet does not contain comes back
//! unchanged from [`singularize`] unless you opt in to a conservative guess
//! with [`InflectorBuilder::guess_unknown`]; the free functions never guess.
//!
//! ```
//! assert_eq!(moult::singularize("waves"), "wave");
//! assert_eq!(moult::singularize("analysis"), "analysis");
//! assert_eq!(moult::pluralize("criterion"), "criteria");
//! assert_eq!(moult::singularize("ISBNs"), "ISBN");
//! ```
//!
//! Both functions return a [`Cow`]: [`Cow::Borrowed`] with the input when the
//! word comes back unchanged, so the common case allocates nothing.
//! [`is_noun`] tells "unchanged because the dictionary knows it" from
//! "unchanged because nothing does". An [`Inflector`] adds caller-supplied
//! overrides in front of everything below.
//!
//! # Singularize
//!
//! A word resolves in this order; the first step that answers wins. Lookup
//! is case-insensitive.
//!
//! 0. **Overrides**: a plural given to an [`Inflector`] gives its singular;
//!    an override singular or invariant word is unchanged.
//! 1. **Not a word**: input of exactly one character ("s", "a", "I"), input
//!    that contains whitespace, and input without a letter ("", "41") are
//!    unchanged.
//! 2. **Acronym plural**: an all-caps stem of at least two characters (digits
//!    allowed, at least one letter) followed by a lower-case `s`, or `es`
//!    after a stem ending in S, X, Z, CH or SH, gives the stem ("IDs" →
//!    "ID", "SKUs" → "SKU", "OSes" → "OS").
//! 3. **Invariant nouns**: a closed list of nouns whose plural is the
//!    singular, or that read as mass nouns ("data", "news", "series",
//!    "logistics"). Unchanged.
//! 4. **Irregular forms**, in this order: the supplement for pairs WordNet
//!    lacks ("people" → "person"); the curated choices for a word with more
//!    than one reading ("leaves" → "leaf", "men" → "man", "cola" kept); and
//!    WordNet's form list, minus the typos and non-plurals the policy
//!    excludes ("mice" → "mouse", "genus" → "genus"). A form equal to its
//!    lemma is unchanged.
//! 5. **Other part of speech**: a word WordNet lists only under parts of
//!    speech other than noun (verbs, adjectives, adverbs; WordNet files
//!    interjections such as "alas" as adverbs), and that ends in `s`
//!    ("perhaps", "sometimes", "alas"), is unchanged, unless a checked rule reduces it to a noun
//!    whose own plural, as step 4 and the plural rules of pluralize compute
//!    it, is exactly that word ("bats" → "bat", "nuts" → "nut"; "alas" stays,
//!    because the plural of "ala" is "alae"). It is a known word, so the
//!    guess never applies to it, and when unchanged it is not a noun:
//!    [`is_noun`] is `false`.
//! 6. **Checked rules**: the first suffix rule whose output is a WordNet
//!    noun of at least three characters ("waves" → "wave", "statuses" →
//!    "status", "policemen" → "policeman", "mitoses" → "mitosis"). Names
//!    that WordNet writes only with capitals are never a rule's output, and
//!    a `-men` word never reduces to a `-man` noun that takes `-s`
//!    ("dolmen" is not the plural of "dolman").
//! 7. **Known word**: a word in WordNet's nouns or names is already singular
//!    ("gas", "lens", "Kansas").
//! 8. **Unknown word**: a word no step above resolves is unchanged, and the
//!    free functions stop here. Only an [`Inflector`] built with
//!    [`guess_unknown(true)`](InflectorBuilder::guess_unknown) goes on to
//!    guess. It does so only for a word of at least four lower-case ASCII
//!    letters (capitalised words, and words with digits or symbols, are
//!    unchanged), with conservative unchecked rules, each accepted only when the
//!    result is a WordNet noun, or ends in a WordNet noun of at least four
//!    letters after at least two more letters ("microservices" →
//!    "microservice", "endpoints" → "endpoint", but "elaps" stays: "elap" is
//!    no word and "lap" is too short a head): `-ies` after a consonant
//!    becomes `-y`; `s`, `x`, `z`, `ch` or `sh` plus `es` loses the `es`;
//!    `-s` is dropped after a consonant or after `e`. Left unchanged: a word
//!    ending in an invariant noun of at least four letters ("superspecies",
//!    "subseries", "biomechanics"); a word ending in `-as`, `-is`, `-us`,
//!    `-ss`, `-os`, `-ys`, `-ics`, `-ities`, `-wards`, `-selves`, `-ops`, a
//!    vowel plus `-ses` ("moses"), `-sses` ("ulysses"), `-etes`
//!    ("kubernetes"), a taxon ending `-ales`, `-iformes`, `-ides`, or
//!    "facies". Leaving a word alone beats inventing a non-word. Everything
//!    else is unchanged. The guess is best-effort: it can still produce a
//!    non-word for a compound whose last part is a noun ("afterhours" →
//!    "afterhour"). [`pluralize`] and [`is_noun`] do not depend on it.
//!
//! # Pluralize
//!
//! 0. **Overrides**: an override singular gives its plural; an override
//!    plural or invariant word is unchanged.
//! 1. **Not a word**, as in singularize: unchanged.
//! 2. **Invariant nouns**, unchanged.
//! 3. **Already plural**, unchanged: an acronym plural ("IDs"); a word that
//!    steps 3 to 8 of singularize reduce to a different word ("glasses",
//!    "men", "gizmos"); a word WordNet lists only as another part of speech
//!    ("perhaps"); or a word that is not a WordNet noun and ends in
//!    `s` but not `ss` ("Kubernetes", "Athens"), unless it is a `-sis` word
//!    of at least five characters. That holds for every such word, including
//!    one the opt-in unknown-word guess of singularize refuses to strip, and
//!    whether or not that guess is on. It stops "glasses" becoming "glasseses".
//! 4. **Irregular forms**, in this order: the supplement ("person" →
//!    "people"); the curated choices ("brother" → "brothers", "stadium" →
//!    "stadiums", "gas" → "gases"); and WordNet's form list minus the
//!    excluded forms ("mouse" → "mice", "genus" → "genera").
//! 5. **Plural rules**: `-man` to `-men` for a WordNet noun that is not in
//!    the closed list of nouns where "man" is not the word ("policeman" →
//!    "policemen", but "human" → "humans"); `-ch` to `-chs` for a word in
//!    the closed list of nouns whose "ch" is not the sound in "church"
//!    ("stomach" → "stomachs", "monarch" → "monarchs", but "church" →
//!    "churches"); `-sis` to `-ses` for a word of at
//!    least five characters ("mitosis" → "mitoses"); consonant + `y` to
//!    `-ies`; `s`, `x`, `z`, `ch` and `sh` take `es`; anything else takes
//!    `s`. A `-o` or `-f` noun takes a plain `s` unless WordNet lists an
//!    `-oes` or `-ves` form.
//!
//! # Case
//!
//! Both directions keep the input's case pattern. The characters of the
//! input are kept over the parts of the word that do not change ("Mothers-in-Law"
//! → "Mother-in-Law"). A
//! capitalised word stays capitalised ("Mice" → "Mouse", "Person" →
//! "People"). An all-caps word stays upper case where the word is rewritten
//! ("MOUSE" → "MICE", "TERRITORY" → "TERRITORIES") and for an irregular or
//! listed plural ("CHILD" → "CHILDREN", "OX" → "OXEN"), but a rule suffix
//! added to the whole word is lower case, as acronym plurals are written
//! ("ISBN" → "ISBNs", "BOX" → "BOXes"). A plural that is the word plus `s` or
//! `es`, whether it came from a rule, WordNet or an override, counts as such a
//! suffix ("OS" → "OSes", an override "SKU" → "SKUs"); an override plural of
//! any other shape follows the word's case.
//!
//! The functions read a single word. Splitting a compound or an identifier
//! ("OrderLines", "farm_workers") and inflecting its head is the caller's
//! choice.
//!
//! [Open English WordNet]: https://en-word.net/

#![no_std]

extern crate alloc;

use alloc::borrow::Cow;

mod case;
mod data;
mod inflector;
mod lookup;
mod policy;
mod resolve;
mod rules;

pub use inflector::{Inflector, InflectorBuilder, OverrideError};

/// The inflector with no overrides, behind the free functions.
static DEFAULT: Inflector = Inflector::new();

/// The singular form of the English noun `word`.
///
/// Returns [`Cow::Borrowed`] with the input when the word is unchanged. The
/// order of resolution is in the [crate documentation](crate).
///
/// ```
/// assert_eq!(moult::singularize("Territories"), "Territory");
/// assert_eq!(moult::singularize("gas"), "gas");
/// assert!(matches!(moult::singularize("gas"), std::borrow::Cow::Borrowed(_)));
/// ```
pub fn singularize(word: &str) -> Cow<'_, str> {
    DEFAULT.singularize(word)
}

/// The plural form of the English noun `word`.
///
/// Returns [`Cow::Borrowed`] with the input when the word is unchanged, which
/// includes a word that already reads as a plural. The order of resolution is
/// in the [crate documentation](crate).
///
/// ```
/// assert_eq!(moult::pluralize("quiz"), "quizzes");
/// assert_eq!(moult::pluralize("glasses"), "glasses");
/// ```
pub fn pluralize(word: &str) -> Cow<'_, str> {
    DEFAULT.pluralize(word)
}

/// Whether `word` is a noun Open English WordNet knows: a noun or name in the
/// word lists, or a word the invariant list, the irregular forms or the
/// checked rules reduce to a noun. The opt-in unknown-word guess does not
/// count, so `false` means [`pluralize`] is guessing and [`singularize`] leaves
/// the word alone.
///
/// ```
/// assert!(moult::is_noun("mice"));
/// assert!(!moult::is_noun("blorps"));
/// ```
pub fn is_noun(word: &str) -> bool {
    DEFAULT.is_noun(word)
}

#[cfg(test)]
mod table_tests;
