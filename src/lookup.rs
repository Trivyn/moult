//! Allocation-free lookups in the generated tables.

use crate::data;

/// Whether `word` is one of the lines of `table` (see [`find`]).
pub(crate) fn contains(table: &str, word: &str) -> bool {
    find(table, word).is_some()
}

/// The line of `table` equal to `word`, as a slice of the table: lower-case
/// words, one per line, each `\n`-terminated, sorted by byte order.
///
/// A binary search over byte offsets. The search range always starts and ends
/// on a line boundary; each probe walks back from the middle to the start of
/// its line, reads the line, and narrows the range.
pub(crate) fn find<'t>(table: &'t str, word: &str) -> Option<&'t str> {
    let t = table.as_bytes();
    let w = word.as_bytes();
    let (mut lo, mut hi) = (0, t.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let start = t[lo..mid]
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(lo, |i| lo + i + 1);
        let end = t[start..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(t.len(), |i| start + i);
        match t[start..end].cmp(w) {
            core::cmp::Ordering::Equal => return Some(&table[start..end]),
            core::cmp::Ordering::Less => lo = end + 1,
            core::cmp::Ordering::Greater => hi = start,
        }
    }
    None
}

/// The rows of `table`, sorted by first element, whose first element is `key`.
pub(crate) fn rows<'t>(
    table: &'t [(&'static str, &'static str)],
    key: &str,
) -> &'t [(&'static str, &'static str)] {
    let start = table.partition_point(|row| row.0 < key);
    let end = start + table[start..].partition_point(|row| row.0 == key);
    &table[start..end]
}

/// The `(form, lemma)` rows of `FORM_TO_LEMMA` whose lemma is `key`, in form
/// order, found through the `LEMMA_TO_PLURAL` index.
pub(crate) fn rows_by_lemma(
    key: &str,
) -> impl Iterator<Item = (&'static str, &'static str)> + 'static {
    let index = data::LEMMA_TO_PLURAL;
    let lemma = |i: &u16| data::FORM_TO_LEMMA[usize::from(*i)].1;
    let start = index.partition_point(|i| lemma(i) < key);
    let end = start + index[start..].partition_point(|i| lemma(i) == key);
    index[start..end]
        .iter()
        .map(|&i| data::FORM_TO_LEMMA[usize::from(i)])
}

#[cfg(test)]
mod tests {
    use super::contains;

    #[test]
    fn finds_every_word_and_nothing_else() {
        let table = "ant\nbee\ncat\ndog\neel\nfox\n";
        for w in ["ant", "bee", "cat", "dog", "eel", "fox"] {
            assert!(contains(table, w), "{w}");
        }
        // Before the first, between, after the last, prefixes and extensions.
        for w in [
            "", "a", "an", "ants", "bea", "bees", "cow", "dogs", "fo", "foxes", "zebra",
        ] {
            assert!(!contains(table, w), "{w}");
        }
    }

    #[test]
    fn small_tables() {
        assert!(!contains("", "a"));
        assert!(!contains("", ""));
        assert!(contains("only\n", "only"));
        assert!(!contains("only\n", "on"));
        assert!(contains("a\nb\n", "a") && contains("a\nb\n", "b"));
        assert!(!contains("a\nb\n", "c"));
    }

    #[test]
    fn every_word_of_a_larger_table() {
        let words: alloc::vec::Vec<alloc::string::String> =
            (0..500).map(|i| alloc::format!("w{i:04}x")).collect();
        let table: alloc::string::String = words.iter().map(|w| alloc::format!("{w}\n")).collect();
        for w in &words {
            assert!(contains(&table, w));
            assert!(!contains(&table, &alloc::format!("{w}y")));
        }
    }
}
