//! English inflection and lemmatization checked against a dictionary.
//!
//! `moult` turns an English noun into its singular or plural form, forms the
//! third person singular present of a verb, and finds dictionary lemmas. Suffix
//! rules alone over-fire ("waves" → "wafe", "police" → "polouse") or
//! under-fire ("analysis" → "analysi"). When singularizing, `moult` accepts a
//! suffix rule's output only when it is a noun [Open English WordNet] knows,
//! and it reads irregular forms from WordNet's own form list. Plural rules
//! form a plural from the word without such a check, since a plural is not a
//! lemma ("cat" → "cats"). Verb inflection checks the input against WordNet's
//! verb lemmas before forming its third person singular. A word WordNet does
//! not contain comes back unchanged from [`singularize`] or
//! [`third_person_singular`] unless you opt
//! in to a conservative guess with [`InflectorBuilder::guess_unknown`]; the
//! free functions never guess.
//!
//! ```
//! assert_eq!(moult::singularize("waves"), "wave");
//! assert_eq!(moult::singularize("analysis"), "analysis");
//! assert_eq!(moult::pluralize("criterion"), "criteria");
//! assert_eq!(moult::singularize("ISBNs"), "ISBN");
//! ```
//!
//! [`singularize_phrase`] and [`pluralize_phrase`] do the same for a noun
//! phrase: they inflect its last word. [`third_person_singular_phrase`]
//! inflects the first word of a verb phrase (see [Phrases](#phrases)).
//!
//! The inflection functions return a [`Cow`]: [`Cow::Borrowed`] with the input
//! when the word comes back unchanged, so the common case allocates nothing.
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
//!    that contains whitespace (see [`singularize_phrase`]), and input without
//!    a letter ("", "41") are unchanged.
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
//! 1. **Not a word**, as in singularize: unchanged (see
//!    [`pluralize_phrase`] for input that contains whitespace). So is a word
//!    that ends in a character that is not a letter or digit ("dock,",
//!    "C++"), which has no end to append a suffix to; "MP3" gives "MP3s".
//! 2. **Invariant nouns**, unchanged.
//! 3. **Already plural**, unchanged: an acronym plural ("IDs"); a word that
//!    steps 3 to 8 of singularize reduce to a different word ("glasses",
//!    "men", "gizmos"); a word WordNet lists only as another part of speech
//!    ("perhaps"); or a word that is not a WordNet noun and ends in
//!    `s` but not `ss` ("Kubernetes", "Athens"), unless it is a `-sis` word
//!    of at least five characters. That holds for every such word, including
//!    one the opt-in unknown-word guess of singularize refuses to strip, and
//!    whether or not that guess is on. It stops "glasses" becoming "glasseses".
//!    A WordNet noun is also unchanged when singularize keeps it, it ends in
//!    `s` but not `ss` after a consonant, `e`, a digit or an apostrophe (not
//!    after `a`, `i`, `o`, `u` or `y`, which end real singulars such as "bus"
//!    and "iris"), it is not a `-sis` word of at least five characters, and
//!    no list or WordNet form gives it a plural ("thanks", "clothes",
//!    "earnings", "1990s"). Nearly all are plural-only nouns, and `es`
//!    would invent a non-word. A closed list holds the real singulars of that
//!    shape, which take the regular plural ("lens" → "lenses", "summons" →
//!    "summonses", "yes" → "yeses").
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
//! # Third person singular
//!
//! A verb resolves in this order; the first step that answers wins. Lookup
//! is case-insensitive.
//!
//! 0. **Overrides**: a verb given to
//!    [`InflectorBuilder::third_person_singular`] gives its form; an override
//!    form is unchanged. A verb equal to its form means unchanged.
//! 1. **Not a word**: input of one character, input containing whitespace,
//!    input without a letter, or a word not ending in a letter or digit is
//!    unchanged (see [`third_person_singular_phrase`] for phrases).
//! 2. **Modals**, unchanged: can, could, may, might, must, ought, shall,
//!    should, will and would. Need and dare are ordinary verbs here.
//! 3. **Listed forms**: be → is, degas → degasses, gas → gasses, have → has,
//!    nonplus → nonplusses, quiz → quizzes and whiz → whizzes.
//! 4. **Known verb**: a WordNet verb or capitalised-only verb takes the suffix
//!    rules. Consonant + `y` becomes `-ies` ("carry" → "carries"); `s`, `x`,
//!    `z`, `ch` or `sh` takes `-es`; consonant + `o` takes `-es` only in a
//!    closed list ("go" → "goes", "echo" → "echoes"). Anything else takes
//!    `-s` ("live" → "lives", "boo" → "boos", "demo" → "demos").
//! 5. **Unknown**, unchanged unless [`guess_unknown(true)`](InflectorBuilder::guess_unknown)
//!    is set. Then the rules apply only to a word of at least four lower-case
//!    ASCII letters with no verb candidates from that inflector's
//!    [`lemmas_for`], and not ending in `s` (except `ss`), `ed` or `ing`.
//!    Guessed `-o` words take plain `-s`. Known verb forms such as "lives"
//!    and "lived" stay unchanged. Leaving a word alone beats inventing a
//!    non-word; the free functions never guess.
//!
//! Every result keeps the input's case, including upper-case suffixes for
//! all-caps verbs. The result is [`Cow::Borrowed`] exactly when unchanged.
//!
//! ```
//! assert_eq!(moult::third_person_singular("carry"), "carries");
//! assert_eq!(moult::third_person_singular("LIVE"), "LIVES");
//! assert_eq!(moult::third_person_singular("be"), "is");
//! ```
//!
//! With the free functions, `lemmatize(third_person_singular(v), Verb) == v`
//! holds for every lower-case WordNet verb except 9 spelling variants that
//! [`lemmatize`] canonicalises: ax → axe, poleax → poleaxe, caddy → caddie,
//! chevy and chivvy → chivy, programme → program, stymie → stymy,
//! swop → swap, whizz → whiz. The 23 capitalised-only verbs that take a
//! suffix (Americanize, French, DJ, …) inflect but do not round-trip, because
//! no suffix rule of `lemmatize` reaches a capitalised-only lemma. Verbs ending
//! in punctuation (nol.pros., o.d., wildin', g.i., o.k.) come back unchanged.
//!
//! # Phrases
//!
//! [`singularize_phrase`] and [`pluralize_phrase`] find the last run of
//! non-whitespace characters ([`char::is_whitespace`], so U+00A0 counts) and
//! inflect it with the word function of the same [`Inflector`], so overrides,
//! case handling and [`guess_unknown`](InflectorBuilder::guess_unknown) apply.
//! The text before and after it is kept byte for byte. Input with no
//! whitespace behaves exactly like the word function, and empty or
//! all-whitespace input is unchanged. The result is [`Cow::Borrowed`] when it
//! equals the input.
//!
//! ```
//! assert_eq!(moult::singularize_phrase("order lines"), "order line");
//! assert_eq!(moult::singularize_phrase("  Order Lines\n"), "  Order Line\n");
//! assert_eq!(moult::pluralize_phrase("dry dock"), "dry docks");
//! ```
//!
//! [`third_person_singular_phrase`] instead inflects the first run of
//! non-whitespace characters, with the same whitespace and borrowing contract.
//! "live in" gives "lives in", and "  carry out\n" gives "  carries out\n".
//! A phrase whose first word is not its verb is inflected wrongly; choosing
//! the verb is the caller's job.
//!
//! Only the last word changes in the noun functions, so a head-initial
//! compound inflects the wrong word: "attorney general" gives "attorney
//! generals", not "attorneys general".
//!
//! # Lemmatize
//!
//! [`lemmatize`] turns an inflected word into its dictionary form as a given
//! [`PartOfSpeech`] ("made" → "make" as a verb, "better" → "good" as an
//! adjective and "well" as an adverb). [`lemmas`] and [`lemmas_for`] list every
//! reading of a word as [`Lemma`] values: lower-case dictionary entries, each
//! with its part of speech. For a verb, adjective or adverb, the case-kept
//! result of `lemmatize` ("Went" → "Go", "MADE" → "MAKE") is the first
//! candidate with the input's case pattern applied; candidates themselves are
//! lower case. For a noun, `lemmatize` is [`singularize`] (see below), which
//! can also apply the acronym rule and the opt-in unknown-word guess.
//!
//! Every candidate is a dictionary lemma (a noun or name of WordNet for the
//! noun reading; a lemma of that part of speech for the others) or an
//! [`Inflector`] override; the input itself need not be listed ("hating"
//! gives "hate" and "hat"). A known lemma lists itself, so a word with no
//! candidates is one that no WordNet form, suffix rule or override connects
//! to a lemma of that part of speech.
//!
//! The candidates of one part of speech come in this order, and a lemma
//! repeated as the same part of speech keeps its first position:
//!
//! 1. the [`Inflector`]'s override ([`InflectorBuilder::lemma`]);
//! 2. the curated choice for a form with more than one reading ("lay" → "lay"
//!    as a verb, which is also the past tense of "lie");
//! 3. the forms WordNet lists ("made" → "make", "better" → "good", "found" →
//!    "find"), in table order;
//! 4. the suffix rules, when the stem is a lemma of that part of speech:
//!    `-ies`, `-ied` and `-ying` ("carries", "carried", "dying"), `-es` and
//!    `-s`, `-ed` and `-ing` to the stem or to the stem plus `e` or without a
//!    doubled consonant ("hating" → "hate", "stopped" → "stop"), and `-ier`,
//!    `-iest`, `-er` and `-est` for adjectives ("later" → "late"). Adverbs have
//!    no rule;
//! 5. the word itself, when it is a lemma of that part of speech ("modest").
//!
//! Lemmas WordNet writes only with capitals ("Americanize" as a verb,
//! "African" as an adjective, "OK" as an adverb) are treated as for nouns,
//! where they are names: they are known, so they list themselves (as
//! lower-case text) and are the lemma of a form WordNet lists for them, but
//! no suffix rule ever reaches one.
//!
//! [`lemmas`] lists the nouns, then the verbs, adjectives and adverbs. That
//! order carries no preference between parts of speech; the caller knows the
//! part of speech and uses [`lemmas_for`] or [`lemmatize`]. For example
//! `lemmas("saw")` has "saw" as a noun, "see" as a verb and "saw" as a verb.
//!
//! The nouns are [`singularize`]: [`lemmatize`] with [`PartOfSpeech::Noun`] is
//! exactly `singularize`. When `singularize` reaches a noun or name of the
//! dictionary through its checked steps (overrides first), that word is the
//! first noun candidate. Where its answer is a word that is not in WordNet's
//! nouns or names (a word the policy lists keep, such as "ellipses"), it is
//! not a candidate, so a word can have no noun candidate and still come back
//! from `singularize` unchanged. The other noun readings follow: other
//! WordNet forms, the irregular pairs, rule results that are nouns, and the
//! word itself when it is a noun or name ("axes" lists "axis", then "ax" and
//! "axe"). There are no acronym or guessed candidates.
//!
//! A word the lists, forms and rules do not connect to a lemma has no
//! candidates, and `lemmatize` returns it unchanged; a lemma lists itself,
//! whatever its length ("a" is a noun lemma, "10" an adjective lemma). The
//! functions take one word: empty input, and input that contains whitespace,
//! has no candidates. The suffix rules have minimum lengths, and stem rules for
//! verbs and adjectives read ASCII words of at most 64 bytes. An [`Inflector`]
//! can override a lemma per part of speech with [`InflectorBuilder::lemma`]; a
//! noun lemma override acts as the singular/plural pair it states.
//!
//! The result ([`Lemmas`]) has fixed storage and its candidate text is
//! borrowed from the tables and the [`Inflector`]. Looking up an ASCII word of
//! at most 64 bytes allocates nothing for verbs, adjectives and adverbs. The
//! noun reading allocates a temporary `String` when a suffix rule of
//! [`singularize`] fires, and a longer or non-ASCII word allocates a
//! lower-cased lookup key (as [`singularize`] does).
//!
//! ```
//! use moult::PartOfSpeech::{Adverb, Verb};
//! assert_eq!(moult::lemmatize("running", Verb), "run");
//! assert_eq!(moult::lemmatize("hating", Verb), "hate");
//! assert_eq!(moult::lemmatize("MADE", Verb), "MAKE");
//! assert_eq!(moult::lemmatize("better", Adverb), "well");
//! ```
//!
//! # Case
//!
//! Inflection keeps the input's case pattern. The characters of the
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
//! any other shape follows the word's case. Verbs always follow the input's
//! case, including the suffix: "LIVE" → "LIVES", unlike "BOX" → "BOXes";
//! "CARRY" → "CARRIES" and "BE" → "IS". All-caps capitalised-only verbs
//! take an upper-case suffix too ("DJ" → "DJS").
//!
//! [`singularize`], [`pluralize`] and [`third_person_singular`] read a single
//! word; the phrase functions inflect the last word of a noun phrase or the
//! first word of a verb phrase.
//! Splitting an identifier ("OrderLines", "farm_workers") and inflecting a
//! head-initial compound's head ("attorney general") is the caller's job.
//!
//! [Open English WordNet]: https://en-word.net/

#![no_std]

extern crate alloc;

use alloc::borrow::Cow;

mod case;
mod data;
mod inflector;
mod lemma;
mod lookup;
mod policy;
mod pos;
mod resolve;
mod rules;
mod verb;

pub use inflector::{Inflector, InflectorBuilder, OverrideError};
pub use lemma::{Lemma, Lemmas};
pub use pos::PartOfSpeech;

/// The inflector with no overrides, behind the free functions.
static DEFAULT: Inflector = Inflector::new();

/// The singular form of the English noun `word`.
///
/// Returns [`Cow::Borrowed`] with the input when the word is unchanged. The
/// order of resolution is in the [crate documentation](crate). Input that
/// contains whitespace is returned unchanged; use [`singularize_phrase`] for a
/// phrase.
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
/// in the [crate documentation](crate). Input that contains whitespace is
/// returned unchanged; use [`pluralize_phrase`] for a phrase.
///
/// ```
/// assert_eq!(moult::pluralize("quiz"), "quizzes");
/// assert_eq!(moult::pluralize("glasses"), "glasses");
/// ```
pub fn pluralize(word: &str) -> Cow<'_, str> {
    DEFAULT.pluralize(word)
}

/// The third person singular present of the English verb `verb`.
///
/// Returns [`Cow::Borrowed`] with the input when unchanged. Resolution is in
/// [Third person singular](crate#third-person-singular). Input containing
/// whitespace is unchanged; use [`third_person_singular_phrase`] for a phrase.
///
/// ```
/// assert_eq!(moult::third_person_singular("live"), "lives");
/// assert_eq!(moult::third_person_singular("Live"), "Lives");
/// assert_eq!(moult::third_person_singular("be"), "is");
/// assert_eq!(moult::third_person_singular("go"), "goes");
/// assert_eq!(moult::third_person_singular("can"), "can");
/// assert_eq!(moult::third_person_singular("qwzzx"), "qwzzx");
/// assert!(matches!(moult::third_person_singular("qwzzx"), std::borrow::Cow::Borrowed(_)));
/// ```
pub fn third_person_singular(verb: &str) -> Cow<'_, str> {
    DEFAULT.third_person_singular(verb)
}

/// The third person singular present of `phrase`: its first word is
/// inflected and everything around it is kept byte for byte.
///
/// The first word is the first run of non-whitespace characters. Returns
/// [`Cow::Borrowed`] when unchanged, including empty or all-whitespace input.
/// Choosing a phrase whose first word is its verb is the caller's job.
/// See [Phrases](crate#phrases).
///
/// ```
/// assert_eq!(moult::third_person_singular_phrase("live in"), "lives in");
/// assert_eq!(moult::third_person_singular_phrase("take place in"), "takes place in");
/// assert_eq!(moult::third_person_singular_phrase("belong to"), "belongs to");
/// assert_eq!(moult::third_person_singular_phrase("  carry out\n"), "  carries out\n");
/// ```
pub fn third_person_singular_phrase(phrase: &str) -> Cow<'_, str> {
    DEFAULT.third_person_singular_phrase(phrase)
}

/// The singular form of the English noun phrase `phrase`: its last word is
/// singularized and the rest is kept as it was.
///
/// The last word is the last run of non-whitespace characters; the text
/// around it is kept byte for byte. Returns [`Cow::Borrowed`] with the input
/// when the result equals it. A head-initial compound ("attorney general")
/// inflects the wrong word, and splitting identifiers is the caller's job.
/// See [Phrases](crate#phrases).
///
/// ```
/// assert_eq!(moult::singularize_phrase("order lines"), "order line");
/// assert_eq!(moult::singularize_phrase("Order Lines"), "Order Line");
/// assert!(matches!(
///     moult::singularize_phrase("order line"),
///     std::borrow::Cow::Borrowed(_)
/// ));
/// ```
pub fn singularize_phrase(phrase: &str) -> Cow<'_, str> {
    DEFAULT.singularize_phrase(phrase)
}

/// The plural form of the English noun phrase `phrase`: its last word is
/// pluralized and the rest is kept as it was.
///
/// Whitespace handling, the `Cow` contract and the head-initial limitation
/// are those of [`singularize_phrase`].
///
/// ```
/// assert_eq!(moult::pluralize_phrase("dry dock"), "dry docks");
/// assert_eq!(moult::pluralize_phrase("  order line\n"), "  order lines\n");
/// assert_eq!(moult::pluralize_phrase("attorney general"), "attorney generals");
/// ```
pub fn pluralize_phrase(phrase: &str) -> Cow<'_, str> {
    DEFAULT.pluralize_phrase(phrase)
}

/// Whether `word` is a noun Open English WordNet knows: a noun or name in the
/// word lists, or a word the invariant list, the irregular forms or the
/// checked rules reduce to a noun. The opt-in unknown-word guess does not
/// count: `false` means no dictionary-checked noun reading was found. It is
/// `false` for input that contains whitespace.
///
/// ```
/// assert!(moult::is_noun("mice"));
/// assert!(!moult::is_noun("blorps"));
/// ```
pub fn is_noun(word: &str) -> bool {
    DEFAULT.is_noun(word)
}

/// Every lemma `word` can be read as, in each part of speech, as lower-case
/// dictionary entries. The order is in [Lemmatize](crate#lemmatize). The
/// result is empty for input that contains whitespace and for a word no list,
/// form or rule connects to a lemma.
///
/// ```
/// use moult::PartOfSpeech;
/// let found: Vec<(&str, PartOfSpeech)> =
///     moult::lemmas("running").map(|l| (l.text, l.pos)).collect();
/// assert!(found.contains(&("run", PartOfSpeech::Verb)));
/// assert!(moult::lemmas("qwzzx").is_empty());
/// ```
pub fn lemmas(word: &str) -> Lemmas<'static> {
    DEFAULT.lemmas(word)
}

/// The lemmas of `word` as one part of speech, in the order of [`lemmas`].
///
/// ```
/// use moult::PartOfSpeech::{Adjective, Adverb};
/// assert_eq!(moult::lemmas_for("better", Adjective).first().map(|l| l.text), Some("good"));
/// assert_eq!(moult::lemmas_for("better", Adverb).first().map(|l| l.text), Some("well"));
/// ```
pub fn lemmas_for(word: &str, pos: PartOfSpeech) -> Lemmas<'static> {
    DEFAULT.lemmas_for(word, pos)
}

/// The lemma of `word` as the part of speech `pos`, with the case of the
/// input, or the word itself when it has none. For [`PartOfSpeech::Noun`] this
/// is [`singularize`]. Returns [`Cow::Borrowed`] with the input when it is
/// unchanged.
///
/// ```
/// use moult::PartOfSpeech::{Adjective, Noun, Verb};
/// assert_eq!(moult::lemmatize("made", Verb), "make");
/// assert_eq!(moult::lemmatize("Went", Verb), "Go");
/// assert_eq!(moult::lemmatize("better", Adjective), "good");
/// assert_eq!(moult::lemmatize("mice", Noun), "mouse");
/// assert!(matches!(moult::lemmatize("qwzzx", Verb), std::borrow::Cow::Borrowed(_)));
/// ```
pub fn lemmatize(word: &str, pos: PartOfSpeech) -> Cow<'_, str> {
    DEFAULT.lemmatize(word, pos)
}

#[cfg(test)]
mod table_tests;
