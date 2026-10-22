//! Lemmatization: the candidate lemmas of a word, as a fixed-size list that
//! borrows its text from the generated tables and the [`Inflector`].

use crate::inflector::{is_inert, Inflector, Over};
use crate::lookup::{find, rows};
use crate::pos::{PartOfSpeech, LEMMA_CAPACITY};
use crate::resolve::{
    is_excluded, is_man_regular, is_noun_key, is_other_only, other_only_noun, resolve, Sing,
};
use crate::{data, policy, rules};

/// One reading of a word: a lemma, in lower case, and the part of speech the
/// word has as that lemma's form.
///
/// `text` is the dictionary entry: a word of Open English WordNet, a listed
/// form of the policy lists, or an override of the [`Inflector`]. It does not
/// follow the input's case; [`Inflector::lemmatize`] does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lemma<'a> {
    /// The lemma, in lower case.
    pub text: &'a str,
    /// The part of speech of the reading.
    pub pos: PartOfSpeech,
}

/// The candidate lemmas of one word, from [`Inflector::lemmas`] and
/// [`Inflector::lemmas_for`]. An iterator of [`Lemma`] in the order set out in
/// the [crate documentation](crate#lemmatize). It has fixed storage and borrows
/// its candidate text; see the crate documentation for when producing it
/// allocates.
#[derive(Clone, Debug)]
pub struct Lemmas<'a> {
    items: [Option<Lemma<'a>>; LEMMA_CAPACITY],
    len: usize,
    next: usize,
}

impl<'a> Lemmas<'a> {
    fn new() -> Self {
        Lemmas {
            items: [None; LEMMA_CAPACITY],
            len: 0,
            next: 0,
        }
    }

    /// Adds a candidate unless the same lemma as the same part of speech is
    /// already there, so the first position wins.
    fn push(&mut self, text: &'a str, pos: PartOfSpeech) {
        let lemma = Lemma { text, pos };
        if self.items[..self.len].contains(&Some(lemma)) {
            return;
        }
        // The generator keeps the table-derived candidates of any input within
        // LEMMA_CAPACITY - 4, and overrides add at most one per part of speech.
        debug_assert!(self.len < LEMMA_CAPACITY, "too many lemma candidates");
        if self.len < LEMMA_CAPACITY {
            self.items[self.len] = Some(lemma);
            self.len += 1;
        }
    }

    /// The number of candidates not yet yielded.
    pub fn len(&self) -> usize {
        self.len - self.next
    }

    /// Whether no candidate is left.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The next candidate, which is the preferred one before any is yielded,
    /// without consuming it.
    pub fn first(&self) -> Option<Lemma<'a>> {
        self.items[..self.len].get(self.next).copied().flatten()
    }
}

impl<'a> Iterator for Lemmas<'a> {
    type Item = Lemma<'a>;

    fn next(&mut self) -> Option<Lemma<'a>> {
        let lemma = self.first()?;
        self.next += 1;
        Some(lemma)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for Lemmas<'_> {}

/// The candidates of `word` (lower-cased as `key`) for `only`, or for every
/// part of speech in the order noun, verb, adjective, adverb.
pub(crate) fn collect<'a>(
    inflector: &'a Inflector,
    word: &str,
    key: &str,
    only: Option<PartOfSpeech>,
) -> Lemmas<'a> {
    let mut out = Lemmas::new();
    // Not one word: nothing to look up.
    if word.is_empty() || word.chars().any(char::is_whitespace) {
        return out;
    }
    let wanted = |pos: PartOfSpeech| only.map_or(true, |o| o == pos);
    if wanted(PartOfSpeech::Noun) {
        nouns(inflector, key, is_inert(word), &mut out);
    }
    for pos in [
        PartOfSpeech::Verb,
        PartOfSpeech::Adjective,
        PartOfSpeech::Adverb,
    ] {
        if wanted(pos) {
            others(inflector, key, pos, &mut out);
        }
    }
    out
}

/// The entry of NOUNS or NAMES equal to `key`.
fn noun_or_name(key: &str) -> Option<&'static str> {
    find(data::NOUNS, key).or_else(|| find(data::NAMES, key))
}

/// Noun candidates: the override, singularize's checked answer, the other
/// readings, then the word itself. Every one but an override is a noun or name
/// of the dictionary. `inert` input (one character, or no letter) has only the
/// override and the word itself.
fn nouns<'a>(inflector: &'a Inflector, key: &str, inert: bool, out: &mut Lemmas<'a>) {
    match inflector.over_singular(key) {
        Some(Over::To(singular)) => out.push(singular, PartOfSpeech::Noun),
        Some(Over::Keep) => {
            if let Some(word) = inflector.override_word(key) {
                out.push(word, PartOfSpeech::Noun);
            }
        }
        None => {}
    }
    let noun = PartOfSpeech::Noun;
    let entry = |out: &mut Lemmas<'a>, word: &str| {
        if let Some(entry) = noun_or_name(word) {
            out.push(entry, noun);
        }
    };
    if inert {
        entry(out, key);
        return;
    }
    match resolve(key) {
        Sing::To(singular) => entry(out, &singular),
        Sing::Keep => entry(out, key),
        Sing::Other | Sing::Unknown => {}
    }
    for &(form, lemma) in rows(data::FORM_TO_LEMMA, key) {
        if !is_excluded(lemma, form) {
            entry(out, lemma);
        }
    }
    for &(singular, plural) in policy::IRREGULAR {
        if plural == key {
            entry(out, singular);
        }
    }
    if is_other_only(key) {
        // singularize reduces such a word only by its first checked rule and
        // only when it is that noun's plural; no other rule reading counts.
        let reduced = rules::checked_singular(key, is_noun_key, is_man_regular);
        if let Some(noun) = other_only_noun(key, reduced) {
            entry(out, &noun);
        }
    } else {
        let men = key.ends_with("men");
        rules::singular_stems(key, &mut |stem| {
            if men && is_man_regular(stem) {
                return;
            }
            if let Some(entry) = find(data::NOUNS, stem) {
                out.push(entry, noun);
            }
        });
    }
    entry(out, key);
}

/// Verb, adjective or adverb candidates: the override, the curated choice, the
/// listed forms, the rule stems that are lower-case lemmas, then the word itself
/// when it is a lemma or a capitalised-only one (which no rule reaches).
fn others<'a>(inflector: &'a Inflector, key: &str, pos: PartOfSpeech, out: &mut Lemmas<'a>) {
    let (words, names, forms) = match pos {
        PartOfSpeech::Verb => (data::VERBS, data::VERB_NAMES, data::VERB_FORMS),
        PartOfSpeech::Adjective => (
            data::ADJECTIVES,
            data::ADJECTIVE_NAMES,
            data::ADJECTIVE_FORMS,
        ),
        PartOfSpeech::Adverb => (data::ADVERBS, data::ADVERB_NAMES, data::ADVERB_FORMS),
        PartOfSpeech::Noun => return,
    };
    if let Some(lemma) = inflector.lemma_override(key, pos) {
        out.push(lemma, pos);
    }
    if let Ok(i) = policy::LEMMA_CHOICE.binary_search_by(|&(f, p, _)| (f, p).cmp(&(key, pos))) {
        out.push(policy::LEMMA_CHOICE[i].2, pos);
    }
    for &(_, lemma) in rows(forms, key) {
        out.push(lemma, pos);
    }
    let mut stem = |s: &str| {
        if let Some(entry) = find(words, s) {
            out.push(entry, pos);
        }
    };
    match pos {
        PartOfSpeech::Verb => rules::verb_stems(key, &mut stem),
        PartOfSpeech::Adjective => rules::adjective_stems(key, &mut stem),
        _ => {}
    }
    if let Some(entry) = find(words, key).or_else(|| find(names, key)) {
        out.push(entry, pos);
    }
}
