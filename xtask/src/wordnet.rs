//! Stream the WN-LMF XML and keep what the tables need from every entry, by
//! part of speech.

use flate2::read::GzDecoder;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::pos::PartOfSpeech;

#[derive(Default)]
pub struct Stats {
    pub noun_entries: usize,
    pub multi_word: usize,
    pub underscore: usize,
    pub noun_forms_total: usize,
}

/// What the lemma tables keep for one non-noun part of speech.
#[derive(Default)]
pub struct PosTables {
    /// Entries of this part of speech, before any filtering.
    pub entries: usize,
    /// Entries with a space in the lemma (dropped).
    pub multi_word: usize,
    /// `<Form>` elements of all entries of this part of speech.
    pub forms_total: usize,
    /// Single-token lemmas written entirely in lower case.
    pub lemmas: BTreeSet<String>,
    /// Lower-cased single-token lemmas written with some upper case and not
    /// also written entirely in lower case in this part of speech. Known
    /// words, like NAMES for nouns: never the target of a suffix rule.
    pub capitalised: BTreeSet<String>,
    /// `(form, lemma)`, both lower-cased and single-token, from entries whose
    /// lemma is in `lemmas` or `capitalised`.
    pub forms: BTreeSet<(String, String)>,
    /// Forms and senses of capitalised entries, held until every entry has
    /// been seen and it is known which of them are capitalised-only.
    pending: Vec<(String, u32, Vec<String>)>,
    /// Lower-cased lemma to its sense count in this part of speech.
    pub senses: BTreeMap<String, u32>,
}

impl PosTables {
    /// Whether `w` is a lemma, or a capitalised-only one.
    pub fn is_known(&self, w: &str) -> bool {
        self.lemmas.contains(w) || self.capitalised.contains(w)
    }

    /// Keep the forms and senses of the capitalised-only entries.
    fn finish(&mut self) {
        for (lc, senses, forms) in std::mem::take(&mut self.pending) {
            if self.lemmas.contains(&lc) {
                continue;
            }
            self.capitalised.insert(lc.clone());
            *self.senses.entry(lc.clone()).or_insert(0) += senses;
            for f in forms {
                let f = f.to_lowercase();
                if !f.contains(char::is_whitespace) {
                    self.forms.insert((f, lc.clone()));
                }
            }
        }
    }

    pub fn senses_of(&self, w: &str) -> u32 {
        self.senses.get(w).copied().unwrap_or(0)
    }
}

#[derive(Default)]
pub struct Wordnet {
    /// Single-token lemmas written entirely in lower case.
    pub nouns: BTreeSet<String>,
    /// Single-token lemmas written with some upper case whose lower-cased
    /// spelling is not in `nouns`.
    pub names: BTreeSet<String>,
    /// Lower-cased single-token lemmas ending in `s` of entries of any other
    /// part of speech, minus every word in `nouns` or `names`.
    pub others: BTreeSet<String>,
    /// `(form, lemma)`, both lower-cased, from single-token noun entries.
    pub forms: BTreeSet<(String, String)>,
    /// Lower-cased lemma to its total sense count.
    pub senses: BTreeMap<String, u32>,
    pub verb: PosTables,
    /// Adjectives, adjective satellites included.
    pub adjective: PosTables,
    pub adverb: PosTables,
    pub stats: Stats,
}

/// The parts of speech with their own lemma tables.
pub const LEMMA_POS: [PartOfSpeech; 3] = [
    PartOfSpeech::Verb,
    PartOfSpeech::Adjective,
    PartOfSpeech::Adverb,
];

impl Wordnet {
    /// The lemma tables of a part of speech other than noun.
    ///
    /// # Panics
    ///
    /// On `PartOfSpeech::Noun`, whose tables are the fields of `Wordnet`.
    pub fn of(&self, pos: PartOfSpeech) -> &PosTables {
        match pos {
            PartOfSpeech::Verb => &self.verb,
            PartOfSpeech::Adjective => &self.adjective,
            PartOfSpeech::Adverb => &self.adverb,
            _ => panic!("nouns have no PosTables"),
        }
    }

    pub fn is_noun(&self, w: &str) -> bool {
        self.nouns.contains(w)
    }

    pub fn is_known(&self, w: &str) -> bool {
        self.nouns.contains(w) || self.names.contains(w)
    }

    pub fn senses_of(&self, w: &str) -> u32 {
        self.senses.get(w).copied().unwrap_or(0)
    }
}

struct Entry {
    pos: PartOfSpeech,
    lemma: String,
    senses: u32,
    forms: Vec<String>,
}

fn attr(e: &BytesStart, key: &str) -> Result<Option<String>, String> {
    for a in e.attributes() {
        let a = a.map_err(|e| format!("xml attribute: {e}"))?;
        if a.key.as_ref() == key {
            let v = a
                .normalized_value(quick_xml::XmlVersion::Explicit1_0)
                .map_err(|e| format!("xml attribute: {e}"))?;
            return Ok(Some(v.into_owned()));
        }
    }
    Ok(None)
}

pub fn parse(path: &Path) -> Result<Wordnet, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_reader(BufReader::new(GzDecoder::new(file)))
}

/// Parse WN-LMF XML from any buffered reader (the release is gzipped; tests
/// pass plain XML).
pub fn parse_reader<R: BufRead>(input: R) -> Result<Wordnet, String> {
    let mut reader = Reader::from_reader(input);
    let mut buf = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    let mut current: Option<Entry> = None;
    let mut others: Vec<String> = Vec::new();
    // Between a `LexicalEntry` start and its `Lemma`.
    let mut pending = false;
    loop {
        let ev = reader
            .read_event_into(&mut buf)
            .map_err(|e| format!("xml at byte {}: {e}", reader.buffer_position()))?;
        match &ev {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                "LexicalEntry" => {
                    current = None;
                    pending = true;
                }
                "Lemma" => {
                    if let (true, Some(code)) = (pending, attr(e, "partOfSpeech")?) {
                        let pos = match code.as_str() {
                            "n" => Some(PartOfSpeech::Noun),
                            "v" => Some(PartOfSpeech::Verb),
                            "a" | "s" => Some(PartOfSpeech::Adjective),
                            "r" => Some(PartOfSpeech::Adverb),
                            _ => None,
                        };
                        let written = attr(e, "writtenForm")?.unwrap_or_default();
                        if code != "n" && !written.is_empty() {
                            others.push(written.clone());
                        }
                        current = pos.map(|pos| Entry {
                            pos,
                            lemma: written,
                            senses: 0,
                            forms: Vec::new(),
                        });
                    }
                }
                "Form" => {
                    if let Some(cur) = current.as_mut() {
                        if let Some(f) = attr(e, "writtenForm")? {
                            cur.forms.push(f);
                        }
                    }
                }
                "Sense" => {
                    if let Some(cur) = current.as_mut() {
                        cur.senses += 1;
                    }
                }
                _ => {}
            },
            Event::End(e) if e.name().as_ref() == "LexicalEntry" => {
                if let Some(cur) = current.take() {
                    entries.push(cur);
                }
                pending = false;
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(build(entries, others))
}

fn build(entries: Vec<Entry>, other_lemmas: Vec<String>) -> Wordnet {
    let mut wn = Wordnet::default();
    let mut capitalised = BTreeSet::new();
    for e in entries {
        if e.pos != PartOfSpeech::Noun {
            wn.add_lemma_entry(e);
            continue;
        }
        wn.stats.noun_entries += 1;
        wn.stats.noun_forms_total += e.forms.len();
        if e.lemma.contains('_') {
            wn.stats.underscore += 1;
        }
        if e.lemma.contains(' ') {
            wn.stats.multi_word += 1;
            continue;
        }
        let lc = e.lemma.to_lowercase();
        if lc == e.lemma {
            wn.nouns.insert(lc.clone());
        } else {
            capitalised.insert(lc.clone());
        }
        *wn.senses.entry(lc.clone()).or_insert(0) += e.senses;
        for f in e.forms {
            wn.forms.insert((f.to_lowercase(), lc.clone()));
        }
    }
    for tables in [&mut wn.verb, &mut wn.adjective, &mut wn.adverb] {
        tables.finish();
    }
    wn.names = capitalised.difference(&wn.nouns).cloned().collect();
    wn.others = other_lemmas
        .iter()
        .filter(|w| !w.contains(char::is_whitespace) && !w.contains('_'))
        .map(|w| w.to_lowercase())
        .filter(|w| w.ends_with('s') && !wn.nouns.contains(w) && !wn.names.contains(w))
        .collect();
    wn
}

impl Wordnet {
    /// Add a verb, adjective or adverb entry: single-token lemmas, with their
    /// lower-cased single-token forms and senses. Those written with upper case
    /// wait in `pending` until `finish`.
    fn add_lemma_entry(&mut self, e: Entry) {
        let tables = match e.pos {
            PartOfSpeech::Verb => &mut self.verb,
            PartOfSpeech::Adjective => &mut self.adjective,
            PartOfSpeech::Adverb => &mut self.adverb,
            _ => return,
        };
        tables.entries += 1;
        tables.forms_total += e.forms.len();
        if e.lemma.contains(' ') {
            tables.multi_word += 1;
            return;
        }
        let lc = e.lemma.to_lowercase();
        if lc != e.lemma {
            tables.pending.push((lc, e.senses, e.forms));
            return;
        }
        tables.lemmas.insert(e.lemma.clone());
        *tables.senses.entry(e.lemma.clone()).or_insert(0) += e.senses;
        for f in e.forms {
            let f = f.to_lowercase();
            if !f.contains(char::is_whitespace) {
                tables.forms.insert((f, e.lemma.clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_nouns_names_and_multiword() {
        let entry = |lemma: &str, senses, forms: &[&str]| Entry {
            pos: PartOfSpeech::Noun,
            lemma: lemma.into(),
            senses,
            forms: forms.iter().map(|s| s.to_string()).collect(),
        };
        let wn = build(
            vec![
                entry("mouse", 2, &["mice"]),
                entry("Mouse", 1, &[]),
                entry("Kansas", 1, &[]),
                entry("ice cream", 1, &[]),
            ],
            vec![
                "Perhaps".into(),
                "mouse".into(),
                "go on".into(),
                "kansas".into(),
                "run".into(),
            ],
        );
        assert!(wn.is_noun("mouse") && !wn.is_noun("kansas"));
        assert!(wn.names.contains("kansas") && !wn.names.contains("mouse"));
        assert_eq!(wn.stats.multi_word, 1);
        // other parts of speech: only `-s` words that are neither noun nor name
        assert_eq!(wn.others.iter().collect::<Vec<_>>(), ["perhaps"]);
        assert_eq!(wn.senses_of("mouse"), 3);
        assert!(wn.forms.contains(&("mice".into(), "mouse".into())));
    }

    #[test]
    fn keeps_verbs_adjectives_and_adverbs_apart() {
        let entry = |pos, lemma: &str, senses, forms: &[&str]| Entry {
            pos,
            lemma: lemma.into(),
            senses,
            forms: forms.iter().map(|s| s.to_string()).collect(),
        };
        use PartOfSpeech::*;
        let wn = build(
            vec![
                entry(Noun, "ferry", 1, &[]),
                entry(Verb, "ferry", 2, &["Ferried"]),
                entry(Verb, "ferry", 1, &[]),
                entry(Verb, "Iowa", 1, &["Iowas"]),
                entry(Verb, "Ferry", 1, &["Ferries"]),
                entry(Verb, "go on", 1, &["goes on"]),
                entry(Adjective, "good", 5, &["better"]),
                entry(Adverb, "well", 3, &["better", "best of all"]),
            ],
            vec![],
        );
        assert_eq!(wn.verb.lemmas.iter().collect::<Vec<_>>(), ["ferry"]);
        // "Iowa" is capitalised-only: known, with its forms, but not a lemma
        assert_eq!(wn.verb.capitalised.iter().collect::<Vec<_>>(), ["iowa"]);
        assert!(wn.verb.is_known("iowa") && !wn.verb.lemmas.contains("iowa"));
        assert!(wn.verb.forms.contains(&("iowas".into(), "iowa".into())));
        assert_eq!(wn.verb.senses_of("ferry"), 3);
        assert_eq!(wn.verb.entries, 5);
        // "Ferry" is shadowed by "ferry": neither capitalised-only nor its forms
        assert!(!wn.verb.forms.contains(&("ferries".into(), "ferry".into())));
        assert_eq!(wn.verb.multi_word, 1);
        assert!(wn.verb.forms.contains(&("ferried".into(), "ferry".into())));
        assert_eq!(wn.verb.forms.len(), 2);
        assert!(wn
            .adjective
            .forms
            .contains(&("better".into(), "good".into())));
        assert!(wn.adverb.forms.contains(&("better".into(), "well".into())));
        assert_eq!(wn.adverb.forms.len(), 1);
        // the noun tables see only the noun entry
        assert!(wn.forms.is_empty() && wn.is_noun("ferry"));
        assert_eq!(wn.senses_of("ferry"), 1);
        assert_eq!(wn.of(Adverb).senses_of("well"), 3);
    }
}
