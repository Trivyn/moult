//! The parts of speech lemmatization distinguishes, and the fixed capacity of a
//! candidate list.

/// A part of speech of an English word, as Open English WordNet distinguishes
/// them. WordNet's adjective satellites count as adjectives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PartOfSpeech {
    /// A noun.
    Noun,
    /// A verb.
    Verb,
    /// An adjective, including WordNet's adjective satellites.
    Adjective,
    /// An adverb.
    Adverb,
}

/// The most lemma candidates one input may have. The candidates the tables
/// and rules produce for one input may use at most `LEMMA_CAPACITY - 4`,
/// which leaves one slot per part of speech for a caller's override. The data
/// generator refuses to write tables that exceed it.
pub(crate) const LEMMA_CAPACITY: usize = 16;
