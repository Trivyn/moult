//! Stream the WN-LMF XML and keep what the tables need from noun entries.

use flate2::read::GzDecoder;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Default)]
pub struct Stats {
    pub noun_entries: usize,
    pub multi_word: usize,
    pub underscore: usize,
    pub noun_forms_total: usize,
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
    pub stats: Stats,
}

impl Wordnet {
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
    let mut noun = false;
    loop {
        let ev = reader
            .read_event_into(&mut buf)
            .map_err(|e| format!("xml at byte {}: {e}", reader.buffer_position()))?;
        match &ev {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                "LexicalEntry" => {
                    current = Some(Entry {
                        lemma: String::new(),
                        senses: 0,
                        forms: Vec::new(),
                    });
                    noun = false;
                }
                "Lemma" => {
                    if let (Some(cur), Some(pos)) = (current.as_mut(), attr(e, "partOfSpeech")?) {
                        if pos == "n" {
                            noun = true;
                            cur.lemma = attr(e, "writtenForm")?.unwrap_or_default();
                        } else if let Some(w) = attr(e, "writtenForm")? {
                            others.push(w);
                        }
                    }
                }
                "Form" => {
                    if let (Some(cur), true) = (current.as_mut(), noun) {
                        if let Some(f) = attr(e, "writtenForm")? {
                            cur.forms.push(f);
                        }
                    }
                }
                "Sense" => {
                    if let (Some(cur), true) = (current.as_mut(), noun) {
                        cur.senses += 1;
                    }
                }
                _ => {}
            },
            Event::End(e) if e.name().as_ref() == "LexicalEntry" => {
                if let (Some(cur), true) = (current.take(), noun) {
                    entries.push(cur);
                }
                noun = false;
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
    wn.names = capitalised.difference(&wn.nouns).cloned().collect();
    wn.others = other_lemmas
        .iter()
        .filter(|w| !w.contains(char::is_whitespace) && !w.contains('_'))
        .map(|w| w.to_lowercase())
        .filter(|w| w.ends_with('s') && !wn.nouns.contains(w) && !wn.names.contains(w))
        .collect();
    wn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_nouns_names_and_multiword() {
        let entry = |lemma: &str, senses, forms: &[&str]| Entry {
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
}
