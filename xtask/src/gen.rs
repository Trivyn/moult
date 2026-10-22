//! `cargo xtask gen-data`: render the committed word tables from the parsed
//! release. `render` is a pure function; file IO lives in `write` / `check`.

use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::manifest::Manifest;
use crate::pos::PartOfSpeech;
use crate::wordnet::{PosTables, Wordnet, LEMMA_POS};

/// Paths of the generated files, relative to the crate root.
pub const NOUNS_PATH: &str = "src/nouns.txt";
pub const NAMES_PATH: &str = "src/names.txt";
pub const OTHERS_PATH: &str = "src/others.txt";
pub const VERBS_PATH: &str = "src/verbs.txt";
pub const ADJECTIVES_PATH: &str = "src/adjectives.txt";
pub const ADVERBS_PATH: &str = "src/adverbs.txt";
pub const VERB_NAMES_PATH: &str = "src/verb_names.txt";
pub const ADJECTIVE_NAMES_PATH: &str = "src/adjective_names.txt";
pub const ADVERB_NAMES_PATH: &str = "src/adverb_names.txt";
pub const DATA_PATH: &str = "src/data.rs";

/// The ten generated files' contents.
pub struct Outputs {
    pub nouns: String,
    pub names: String,
    pub others: String,
    pub verbs: String,
    pub adjectives: String,
    pub adverbs: String,
    pub verb_names: String,
    pub adjective_names: String,
    pub adverb_names: String,
    pub data: String,
}

impl Outputs {
    pub fn files(&self) -> [(&'static str, &str); 10] {
        [
            (NOUNS_PATH, &self.nouns),
            (NAMES_PATH, &self.names),
            (OTHERS_PATH, &self.others),
            (VERBS_PATH, &self.verbs),
            (ADJECTIVES_PATH, &self.adjectives),
            (ADVERBS_PATH, &self.adverbs),
            (VERB_NAMES_PATH, &self.verb_names),
            (ADJECTIVE_NAMES_PATH, &self.adjective_names),
            (ADVERB_NAMES_PATH, &self.adverb_names),
            (DATA_PATH, &self.data),
        ]
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn word_list<'a>(words: impl Iterator<Item = &'a String>) -> String {
    let mut v: Vec<&str> = words.map(String::as_str).collect();
    v.sort_unstable();
    v.dedup();
    let mut out = String::new();
    for w in v {
        out.push_str(w);
        out.push('\n');
    }
    out
}

fn pair_table(out: &mut String, name: &str, doc: &[&str], attrs: &[&str], pairs: &[(&str, &str)]) {
    for line in doc {
        let _ = writeln!(out, "/// {line}");
    }
    for attr in attrs {
        let _ = writeln!(out, "{attr}");
    }
    let _ = writeln!(out, "pub(crate) static {name}: &[(&str, &str)] = &[");
    for (a, b) in pairs {
        let _ = writeln!(out, "    ({a:?}, {b:?}),");
    }
    out.push_str("];\n");
}

/// The `(form, lemma)` pairs of a part of speech without the excluded ones,
/// byte-sorted.
fn lemma_forms<'a>(
    tab: &'a PosTables,
    pos: PartOfSpeech,
    excluded: &[(&str, PartOfSpeech, &str)],
) -> Vec<(&'a str, &'a str)> {
    let mut v: Vec<(&str, &str)> = tab
        .forms
        .iter()
        .filter(|(f, l)| !excluded.contains(&(f.as_str(), pos, l.as_str())))
        .map(|(f, l)| (f.as_str(), l.as_str()))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Render the word lists and `data.rs`. Pure and deterministic: the result
/// depends only on the set of extracted words, the manifest and the excluded
/// lemma forms.
pub fn render(
    manifest: &Manifest,
    wn: &Wordnet,
    lemma_excluded: &[(&str, PartOfSpeech, &str)],
) -> Outputs {
    let nouns = word_list(wn.nouns.iter());
    let names = word_list(wn.names.iter());
    let others = word_list(wn.others.iter());
    let verbs = word_list(wn.verb.lemmas.iter());
    let adjectives = word_list(wn.adjective.lemmas.iter());
    let adverbs = word_list(wn.adverb.lemmas.iter());
    let verb_names = word_list(wn.verb.capitalised.iter());
    let adjective_names = word_list(wn.adjective.capitalised.iter());
    let adverb_names = word_list(wn.adverb.capitalised.iter());
    let [verb_forms, adjective_forms, adverb_forms] =
        LEMMA_POS.map(|p| lemma_forms(wn.of(p), p, lemma_excluded));

    // Single-token pairs only, byte-sorted by (form, lemma).
    let mut f2l: Vec<(&str, &str)> = wn
        .forms
        .iter()
        .filter(|(f, l)| !f.contains(char::is_whitespace) && !l.contains(char::is_whitespace))
        .map(|(f, l)| (f.as_str(), l.as_str()))
        .collect();
    f2l.sort_unstable();
    f2l.dedup();
    // Indices into `f2l`, ordered by (lemma, form).
    assert!(
        f2l.len() <= usize::from(u16::MAX) + 1,
        "FORM_TO_LEMMA has {} rows; LEMMA_TO_PLURAL indices no longer fit u16",
        f2l.len()
    );
    let mut l2p: Vec<u16> = (0..f2l.len()).map(|i| i as u16).collect();
    l2p.sort_unstable_by_key(|&i| {
        let (form, lemma) = f2l[usize::from(i)];
        (lemma, form)
    });

    let n_nouns = nouns.lines().count();
    let n_names = names.lines().count();
    let n_others = others.lines().count();
    let n_verbs = verbs.lines().count();
    let n_adjectives = adjectives.lines().count();
    let n_adverbs = adverbs.lines().count();
    let n_verb_names = verb_names.lines().count();
    let n_adjective_names = adjective_names.lines().count();
    let n_adverb_names = adverb_names.lines().count();

    let mut d = String::new();
    let _ = writeln!(
        d,
        "//! @generated by `cargo xtask gen-data` -- do not edit.
//!
//! The word tables, derived from {name} {edition} (licence {licence}; see
//! `LICENSE-DATA`).
//!
//! - source url:     <{url}>
//! - source sha256:  {sha}
//! - nouns.txt sha256: {nouns_sha}
//! - names.txt sha256: {names_sha}
//! - others.txt sha256: {others_sha}
//! - verbs.txt sha256: {verbs_sha}
//! - adjectives.txt sha256: {adjectives_sha}
//! - adverbs.txt sha256: {adverbs_sha}
//! - verb_names.txt sha256: {verb_names_sha}
//! - adjective_names.txt sha256: {adjective_names_sha}
//! - adverb_names.txt sha256: {adverb_names_sha}
//!
//! Counts:
//!
//! - NOUNS: {n_nouns}
//! - NAMES: {n_names}
//! - OTHERS: {n_others}
//! - FORM_TO_LEMMA: {n_f2l}
//! - LEMMA_TO_PLURAL: {n_l2p} (indices into FORM_TO_LEMMA)
//! - VERBS: {n_verbs}
//! - ADJECTIVES: {n_adjectives}
//! - ADVERBS: {n_adverbs}
//! - VERB_NAMES: {n_verb_names}
//! - ADJECTIVE_NAMES: {n_adjective_names}
//! - ADVERB_NAMES: {n_adverb_names}
//! - VERB_FORMS: {n_vf}
//! - ADJECTIVE_FORMS: {n_af}
//! - ADVERB_FORMS: {n_rf}
//!
//! Mechanical transforms, and no others:
//!
//! - NOUNS, NAMES and FORM_TO_LEMMA come from noun entries only; lemmas and
//!   forms are lower-cased
//! - lemmas containing a space are dropped
//! - single-token noun lemmas written only with capitals go to NAMES
//! - single-token lemmas of parts of speech other than noun (verb, adjective,
//!   adjective satellite, adverb) that end in `s`, and are in neither NOUNS
//!   nor NAMES, are lower-cased and go to OTHERS
//! - noun forms are kept as (form, lemma) pairs, unfiltered
//! - VERBS, ADJECTIVES and ADVERBS are the single-token lemmas of verb
//!   entries, of adjective and adjective satellite entries, and of adverb
//!   entries that are written entirely in lower case
//! - VERB_NAMES, ADJECTIVE_NAMES and ADVERB_NAMES are the lower-cased
//!   single-token lemmas of those parts of speech that WordNet writes only
//!   with capitals, and not also in lower case in the same part of speech
//! - VERB_FORMS, ADJECTIVE_FORMS and ADVERB_FORMS are the lower-cased,
//!   single-token (form, lemma) pairs of the entries kept in VERBS,
//!   ADJECTIVES, ADVERBS and the matching NAMES list; the only pairs removed
//!   are the policy's LEMMA_EXCLUDED rows
//! - duplicates are removed and every table is sorted by byte order",
        name = manifest.name,
        edition = manifest.edition,
        licence = manifest.license,
        url = manifest.url,
        sha = manifest.sha256,
        nouns_sha = sha256_hex(nouns.as_bytes()),
        names_sha = sha256_hex(names.as_bytes()),
        others_sha = sha256_hex(others.as_bytes()),
        verbs_sha = sha256_hex(verbs.as_bytes()),
        adjectives_sha = sha256_hex(adjectives.as_bytes()),
        adverbs_sha = sha256_hex(adverbs.as_bytes()),
        verb_names_sha = sha256_hex(verb_names.as_bytes()),
        adjective_names_sha = sha256_hex(adjective_names.as_bytes()),
        adverb_names_sha = sha256_hex(adverb_names.as_bytes()),
        n_vf = verb_forms.len(),
        n_af = adjective_forms.len(),
        n_rf = adverb_forms.len(),
        n_f2l = f2l.len(),
        n_l2p = l2p.len(),
    );
    d.push('\n');
    d.push_str(
        "/// Every noun lemma written in lower case, one per line, `\\n`-terminated,\n\
         /// sorted by byte order (`nouns.txt`).\n\
         pub(crate) static NOUNS: &str = include_str!(\"nouns.txt\");\n\n\
         /// Known words that WordNet writes only with capitals, lower-cased, one per\n\
         /// line, sorted by byte order (`names.txt`). Never a rule target.\n\
         pub(crate) static NAMES: &str = include_str!(\"names.txt\");\n\n\
         /// Words that WordNet lists only under parts of speech other than\n\
         /// noun (verb, adjective, adverb) and that end in `s`, lower-cased, one per line,\n\
         /// sorted by byte order (`others.txt`). They are known words but not\n\
         /// nouns: the unknown-word fallback never applies to them.\n\
         pub(crate) static OTHERS: &str = include_str!(\"others.txt\");\n\n\
         /// Every verb lemma written in lower case, one per line, `\\n`-terminated,\n\
         /// sorted by byte order (`verbs.txt`).\n\
         pub(crate) static VERBS: &str = include_str!(\"verbs.txt\");\n\n\
         /// Every adjective lemma (satellites included) written in lower case, one per\n\
         /// line, sorted by byte order (`adjectives.txt`).\n\
         pub(crate) static ADJECTIVES: &str = include_str!(\"adjectives.txt\");\n\n\
         /// Every adverb lemma written in lower case, one per line, sorted by byte\n\
         /// order (`adverbs.txt`).\n\
         pub(crate) static ADVERBS: &str = include_str!(\"adverbs.txt\");\n\n\
         /// Verb lemmas that WordNet writes only with capitals, lower-cased, one per\n\
         /// line, sorted by byte order (`verb_names.txt`). Known words, never a rule\n\
         /// target.\n\
         pub(crate) static VERB_NAMES: &str = include_str!(\"verb_names.txt\");\n\n\
         /// The same for adjectives (`adjective_names.txt`).\n\
         pub(crate) static ADJECTIVE_NAMES: &str = include_str!(\"adjective_names.txt\");\n\n\
         /// The same for adverbs (`adverb_names.txt`).\n\
         pub(crate) static ADVERB_NAMES: &str = include_str!(\"adverb_names.txt\");\n\n",
    );
    pair_table(
        &mut d,
        "FORM_TO_LEMMA",
        &[
            "Every WordNet noun form as `(form, lemma)`, sorted by `(form, lemma)` in",
            "byte order. A form may appear in several rows.",
        ],
        &[],
        &f2l,
    );
    d.push('\n');
    d.push_str(
        "/// The same pairs as an index into `FORM_TO_LEMMA`: row numbers ordered by\n\
         /// `(lemma, form)` in byte order, so a lemma's forms are adjacent. A lemma\n\
         /// may have several rows. It avoids storing every pair twice.\n\
         #[rustfmt::skip]\n\
         pub(crate) static LEMMA_TO_PLURAL: &[u16] = &[\n",
    );
    for line in l2p.chunks(16) {
        let nums: Vec<String> = line.iter().map(u16::to_string).collect();
        let _ = writeln!(d, "    {},", nums.join(", "));
    }
    d.push_str("];\n");
    for (name, what, pairs) in [
        ("VERB_FORMS", "verb", &verb_forms),
        ("ADJECTIVE_FORMS", "adjective", &adjective_forms),
        ("ADVERB_FORMS", "adverb", &adverb_forms),
    ] {
        d.push('\n');
        pair_table(
            &mut d,
            name,
            &[
                &format!(
                    "Every WordNet {what} form as `(form, lemma)` for a lemma in the matching"
                ),
                "word list or names list, sorted by `(form, lemma)` in byte order, without the policy's",
                "`LEMMA_EXCLUDED` rows. A form may appear in several rows.",
            ],
            &[],
            pairs,
        );
    }
    Outputs {
        nouns,
        names,
        others,
        verbs,
        adjectives,
        adverbs,
        verb_names,
        adjective_names,
        adverb_names,
        data: d,
    }
}

fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let tmp = path.with_extension("tmp-gen");
    fs::write(&tmp, contents.as_bytes()).map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })
}

pub fn write(root: &Path, out: &Outputs) -> Result<(), String> {
    for (rel, contents) in out.files() {
        write_atomic(&root.join(rel), contents)?;
        eprintln!("wrote {rel} ({} bytes)", contents.len());
    }
    Ok(())
}

/// Describe the first difference between two texts.
fn first_diff(old: &str, new: &str) -> String {
    let mut a = old.lines();
    let mut b = new.lines();
    let mut n = 1;
    loop {
        match (a.next(), b.next()) {
            (Some(x), Some(y)) if x == y => n += 1,
            (None, None) => return "line endings or trailing bytes differ".to_string(),
            (x, y) => {
                return format!(
                    "first difference at line {n}:\n    committed: {}\n    generated: {}",
                    x.unwrap_or("<end of file>"),
                    y.unwrap_or("<end of file>")
                )
            }
        }
    }
}

/// Compare with the committed files; no writes.
pub fn check(root: &Path, out: &Outputs) -> Result<(), String> {
    let mut diffs = Vec::new();
    for (rel, contents) in out.files() {
        match fs::read(root.join(rel)) {
            Ok(old) if old == contents.as_bytes() => {}
            Ok(old) => diffs.push(format!(
                "{rel} differs\n  {}",
                first_diff(&String::from_utf8_lossy(&old), contents)
            )),
            Err(e) => diffs.push(format!("{rel}: {e}")),
        }
    }
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{}\nrun `cargo xtask gen-data` to regenerate",
            diffs.join("\n")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wordnet::parse_reader;

    const FIXTURE: &str = include_str!("../tests/fixtures/mini.xml");

    fn manifest() -> Manifest {
        Manifest {
            name: "Mini".into(),
            edition: "0".into(),
            license: "CC-BY-4.0".into(),
            url: "https://example.invalid/mini.xml.gz".into(),
            sha256: "0".repeat(64),
        }
    }

    fn gen(xml: &str) -> Outputs {
        render(&manifest(), &parse_reader(xml.as_bytes()).unwrap(), &[])
    }

    /// The fixture with its lexical entries in reverse order.
    fn reversed(xml: &str) -> String {
        let start = xml.find("    <LexicalEntry").unwrap();
        let end = xml.rfind("</LexicalEntry>").unwrap() + "</LexicalEntry>".len();
        let mut entries: Vec<&str> = xml[start..end].split_inclusive("</LexicalEntry>").collect();
        entries.reverse();
        format!("{}{}{}", &xml[..start], entries.concat(), &xml[end..])
    }

    #[test]
    fn deterministic_and_order_independent() {
        let a = gen(FIXTURE);
        let b = gen(FIXTURE);
        let c = gen(&reversed(FIXTURE));
        assert_ne!(reversed(FIXTURE), FIXTURE);
        for ((_, x), ((_, y), (_, z))) in a.files().iter().zip(b.files().iter().zip(c.files())) {
            assert_eq!(x, y);
            assert_eq!(*x, z);
        }
    }

    #[test]
    fn extraction_rules() {
        let o = gen(FIXTURE);
        let nouns: Vec<&str> = o.nouns.lines().collect();
        assert_eq!(
            nouns,
            [
                "café",
                "fellow-traveler",
                "ferry",
                "gander",
                "lantern",
                "o'clock",
                "pip",
                "yakk"
            ]
        );
        assert!(o.nouns.ends_with('\n') && !o.nouns.contains("\n\n"));
        // multi-word dropped, verb and adjective ignored
        assert!(!o.nouns.contains("sorbet") && !o.data.contains("sorbet"));
        assert!(!o.nouns.contains("quillate") && !o.nouns.contains("brisk"));
        let forms = o.data.find("static VERB_FORMS").unwrap();
        let noun_forms = &o.data[..forms];
        assert!(
            !noun_forms.contains("ferried"),
            "verb forms stay out of the noun table"
        );
        // capitalised-only goes to names; "Pip" is shadowed by "pip"
        assert_eq!(o.names, "zorland\n");
        // a non-noun ending in `s` lands in others; "brisk" does not end in `s`
        assert_eq!(o.others, "perhaps\n");
        assert!(o.data.contains("static OTHERS"));
        // per part of speech: lower-case single-token lemmas, satellites as adjectives
        assert_eq!(o.verbs, "ferry\npip\nquillate\n");
        assert_eq!(o.adjectives, "brisk\ndrowsy\ngood\n");
        assert_eq!(o.adverbs, "perhaps\nwell\n");
        // capitalised-only lemmas per part of speech are their own lists
        assert_eq!(o.adjective_names, "gallic\n");
        assert_eq!((o.verb_names.as_str(), o.adverb_names.as_str()), ("", ""));
        assert!(o
            .data
            .contains("static ADJECTIVE_NAMES: &str = include_str!(\"adjective_names.txt\")"));
        assert!(o.data.contains(
            "static VERB_FORMS: &[(&str, &str)] = &[\n    (\"ferried\", \"ferry\"),\n];"
        ));
        assert!(o.data.contains(
            "static ADJECTIVE_FORMS: &[(&str, &str)] = &[\n    (\"better\", \"good\"),\n];"
        ));
        assert!(o.data.contains(
            "static ADVERB_FORMS: &[(&str, &str)] = &[\n    (\"better\", \"well\"),\n];"
        ));
        // forms, with apostrophes and non-ASCII escaped correctly
        assert!(o.data.contains("    (\"gandri\", \"gander\"),\n"));
        assert!(o.data.contains("    (\"o'clocks\", \"o'clock\"),\n"));
        assert!(o.data.contains("    (\"cafés\", \"café\"),\n"));
        assert!(o
            .data
            .contains("    (\"fellows-traveler\", \"fellow-traveler\"),\n"));
        // both directions are present and sorted
        let f2l = o.data.find("static FORM_TO_LEMMA").unwrap();
        let l2p = o.data.find("static LEMMA_TO_PLURAL").unwrap();
        assert!(f2l < l2p);
        assert!(o.data[l2p..].contains("&[u16]"));
        assert!(o.data[f2l..l2p].find("\"cafés\"") < o.data[f2l..l2p].find("\"yakken\""));
    }

    #[test]
    fn lemma_excluded_rows_are_the_only_removal() {
        let wn = parse_reader(FIXTURE.as_bytes()).unwrap();
        let o = render(
            &manifest(),
            &wn,
            &[("better", PartOfSpeech::Adjective, "good")],
        );
        assert!(o
            .data
            .contains("static ADJECTIVE_FORMS: &[(&str, &str)] = &[\n];"));
        assert!(o.data.contains("(\"better\", \"well\")"));
        assert!(o.data.contains("(\"ferried\", \"ferry\")"));
        assert!(o.data.contains("ADJECTIVE_FORMS: 0"));
    }

    #[test]
    fn header_is_stable_and_timestamp_free() {
        let o = gen(FIXTURE);
        let header: String = o
            .data
            .lines()
            .take_while(|l| l.starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(header.contains("@generated by `cargo xtask gen-data`"));
        assert!(
            header.contains("VERBS: 3")
                && header.contains("ADJECTIVES: 3")
                && header.contains("ADVERBS: 2")
                && header.contains("ADJECTIVE_NAMES: 1")
                && header.contains("VERB_NAMES: 0")
                && header.contains("VERB_FORMS: 1")
                && header.contains(&sha256_hex(o.verbs.as_bytes()))
                && header.contains("NOUNS: 8")
                && header.contains("NAMES: 1")
                && header.contains("OTHERS: 1")
        );
        assert!(header.contains(&sha256_hex(o.nouns.as_bytes())));
        for needle in ["/Users/", "/home/", "UTC", "2024", "2025", "2026"] {
            assert!(!header.contains(needle), "{needle}");
        }
    }

    #[test]
    fn check_reports_first_difference() {
        let o = gen(FIXTURE);
        let dir = std::env::temp_dir().join("moult-xtask-gen-check");
        fs::create_dir_all(dir.join("src")).unwrap();
        write(&dir, &o).unwrap();
        assert!(check(&dir, &o).is_ok());
        fs::write(dir.join(NAMES_PATH), "other\n").unwrap();
        let err = check(&dir, &o).unwrap_err();
        assert!(err.contains("src/names.txt") && err.contains("line 1"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
