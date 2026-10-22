//! The caller-supplied override layer, and the resolution order that puts it
//! first.

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::case::{restore, Lower, Origin};
use crate::lemma::{self, Lemmas};
use crate::pos::PartOfSpeech;
use crate::resolve::{
    acronym_stem, fallback_singular, is_invariant, is_noun_key, plural, plural_origin,
    reads_as_plural, resolve, Sing,
};

/// An inflector with a set of caller-supplied overrides. It is immutable and
/// `Send + Sync`; build one once and share it.
///
/// Overrides are checked before everything else. A word is an override when,
/// compared case-insensitively, it is the singular or plural of a pair given
/// to [`InflectorBuilder::pair`], or an [`invariant`] word.
///
/// [`invariant`]: InflectorBuilder::invariant
///
/// ```
/// let inflector = moult::Inflector::builder()
///     .pair("sku", "skus")
///     .invariant("personnel")
///     .build()
///     .unwrap();
/// assert_eq!(inflector.pluralize("SKU"), "SKUs");
/// assert_eq!(inflector.singularize("skus"), "sku");
/// assert_eq!(inflector.pluralize("Personnel"), "Personnel");
/// ```
///
/// A lemma override ([`InflectorBuilder::lemma`]) does the same for
/// [`lemmatize`](Inflector::lemmatize) and [`lemmas`](Inflector::lemmas).
#[derive(Clone, Debug, Default)]
pub struct Inflector {
    /// `(singular, plural)`, sorted by singular.
    by_singular: Vec<(Box<str>, Box<str>)>,
    /// `(plural, singular)`, sorted by plural.
    by_plural: Vec<(Box<str>, Box<str>)>,
    /// Sorted.
    invariant: Vec<Box<str>>,
    /// `(form, part of speech, lemma)` for verbs, adjectives and adverbs,
    /// sorted by form and part of speech. Noun lemma overrides are pairs.
    lemmas: Vec<(Box<str>, PartOfSpeech, Box<str>)>,
    /// Whether singularize guesses at words no dictionary-checked step
    /// resolves; see [`InflectorBuilder::guess_unknown`].
    guess_unknown: bool,
}

/// What an override says about a word.
pub(crate) enum Over<'a> {
    /// Return the word unchanged.
    Keep,
    /// Return this lower-case word, the supplied plural when pluralizing.
    To(&'a str),
}

impl Inflector {
    /// An inflector with no overrides: what the free functions use.
    pub const fn new() -> Self {
        Inflector {
            by_singular: Vec::new(),
            by_plural: Vec::new(),
            invariant: Vec::new(),
            lemmas: Vec::new(),
            guess_unknown: false,
        }
    }

    /// A builder for an inflector with overrides.
    pub fn builder() -> InflectorBuilder {
        InflectorBuilder::default()
    }

    /// The override for singularizing the lower-case `key`, if any: a plural
    /// of a pair gives its singular; a singular or an invariant word is left
    /// unchanged.
    pub(crate) fn over_singular(&self, key: &str) -> Option<Over<'_>> {
        if let Ok(i) = self.by_plural.binary_search_by(|(p, _)| (**p).cmp(key)) {
            return Some(Over::To(&self.by_plural[i].1));
        }
        self.is_override_word(key).then_some(Over::Keep)
    }

    /// The override for pluralizing the lower-case `key`, if any: a singular
    /// of a pair gives its plural; a plural or an invariant word is left
    /// unchanged.
    fn over_plural(&self, key: &str) -> Option<Over<'_>> {
        if let Ok(i) = self.by_singular.binary_search_by(|(s, _)| (**s).cmp(key)) {
            return Some(Over::To(&self.by_singular[i].1));
        }
        self.is_override_word(key).then_some(Over::Keep)
    }

    /// The stored lower-case override singular or invariant word equal to
    /// `key`, the text of the candidate that [`Over::Keep`] stands for.
    pub(crate) fn override_word(&self, key: &str) -> Option<&str> {
        if let Ok(i) = self.by_singular.binary_search_by(|(s, _)| (**s).cmp(key)) {
            return Some(&self.by_singular[i].0);
        }
        let i = self.invariant.binary_search_by(|w| (**w).cmp(key)).ok()?;
        Some(&self.invariant[i])
    }

    /// The override lemma of the lower-case `key` as `pos`, if any. Nouns
    /// have none here: their lemma overrides are pairs.
    pub(crate) fn lemma_override(&self, key: &str, pos: PartOfSpeech) -> Option<&str> {
        let i = self
            .lemmas
            .binary_search_by(|(f, p, _)| (&**f, *p).cmp(&(key, pos)))
            .ok()?;
        Some(&self.lemmas[i].2)
    }

    fn is_override_word(&self, key: &str) -> bool {
        let has = |v: &[Box<str>]| v.binary_search_by(|w| (**w).cmp(key)).is_ok();
        has(&self.invariant)
            || self
                .by_singular
                .binary_search_by(|(s, _)| (**s).cmp(key))
                .is_ok()
            || self
                .by_plural
                .binary_search_by(|(p, _)| (**p).cmp(key))
                .is_ok()
    }

    /// The singular form of the English noun `word`.
    ///
    /// Returns [`Cow::Borrowed`] with the input when the word is unchanged.
    /// The order of resolution is in the [crate documentation](crate). Input
    /// that contains whitespace is returned unchanged; use
    /// [`singularize_phrase`](Self::singularize_phrase) for a phrase.
    pub fn singularize<'a>(&self, word: &'a str) -> Cow<'a, str> {
        let low = Lower::new(word);
        let key = low.as_str();
        match self.over_singular(key) {
            Some(Over::To(singular)) => return changed(word, key, singular, Origin::Form),
            Some(Over::Keep) => return Cow::Borrowed(word),
            None => {}
        }
        if is_inert(word) {
            return Cow::Borrowed(word);
        }
        if let Some(len) = acronym_stem(word) {
            return finish(word, String::from(&word[..len]));
        }
        match resolve(key) {
            Sing::Keep | Sing::Other => Cow::Borrowed(word),
            // A singular is never a suffix added to the whole word.
            Sing::To(singular) => changed(word, key, &singular, Origin::Rule),
            Sing::Unknown if self.guess_unknown => match fallback_singular(word) {
                Some(singular) => finish(word, singular),
                None => Cow::Borrowed(word),
            },
            Sing::Unknown => Cow::Borrowed(word),
        }
    }

    /// The plural form of the English noun `word`.
    ///
    /// Returns [`Cow::Borrowed`] with the input when the word is unchanged,
    /// which includes a word that already reads as a plural. The order of
    /// resolution is in the [crate documentation](crate). Input that contains
    /// whitespace is returned unchanged; use
    /// [`pluralize_phrase`](Self::pluralize_phrase) for a phrase.
    pub fn pluralize<'a>(&self, word: &'a str) -> Cow<'a, str> {
        let low = Lower::new(word);
        let key = low.as_str();
        match self.over_plural(key) {
            Some(Over::To(plural)) => {
                // A supplied plural that only adds "s" or "es" is an appended
                // suffix ("SKU" to "SKUs"); any other follows the word's case.
                let origin = plural_origin(key, plural);
                return changed(word, key, plural, origin);
            }
            Some(Over::Keep) => return Cow::Borrowed(word),
            None => {}
        }
        // A word that ends in punctuation ("dock,", "C++") has no end to
        // append a suffix to.
        if key.is_empty()
            || is_inert(word)
            || !word.ends_with(char::is_alphanumeric)
            || is_invariant(key)
            || reads_as_plural(word, key)
        {
            return Cow::Borrowed(word);
        }
        let (plural, origin) = plural(key);
        changed(word, key, &plural, origin)
    }

    /// The singular form of the English noun phrase `phrase`: its last word
    /// ([`singularize`](Self::singularize)) with the rest kept as it was.
    ///
    /// The last word is the last run of non-whitespace characters
    /// ([`char::is_whitespace`]). The text before and after it, including
    /// any leading or trailing whitespace, is kept byte for byte. Input with
    /// no whitespace behaves exactly like `singularize`, and empty or
    /// all-whitespace input is unchanged. Returns [`Cow::Borrowed`] with the
    /// input when the result equals it.
    ///
    /// Head-initial compounds inflect the wrong word ("attorney general"
    /// gives "attorney generals" when pluralized, not "attorneys general"),
    /// and identifiers ("order_lines") are not split. Both are the caller's
    /// job.
    ///
    /// ```
    /// let inflector = moult::Inflector::builder().pair("sku", "skus").build().unwrap();
    /// assert_eq!(inflector.singularize_phrase("blue SKUs"), "blue SKU");
    /// assert_eq!(inflector.singularize_phrase("Order Lines\n"), "Order Line\n");
    /// ```
    pub fn singularize_phrase<'a>(&self, phrase: &'a str) -> Cow<'a, str> {
        self.last_word(phrase, |word| self.singularize(word))
    }

    /// The plural form of the English noun phrase `phrase`: its last word
    /// ([`pluralize`](Self::pluralize)) with the rest kept as it was.
    ///
    /// Whitespace handling, the `Cow` contract and the head-initial
    /// limitation are those of
    /// [`singularize_phrase`](Self::singularize_phrase).
    ///
    /// ```
    /// let inflector = moult::Inflector::builder().pair("sku", "skus").build().unwrap();
    /// assert_eq!(inflector.pluralize_phrase("blue sku"), "blue skus");
    /// assert_eq!(inflector.pluralize_phrase("dry dock "), "dry docks ");
    /// ```
    pub fn pluralize_phrase<'a>(&self, phrase: &'a str) -> Cow<'a, str> {
        self.last_word(phrase, |word| self.pluralize(word))
    }

    /// Applies `inflect` to the last run of non-whitespace characters of
    /// `phrase` and keeps the rest. Borrowed exactly when `inflect` borrows.
    fn last_word<'a>(
        &self,
        phrase: &'a str,
        inflect: impl FnOnce(&'a str) -> Cow<'a, str>,
    ) -> Cow<'a, str> {
        let end = phrase.trim_end().len();
        let start = phrase[..end].rfind(char::is_whitespace).map_or(0, |i| {
            i + phrase[i..].chars().next().map_or(0, char::len_utf8)
        });
        if start == end {
            return Cow::Borrowed(phrase);
        }
        match inflect(&phrase[start..end]) {
            Cow::Borrowed(_) => Cow::Borrowed(phrase),
            Cow::Owned(word) => {
                let mut out = String::with_capacity(phrase.len() - (end - start) + word.len());
                out.push_str(&phrase[..start]);
                out.push_str(&word);
                out.push_str(&phrase[end..]);
                Cow::Owned(out)
            }
        }
    }

    /// Every lemma `word` can be read as, in each part of speech, as
    /// lower-case dictionary entries. See [Lemmatize](crate#lemmatize) for the
    /// order. Empty input and input that contains whitespace have none, and so
    /// has a word no list, form or rule connects to a lemma.
    ///
    /// ```
    /// use moult::PartOfSpeech;
    /// let inflector = moult::Inflector::new();
    /// let found: Vec<_> = inflector.lemmas("running").map(|l| (l.text, l.pos)).collect();
    /// assert!(found.contains(&("run", PartOfSpeech::Verb)));
    /// ```
    pub fn lemmas(&self, word: &str) -> Lemmas<'_> {
        let low = Lower::new(word);
        lemma::collect(self, word, low.as_str(), None)
    }

    /// The lemmas of `word` as one part of speech, in the order of
    /// [`lemmas`](Self::lemmas). Nouns follow singularize, with the other
    /// readings of the word after its answer.
    ///
    /// ```
    /// use moult::PartOfSpeech;
    /// let inflector = moult::Inflector::new();
    /// let first = inflector.lemmas_for("better", PartOfSpeech::Adverb).first();
    /// assert_eq!(first.map(|l| l.text), Some("well"));
    /// ```
    pub fn lemmas_for(&self, word: &str, pos: PartOfSpeech) -> Lemmas<'_> {
        let low = Lower::new(word);
        lemma::collect(self, word, low.as_str(), Some(pos))
    }

    /// The lemma of `word` as the part of speech `pos`, with the case of the
    /// input, or the word itself when it has none.
    ///
    /// For [`PartOfSpeech::Noun`] this is exactly
    /// [`singularize`](Self::singularize). For the others it is the first
    /// candidate of [`lemmas_for`](Self::lemmas_for), with the input's case
    /// pattern applied ("Went" gives "Go", "MADE" gives "MAKE"). Returns
    /// [`Cow::Borrowed`] with the input when it is unchanged.
    ///
    /// ```
    /// use moult::PartOfSpeech::Verb;
    /// let inflector = moult::Inflector::new();
    /// assert_eq!(inflector.lemmatize("Went", Verb), "Go");
    /// assert_eq!(inflector.lemmatize("stopped", Verb), "stop");
    /// ```
    pub fn lemmatize<'a>(&self, word: &'a str, pos: PartOfSpeech) -> Cow<'a, str> {
        if pos == PartOfSpeech::Noun {
            return self.singularize(word);
        }
        let low = Lower::new(word);
        let key = low.as_str();
        match lemma::collect(self, word, key, Some(pos)).first() {
            Some(lemma) => changed(word, key, lemma.text, Origin::Form),
            None => Cow::Borrowed(word),
        }
    }

    /// Whether `word` is a noun this inflector knows: an override word, a
    /// word in Open English WordNet's noun or name lists, or a word that
    /// the overrides, the invariant list, the irregular forms or the checked
    /// rules reduce to a WordNet noun. The unknown-word fallback does not
    /// count, whether or not [`guess_unknown`](InflectorBuilder::guess_unknown)
    /// is on, so `false` means the answer is a guess or the word is left alone.
    /// It is `false` for input that contains whitespace.
    pub fn is_noun(&self, word: &str) -> bool {
        let low = Lower::new(word);
        let key = low.as_str();
        if self.is_override_word(key) {
            return true;
        }
        if let Some(len) = acronym_stem(word) {
            if is_noun_key(Lower::new(&word[..len]).as_str()) {
                return true;
            }
        }
        matches!(resolve(key), Sing::Keep | Sing::To(_))
    }
}

/// Whether both directions leave `word` alone: it is one character, contains
/// whitespace (so it is not one word), or has no letter.
pub(crate) fn is_inert(word: &str) -> bool {
    let mut chars = word.chars();
    let single = chars.next().is_some() && chars.next().is_none();
    single || word.chars().any(char::is_whitespace) || !word.chars().any(char::is_alphabetic)
}

/// `result` with the case of `word`, or the word itself when nothing changed.
fn changed<'a>(word: &'a str, key: &str, result: &str, origin: Origin) -> Cow<'a, str> {
    if result == key {
        Cow::Borrowed(word)
    } else {
        finish(word, restore(word, key, result, origin))
    }
}

/// The one place a result becomes a `Cow`: borrowed exactly when it equals
/// the input, so "unchanged" holds even when case restoration undoes a change
/// ("ΟΣ" with the override "οσ" to "ος").
fn finish(word: &str, result: String) -> Cow<'_, str> {
    if result == word {
        Cow::Borrowed(word)
    } else {
        Cow::Owned(result)
    }
}

/// Collects overrides for an [`Inflector`].
#[derive(Clone, Debug, Default)]
pub struct InflectorBuilder {
    pairs: Vec<(String, String)>,
    invariant: Vec<String>,
    lemmas: Vec<(String, PartOfSpeech, String)>,
    guess_unknown: bool,
}

impl InflectorBuilder {
    /// Overrides the plural of `singular`: singularize turns `plural` into
    /// `singular`, and pluralize turns `singular` into `plural`.
    pub fn pair(mut self, singular: &str, plural: &str) -> Self {
        self.pairs.push((singular.into(), plural.into()));
        self
    }

    /// Overrides the lemma of `form` as the part of speech `pos`:
    /// [`lemmatize`](Inflector::lemmatize) turns `form` into `lemma` first,
    /// and [`lemmas_for`](Inflector::lemmas_for) lists `lemma` ahead of every
    /// dictionary candidate.
    ///
    /// For the other parts of speech, `form` equal to `lemma` is allowed: it
    /// makes the word its own first lemma as that part of speech, so a caller
    /// can prefer "saw" over "see" for the verb "saw". Two overrides for one
    /// form and part of speech fail [`build`](Self::build) with
    /// [`OverrideError::DuplicateLemma`].
    ///
    /// For [`PartOfSpeech::Noun`] this is the pair `(lemma, form)`: the lemma
    /// of a noun is its singular, so `lemma("geese", Noun, "goose")` acts as
    /// `pair("goose", "geese")` and
    /// [`pluralize`](Inflector::pluralize) turns "goose" into "geese". It
    /// must agree with every other noun override or [`build`](Self::build)
    /// fails; an exact duplicate of a pair is accepted. This keeps
    /// `lemmatize(w, Noun)` equal to `singularize(w)`.
    ///
    /// ```
    /// use moult::PartOfSpeech::Verb;
    /// let inflector = moult::Inflector::builder()
    ///     .lemma("googled", Verb, "google")
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(inflector.lemmatize("Googled", Verb), "Google");
    /// ```
    pub fn lemma(mut self, form: &str, pos: PartOfSpeech, lemma: &str) -> Self {
        self.lemmas.push((form.into(), pos, lemma.into()));
        self
    }

    /// Marks `word` as returned unchanged in both directions.
    pub fn invariant(mut self, word: &str) -> Self {
        self.invariant.push(word.into());
        self
    }

    /// Lets singularize guess the singular of a word that no
    /// dictionary-checked step resolves. Off by default: such a word comes
    /// back unchanged. A capitalised word is never guessed ("Transitlines" and
    /// "TransitLines" stay unchanged, which keeps "Kubernetes" and "Athens"
    /// safe); to have one guessed, lower-case it first and accept lower-case
    /// output.
    ///
    /// With this on, an unknown word of at least four lower-case ASCII
    /// letters takes a few conservative suffix rules, accepted only when the
    /// result is a WordNet noun or ends in one (the last step of the
    /// [crate documentation](crate)). Acronym plurals ("IDs") and
    /// [`pluralize`](Inflector::pluralize) do not depend on this option, and
    /// neither does [`is_noun`](Inflector::is_noun).
    ///
    /// This is best-effort for words WordNet lacks. It can produce a
    /// non-word for a compound whose last part is a noun ("afterhours" gives
    /// "afterhour"). Leave it off when outputs are shown to users without
    /// review.
    ///
    /// ```
    /// let guessing = moult::Inflector::builder().guess_unknown(true).build().unwrap();
    /// assert_eq!(guessing.singularize("microservices"), "microservice");
    /// // The free functions and a default inflector leave the word alone.
    /// assert_eq!(moult::singularize("microservices"), "microservices");
    /// ```
    pub fn guess_unknown(mut self, yes: bool) -> Self {
        self.guess_unknown = yes;
        self
    }

    /// Checks the overrides and builds the [`Inflector`].
    ///
    /// Words are lower-cased. It fails when a word is empty or contains
    /// whitespace, when a pair has the same singular and plural (use
    /// [`invariant`](Self::invariant)), when two pairs share a singular or a
    /// plural, or when one word plays two roles: an invariant that is also in
    /// a pair, or a plural of one pair that is the singular of another. An
    /// invariant word given twice is accepted. The same holds for lemma
    /// overrides: a form or lemma that is empty or contains whitespace, a
    /// noun form equal to its lemma, and two lemmas for one form and part of speech,
    /// nouns included ([`OverrideError::DuplicateLemma`]), are rejected; a noun
    /// lemma override is checked as the pair it stands for.
    pub fn build(self) -> Result<Inflector, OverrideError> {
        let lower = |w: &String| -> Result<Box<str>, OverrideError> {
            if w.is_empty() || w.chars().any(char::is_whitespace) {
                return Err(OverrideError::InvalidWord(w.clone()));
            }
            Ok(w.to_lowercase().into_boxed_str())
        };
        let mut by_singular = Vec::with_capacity(self.pairs.len() + self.lemmas.len());
        for (s, p) in &self.pairs {
            let (s, p) = (lower(s)?, lower(p)?);
            if s == p {
                return Err(OverrideError::IdenticalPair(s.into()));
            }
            by_singular.push((s, p));
        }
        let mut stated = Vec::with_capacity(self.lemmas.len());
        for (form, pos, lemma) in &self.lemmas {
            let (form, lemma) = (lower(form)?, lower(lemma)?);
            // A noun lemma override becomes a pair, and a pair needs two words.
            if form == lemma && *pos == PartOfSpeech::Noun {
                return Err(OverrideError::IdenticalPair(form.into()));
            }
            stated.push((form, *pos, lemma));
        }
        stated.sort();
        // Two statements for one form and part of speech, nouns included.
        if let Some(w) = stated
            .windows(2)
            .find(|w| w[0].0 == w[1].0 && w[0].1 == w[1].1)
        {
            return Err(OverrideError::DuplicateLemma(w[0].0.clone().into()));
        }
        let mut lemmas = Vec::new();
        for (form, pos, lemma) in stated {
            if pos == PartOfSpeech::Noun {
                // The lemma of a noun is its singular. A pair that states the
                // same thing is the same statement, not a second one.
                if !by_singular.contains(&(lemma.clone(), form.clone())) {
                    by_singular.push((lemma, form));
                }
            } else {
                lemmas.push((form, pos, lemma));
            }
        }
        let mut invariant = Vec::with_capacity(self.invariant.len());
        for w in &self.invariant {
            invariant.push(lower(w)?);
        }
        by_singular.sort();
        invariant.sort();
        invariant.dedup();
        let mut by_plural: Vec<(Box<str>, Box<str>)> = by_singular
            .iter()
            .map(|(s, p)| (p.clone(), s.clone()))
            .collect();
        by_plural.sort();
        if let Some(w) = adjacent_duplicate(by_singular.iter().map(|p| &p.0)) {
            return Err(OverrideError::DuplicateSingular(w.into()));
        }
        if let Some(w) = adjacent_duplicate(by_plural.iter().map(|p| &p.0)) {
            return Err(OverrideError::DuplicatePlural(w.into()));
        }
        let singular = |w: &str| by_singular.binary_search_by(|(s, _)| (**s).cmp(w)).is_ok();
        let plural = |w: &str| by_plural.binary_search_by(|(p, _)| (**p).cmp(w)).is_ok();
        for (_, p) in &by_singular {
            if singular(p) {
                return Err(OverrideError::Conflict(p.clone().into()));
            }
        }
        for w in &invariant {
            if singular(w) || plural(w) {
                return Err(OverrideError::Conflict(w.clone().into()));
            }
        }
        Ok(Inflector {
            by_singular,
            by_plural,
            invariant,
            lemmas,
            guess_unknown: self.guess_unknown,
        })
    }
}

fn adjacent_duplicate<'a>(mut words: impl Iterator<Item = &'a Box<str>>) -> Option<&'a str> {
    let mut prev = words.next()?;
    for w in words {
        if w == prev {
            return Some(w);
        }
        prev = w;
    }
    None
}

/// Why [`InflectorBuilder::build`] rejected a set of overrides. Each variant
/// carries the offending word, lower-cased except for `InvalidWord`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverrideError {
    /// The word is empty or contains whitespace.
    InvalidWord(String),
    /// A pair whose singular and plural are the same word.
    IdenticalPair(String),
    /// Two pairs share this singular.
    DuplicateSingular(String),
    /// Two pairs share this plural.
    DuplicatePlural(String),
    /// The word has two roles: an invariant that is also in a pair, or a
    /// plural of one pair that is the singular of another.
    Conflict(String),
    /// Two lemma overrides share this form and part of speech.
    DuplicateLemma(String),
}

impl fmt::Display for OverrideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverrideError::InvalidWord(w) => {
                write!(f, "override word {w:?} is empty or contains whitespace")
            }
            OverrideError::IdenticalPair(w) => {
                write!(
                    f,
                    "override pair {w:?} has the same singular and plural; use an invariant"
                )
            }
            OverrideError::DuplicateSingular(w) => {
                write!(f, "two override pairs have the singular {w:?}")
            }
            OverrideError::DuplicatePlural(w) => {
                write!(f, "two override pairs have the plural {w:?}")
            }
            OverrideError::Conflict(w) => {
                write!(f, "override word {w:?} has conflicting roles")
            }
            OverrideError::DuplicateLemma(w) => {
                write!(f, "two override lemmas have the form {w:?}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn build(pairs: &[(&str, &str)], invariant: &[&str]) -> Result<Inflector, OverrideError> {
        let mut b = Inflector::builder();
        for (s, p) in pairs {
            b = b.pair(s, p);
        }
        for w in invariant {
            b = b.invariant(w);
        }
        b.build()
    }

    #[test]
    fn overrides_win_and_keep_case() {
        let inf = build(&[("sku", "skus"), ("cactus", "cactuses")], &["staff"]).unwrap();
        assert_eq!(inf.singularize("SKUs"), "SKU");
        assert_eq!(inf.singularize("Skus"), "Sku");
        assert_eq!(inf.pluralize("SKU"), "SKUs");
        assert_eq!(inf.pluralize("cactus"), "cactuses");
        assert_eq!(inf.singularize("cactuses"), "cactus");
        assert_eq!(inf.singularize("staff"), "staff");
        assert_eq!(inf.pluralize("Staff"), "Staff");
        // Overrides leave the rest alone.
        assert_eq!(inf.pluralize("mouse"), "mice");
        assert!(inf.is_noun("sku") && inf.is_noun("staff") && !inf.is_noun("blorp"));
    }

    #[test]
    fn rejects_bad_overrides() {
        use OverrideError::*;
        assert_eq!(build(&[("", "x")], &[]).err(), Some(InvalidWord("".into())));
        assert_eq!(
            build(&[], &["two words"]).err(),
            Some(InvalidWord("two words".into()))
        );
        assert_eq!(
            build(&[("Sku", "SKU")], &[]).err(),
            Some(IdenticalPair("sku".into()))
        );
        assert_eq!(
            build(&[("a", "b"), ("A", "c")], &[]).err(),
            Some(DuplicateSingular("a".into()))
        );
        assert_eq!(
            build(&[("a", "c"), ("b", "C")], &[]).err(),
            Some(DuplicatePlural("c".into()))
        );
        assert_eq!(
            build(&[("a", "b"), ("b", "c")], &[]).err(),
            Some(Conflict("b".into()))
        );
        assert_eq!(
            build(&[("a", "b")], &["a"]).err(),
            Some(Conflict("a".into()))
        );
        assert_eq!(
            build(&[("a", "b")], &["B"]).err(),
            Some(Conflict("b".into()))
        );
        assert!(build(&[], &["x", "X"]).is_ok());
        assert!(OverrideError::Conflict("b".into())
            .to_string()
            .contains("\"b\""));
    }

    #[test]
    fn lemma_overrides() {
        use PartOfSpeech::*;
        let inf = Inflector::builder()
            .lemma("Googled", Verb, "Google")
            .lemma("geese", Noun, "goose")
            .lemma("saw", Verb, "saw")
            .lemma("fastest", Adjective, "quick")
            .build()
            .unwrap();
        assert_eq!(inf.lemmatize("googled", Verb), "google");
        assert_eq!(
            inf.lemmas_for("GOOGLED", Verb).first().unwrap().text,
            "google"
        );
        assert_eq!(inf.lemmatize("fastest", Adjective), "quick");
        assert_eq!(inf.lemmatize("geese", Noun), inf.singularize("geese"));
        assert_eq!(inf.pluralize("goose"), "geese");
        let first = |w: &str, p| inf.lemmas_for(w, p).map(|l| l.text).next();
        assert_eq!(first("fastest", Adjective), Some("quick"));
        assert_eq!(first("fastest", Verb), None);
        use OverrideError::*;
        let err = |b: InflectorBuilder| b.build().err();
        assert_eq!(
            err(Inflector::builder().lemma("a b", Verb, "x")),
            Some(InvalidWord("a b".into()))
        );
        // A noun lemma override equal to its form is an identical pair; for
        // the other parts of speech it makes the word its own first lemma.
        assert_eq!(
            err(Inflector::builder().lemma("Run", Noun, "run")),
            Some(IdenticalPair("run".into()))
        );
        assert_eq!(inf.lemmatize("saw", Verb), "saw");
        assert_eq!(first("saw", Verb), Some("saw"));
        assert_eq!(Inflector::new().lemmatize("saw", Verb), "see");
        // Duplicates are rejected for every part of speech, nouns included.
        assert_eq!(
            err(Inflector::builder()
                .lemma("geese", Noun, "goose")
                .lemma("Geese", Noun, "goose")),
            Some(DuplicateLemma("geese".into()))
        );
        assert_eq!(
            err(Inflector::builder()
                .lemma("geese", Noun, "goose")
                .lemma("geese", Noun, "gander")),
            Some(DuplicateLemma("geese".into()))
        );
        assert_eq!(
            err(Inflector::builder()
                .lemma("x", Verb, "a")
                .lemma("X", Verb, "b")),
            Some(DuplicateLemma("x".into()))
        );
        // The same form as another part of speech is not a duplicate.
        assert!(Inflector::builder()
            .lemma("x", Verb, "a")
            .lemma("x", Adjective, "b")
            .build()
            .is_ok());
        // A noun lemma override must agree with the pairs.
        assert!(Inflector::builder()
            .pair("goose", "geese")
            .lemma("geese", Noun, "goose")
            .build()
            .is_ok());
        assert_eq!(
            err(Inflector::builder()
                .pair("goose", "gooses")
                .lemma("geese", Noun, "goose")),
            Some(DuplicateSingular("goose".into()))
        );
        assert!(OverrideError::DuplicateLemma("x".into())
            .to_string()
            .contains("\"x\""));
    }

    #[test]
    fn lemmatize_keeps_case_and_borrows_when_unchanged() {
        use PartOfSpeech::*;
        let inf = Inflector::new();
        for (w, p, want) in [
            ("made", Verb, "make"),
            ("running", Verb, "run"),
            ("stopped", Verb, "stop"),
            ("autopsied", Verb, "autopsy"),
            ("hating", Verb, "hate"),
            ("found", Verb, "find"),
            ("lay", Verb, "lay"),
            ("better", Adjective, "good"),
            ("better", Adverb, "well"),
            ("later", Adjective, "late"),
            ("modest", Adjective, "modest"),
            ("Went", Verb, "Go"),
            ("MADE", Verb, "MAKE"),
            ("qwzzx", Verb, "qwzzx"),
            ("two words", Verb, "two words"),
        ] {
            assert_eq!(inf.lemmatize(w, p), want, "{w}");
        }
        assert!(matches!(inf.lemmatize("qwzzx", Verb), Cow::Borrowed(_)));
        assert!(matches!(inf.lemmatize("lay", Verb), Cow::Borrowed(_)));
        assert!(inf.lemmas("two words").is_empty());
        assert!(inf.lemmas("").is_empty());
    }

    #[test]
    fn is_send_and_sync() {
        fn check<T: Send + Sync>() {}
        check::<Inflector>();
        check::<OverrideError>();
    }
}
