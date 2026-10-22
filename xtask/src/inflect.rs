//! `cargo xtask inflect`: words on stdin, one per line; TSV on stdout. Feeds the
//! NLTK differential in `tools/nltk-diff/`.
//!
//! - default: `word`, `singular`, `plural`, `is_noun`; with `--guess` the
//!   inflector enables `guess_unknown`.
//! - `--lemma`: `word`, `verb`, `adjective`, the `lemmatize` results.
//! - `--lemma-candidates`: `word`, then the comma-separated candidate lemmas of
//!   the word as a verb and as an adjective, in order.

use moult::PartOfSpeech::{Adjective, Verb};
use std::io::{BufRead, BufWriter, Write};

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Noun { guess: bool },
    Lemma,
    LemmaCandidates,
}

pub fn run(mode: Mode) -> Result<(), String> {
    let guess = matches!(mode, Mode::Noun { guess: true });
    let inflector = moult::Inflector::builder()
        .guess_unknown(guess)
        .build()
        .map_err(|e| e.to_string())?;
    let stdin = std::io::stdin();
    let mut out = BufWriter::new(std::io::stdout().lock());
    let candidates = |w: &str, pos| {
        inflector
            .lemmas_for(w, pos)
            .map(|l| l.text)
            .collect::<Vec<_>>()
            .join(",")
    };
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        let w = line.trim_end_matches('\r');
        if w.is_empty() {
            continue;
        }
        let res = match mode {
            Mode::Noun { .. } => writeln!(
                out,
                "{w}\t{}\t{}\t{}",
                inflector.singularize(w),
                inflector.pluralize(w),
                inflector.is_noun(w)
            ),
            Mode::Lemma => writeln!(
                out,
                "{w}\t{}\t{}",
                inflector.lemmatize(w, Verb),
                inflector.lemmatize(w, Adjective)
            ),
            Mode::LemmaCandidates => writeln!(
                out,
                "{w}\t{}\t{}",
                candidates(w, Verb),
                candidates(w, Adjective)
            ),
        };
        res.map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}
