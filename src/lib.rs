//! English noun inflection checked against a dictionary.
//!
//! `moult` turns an English noun into its singular or plural form. Suffix
//! rules alone over-fire ("waves" → "wafe", "police" → "polouse") or
//! under-fire ("analysis" → "analysi"). `moult` accepts a rule's output only
//! when it is a noun [Open English WordNet] knows, and reads irregular forms
//! from WordNet's own form list. Rules apply without a check only to a word
//! WordNet does not contain.
//!
//! Singularize resolves a word in this order; the first step that answers
//! wins:
//!
//! 1. **Invariant nouns**: a closed list of nouns whose plural is the
//!    singular, or that read as mass nouns ("data", "news", "series",
//!    "logistics"). Returned unchanged.
//! 2. **Irregular forms**: WordNet's form list ("mice" → "mouse",
//!    "criteria" → "criterion").
//! 3. **Checked rules**: a suffix rule whose output is a WordNet noun
//!    ("waves" → "wave", "statuses" → "status").
//! 4. **Known noun**: a WordNet noun no checked rule reduces is already
//!    singular ("gas", "lens", "analysis").
//! 5. **Unknown word**: the conservative suffix rules, unchecked.
//!
//! Pluralize runs the irregular forms in reverse, then the regular rules.
//! Both directions keep the input's case: an acronym stays upper case
//! ("ISBNs" → "ISBN") and a capitalised word stays capitalised.
//!
//! The functions read a single word. Splitting a compound or an identifier
//! ("OrderLines", "farm_workers") and inflecting its head is the caller's
//! choice.
//!
//! **Status: skeleton.** The API is fixed; the tables and the resolution
//! order are not implemented yet, and both functions panic.
//!
//! [Open English WordNet]: https://en-word.net/

#![no_std]

extern crate alloc;

use alloc::string::String;

mod data;

/// The singular form of the English noun `word`.
pub fn singularize(word: &str) -> String {
    let _ = (word, data::NOUNS, data::IRREGULAR);
    todo!("moult skeleton: singularize is not implemented")
}

/// The plural form of the English noun `word`.
pub fn pluralize(word: &str) -> String {
    let _ = word;
    todo!("moult skeleton: pluralize is not implemented")
}
