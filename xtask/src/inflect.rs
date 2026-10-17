//! `cargo xtask inflect`: words on stdin, one per line; TSV on stdout with
//! `word`, `singular`, `plural`, `is_noun`. Feeds the NLTK differential in
//! `tools/nltk-diff/`. With `--guess` the inflector enables `guess_unknown`.

use std::io::{BufRead, BufWriter, Write};

pub fn run(guess: bool) -> Result<(), String> {
    let inflector = moult::Inflector::builder()
        .guess_unknown(guess)
        .build()
        .map_err(|e| e.to_string())?;
    let stdin = std::io::stdin();
    let mut out = BufWriter::new(std::io::stdout().lock());
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        let w = line.trim_end_matches('\r');
        if w.is_empty() {
            continue;
        }
        writeln!(
            out,
            "{w}\t{}\t{}\t{}",
            inflector.singularize(w),
            inflector.pluralize(w),
            inflector.is_noun(w)
        )
        .map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}
