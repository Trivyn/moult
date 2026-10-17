//! The caller-supplied override layer, and the resolution order that puts it
//! first.

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::case::{restore, Lower, Origin};
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
#[derive(Clone, Debug, Default)]
pub struct Inflector {
    /// `(singular, plural)`, sorted by singular.
    by_singular: Vec<(Box<str>, Box<str>)>,
    /// `(plural, singular)`, sorted by plural.
    by_plural: Vec<(Box<str>, Box<str>)>,
    /// Sorted.
    invariant: Vec<Box<str>>,
    /// Whether singularize guesses at words no dictionary-checked step
    /// resolves; see [`InflectorBuilder::guess_unknown`].
    guess_unknown: bool,
}

/// What an override says about a word.
enum Over<'a> {
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
    fn over_singular(&self, key: &str) -> Option<Over<'_>> {
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
    /// The order of resolution is in the [crate documentation](crate).
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
    /// resolution is in the [crate documentation](crate).
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
        if key.is_empty() || is_inert(word) || is_invariant(key) || reads_as_plural(word, key) {
            return Cow::Borrowed(word);
        }
        let (plural, origin) = plural(key);
        changed(word, key, &plural, origin)
    }

    /// Whether `word` is a noun this inflector knows: an override word, a
    /// word in Open English WordNet's noun or name lists, or a word that
    /// the overrides, the invariant list, the irregular forms or the checked
    /// rules reduce to a WordNet noun. The unknown-word fallback does not
    /// count, whether or not [`guess_unknown`](InflectorBuilder::guess_unknown)
    /// is on, so `false` means the answer is a guess or the word is left alone.
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
fn is_inert(word: &str) -> bool {
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
    guess_unknown: bool,
}

impl InflectorBuilder {
    /// Overrides the plural of `singular`: singularize turns `plural` into
    /// `singular`, and pluralize turns `singular` into `plural`.
    pub fn pair(mut self, singular: &str, plural: &str) -> Self {
        self.pairs.push((singular.into(), plural.into()));
        self
    }

    /// Marks `word` as returned unchanged in both directions.
    pub fn invariant(mut self, word: &str) -> Self {
        self.invariant.push(word.into());
        self
    }

    /// Lets singularize guess the singular of a word that no
    /// dictionary-checked step resolves. Off by default: such a word comes
    /// back unchanged.
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
    /// invariant word given twice is accepted.
    pub fn build(self) -> Result<Inflector, OverrideError> {
        let lower = |w: &String| -> Result<Box<str>, OverrideError> {
            if w.is_empty() || w.chars().any(char::is_whitespace) {
                return Err(OverrideError::InvalidWord(w.clone()));
            }
            Ok(w.to_lowercase().into_boxed_str())
        };
        let mut by_singular = Vec::with_capacity(self.pairs.len());
        for (s, p) in &self.pairs {
            let (s, p) = (lower(s)?, lower(p)?);
            if s == p {
                return Err(OverrideError::IdenticalPair(s.into()));
            }
            by_singular.push((s, p));
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
    fn is_send_and_sync() {
        fn check<T: Send + Sync>() {}
        check::<Inflector>();
        check::<OverrideError>();
    }
}
