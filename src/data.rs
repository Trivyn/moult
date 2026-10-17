//! The word tables, derived from Open English WordNet (CC-BY-4.0; see
//! `LICENSE-DATA`). The data generator will write this file from the pinned
//! release in `data/MANIFEST.toml`; it is committed, so building the crate
//! needs no network and no WordNet install. Until the generator exists the
//! tables are empty.

/// Every noun lemma, lowercased, sorted by byte order for binary search.
pub(crate) static NOUNS: &[&str] = &[];

/// Irregular noun forms as `(plural, singular)`, sorted by plural.
pub(crate) static IRREGULAR: &[(&str, &str)] = &[];
