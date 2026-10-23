//! Third person singular resolution after the caller's overrides.

use alloc::borrow::Cow;

use crate::inflector::{is_inert, Inflector};
use crate::lookup::find;
use crate::pos::PartOfSpeech::Verb;
use crate::{data, policy, rules};

/// Steps 1 to 5 for `word`, lower-cased as `key`. `None` keeps the input.
pub(crate) fn resolve(
    inflector: &Inflector,
    word: &str,
    key: &str,
    guess_unknown: bool,
) -> Option<Cow<'static, str>> {
    if is_inert(word) || !word.ends_with(char::is_alphanumeric) {
        return None;
    }
    if policy::MODALS.binary_search(&key).is_ok() {
        return None;
    }
    if let Ok(i) = policy::THIRD_PERSON.binary_search_by(|&(v, _)| v.cmp(key)) {
        return Some(Cow::Borrowed(policy::THIRD_PERSON[i].1));
    }
    if find(data::VERBS, key)
        .or_else(|| find(data::VERB_NAMES, key))
        .is_some()
    {
        return Some(Cow::Owned(rules::third_person(
            key,
            policy::O_ES.binary_search(&key).is_ok(),
        )));
    }
    if guess_unknown
        && word.len() >= 4
        && word.bytes().all(|b| b.is_ascii_lowercase())
        && !(word.ends_with('s') && !word.ends_with("ss"))
        && !word.ends_with("ed")
        && !word.ends_with("ing")
        && inflector.lemmas_for(word, Verb).is_empty()
    {
        return Some(Cow::Owned(rules::third_person(key, false)));
    }
    None
}
