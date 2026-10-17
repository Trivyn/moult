//! Case handling: the lower-case lookup key, and putting an input's case
//! pattern back on a lower-case result.

use alloc::string::String;

/// The longest ASCII word lower-cased without allocating.
const STACK_LEN: usize = 64;

/// The lower-cased lookup key of a word. An ASCII word of at most
/// [`STACK_LEN`] bytes lives on the stack; anything else allocates.
pub(crate) enum Lower {
    Stack { buf: [u8; STACK_LEN], len: usize },
    Heap(String),
}

impl Lower {
    pub(crate) fn new(word: &str) -> Lower {
        if word.is_ascii() && word.len() <= STACK_LEN {
            let mut buf = [0; STACK_LEN];
            for (dst, src) in buf.iter_mut().zip(word.bytes()) {
                *dst = src.to_ascii_lowercase();
            }
            Lower::Stack {
                buf,
                len: word.len(),
            }
        } else {
            Lower::Heap(word.to_lowercase())
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            // The bytes are ASCII, so this never fails.
            Lower::Stack { buf, len } => core::str::from_utf8(&buf[..*len]).unwrap_or_default(),
            Lower::Heap(s) => s,
        }
    }
}

/// Where a result came from, which decides the case of letters added to an
/// unchanged all-caps word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Origin {
    /// A suffix rule appended letters to the word ("ISBN" to "ISBNs"): they
    /// stay lower case, as acronym plurals are written.
    Rule,
    /// A listed form: an irregular, choice, dictionary or override result
    /// ("CHILD" to "CHILDREN"). It follows the word's case.
    Form,
}

#[derive(PartialEq)]
enum Style {
    /// At least one upper-case letter and no lower-case letter.
    Caps,
    /// An upper-case first letter and no other upper-case letter.
    Title,
    /// Lower case, or a mixed pattern such as "iPhone".
    Other,
}

fn style(input: &str) -> Style {
    let (mut upper, mut lower) = (0, 0);
    for c in input.chars() {
        if c.is_uppercase() {
            upper += 1;
        } else if c.is_lowercase() {
            lower += 1;
        }
    }
    if upper > 0 && lower == 0 {
        Style::Caps
    } else if upper == 1 && input.chars().next().is_some_and(char::is_uppercase) {
        Style::Title
    } else {
        Style::Other
    }
}

/// `result`, which is lower case, with the case pattern of `input`, whose
/// lower-cased form is `key`.
///
/// The characters of `input` are copied over the prefix `key` and `result`
/// share, so the part of the word that did not change keeps its exact case.
/// For the rest of `result`:
///
/// - Title Case input gives Title Case output (`Mice` to `Mouse`, `Person`
///   to `People`), even when the shared prefix is empty.
/// - All-caps input gives upper case where `result` rewrites the input
///   (`MOUSE` to `MICE`, `TERRITORY` to `TERRITORIES`) and for a listed
///   form that extends it (`CHILD` to `CHILDREN`, [`Origin::Form`]), but lower
///   case for a rule suffix appended to the whole unchanged input (`ISBN` to
///   `ISBNs`, `BOX` to `BOXes`, [`Origin::Rule`]), following the usual
///   convention for acronym plurals.
/// - Any other input gives lower case.
///
/// The characters the result keeps at its end are copied from the input too.
///
/// Lower-casing can turn one character into several ("İ" into "i" and a
/// combining dot), so `key` may be longer than `input`. The copying then goes
/// by whole input characters: an input character is copied only when all the
/// key characters it lower-cases to lie in the shared prefix (or the kept
/// tail); the rest of an expansion is handled as part of the rewritten middle.
pub(crate) fn restore(input: &str, key: &str, result: &str, origin: Origin) -> String {
    // How many characters `c` lower-cases to inside the key.
    let width = |c: char| c.to_lowercase().count();
    let shared = key
        .chars()
        .zip(result.chars())
        .take_while(|(a, b)| a == b)
        .count();
    // The leading input characters whose lower-case form lies in the prefix.
    let (mut head, mut head_key) = (0, 0);
    for c in input.chars() {
        if head_key + width(c) > shared {
            break;
        }
        head_key += width(c);
        head += 1;
    }
    let offset = result
        .char_indices()
        .nth(head_key)
        .map_or(result.len(), |(i, _)| i);
    // The characters the result keeps at its end ("Mothers-in-Law" to
    // "mother-in-law" keeps "-in-Law"); they cannot overlap the prefix.
    let result_len = result.chars().count();
    let (key_rest, result_rest) = (key.chars().count() - head_key, result_len - head_key);
    let kept_tail = key
        .chars()
        .rev()
        .zip(result.chars().rev())
        .take(key_rest.min(result_rest))
        .take_while(|(a, b)| a == b)
        .count();
    let (mut tail, mut tail_key) = (0, 0);
    for c in input.chars().rev() {
        if tail_key + width(c) > kept_tail {
            break;
        }
        tail_key += width(c);
        tail += 1;
    }
    let total = input.chars().count();
    let tail_start = result
        .char_indices()
        .nth(result_len - tail_key)
        .map_or(result.len(), |(i, _)| i);
    let rest = &result[offset..tail_start];
    let mut out = String::with_capacity(result.len());
    out.extend(input.chars().take(head));
    match style(input) {
        Style::Caps if head < total || origin == Origin::Form => {
            out.extend(rest.chars().flat_map(char::to_uppercase))
        }
        Style::Title if head == 0 => {
            let mut chars = rest.chars();
            out.extend(chars.next().into_iter().flat_map(char::to_uppercase));
            out.push_str(chars.as_str());
        }
        _ => out.push_str(rest),
    }
    out.extend(input.chars().skip(total - tail));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(input: &str, result: &str) -> String {
        restore(input, Lower::new(input).as_str(), result, Origin::Rule)
    }

    fn f(input: &str, result: &str) -> String {
        restore(input, Lower::new(input).as_str(), result, Origin::Form)
    }

    #[test]
    fn keeps_the_shared_prefix_exactly() {
        assert_eq!(r("iPhones", "iphone"), "iPhone");
        assert_eq!(r("McDonald", "mcdonalds"), "McDonalds");
        assert_eq!(r("ISBNs", "isbn"), "ISBN");
        assert_eq!(r("Territories", "territory"), "Territory");
        assert_eq!(r("Mothers-in-Law", "mother-in-law"), "Mother-in-Law");
        assert_eq!(r("MOTHERS-IN-LAW", "mother-in-law"), "MOTHER-IN-LAW");
        assert_eq!(r("Mother-in-Law", "mothers-in-law"), "Mothers-in-Law");
    }

    #[test]
    fn title_case() {
        assert_eq!(r("Mice", "mouse"), "Mouse");
        assert_eq!(r("Person", "people"), "People");
        assert_eq!(r("Men", "man"), "Man");
        assert_eq!(r("Cat", "cats"), "Cats");
        // No shared prefix at all.
        assert_eq!(r("Is", "are"), "Are");
    }

    #[test]
    fn all_caps() {
        assert_eq!(r("MOUSE", "mice"), "MICE");
        assert_eq!(r("TERRITORY", "territories"), "TERRITORIES");
        assert_eq!(r("PERSON", "people"), "PEOPLE");
        assert_eq!(r("ISBN", "isbns"), "ISBNs");
        assert_eq!(r("ID", "ids"), "IDs");
        assert_eq!(r("BOX", "boxes"), "BOXes");
        assert_eq!(r("BOXES", "box"), "BOX");
        assert_eq!(r("SKU2", "sku2s"), "SKU2s");
    }

    #[test]
    fn all_caps_listed_forms_follow_the_word() {
        assert_eq!(f("CHILD", "children"), "CHILDREN");
        assert_eq!(f("OX", "oxen"), "OXEN");
        assert_eq!(f("Child", "children"), "Children");
        assert_eq!(f("child", "children"), "children");
        assert_eq!(f("MOUSE", "mice"), "MICE");
    }

    #[test]
    fn other_patterns_lower_the_rest() {
        assert_eq!(r("cat", "cats"), "cats");
        assert_eq!(r("iPhone", "iphones"), "iPhones");
        assert_eq!(r("eBay", "ebays"), "eBays");
    }

    #[test]
    fn expanding_lower_case_keeps_the_input() {
        // "İ" lower-cases to two characters, so the key is longer than the input.
        assert_eq!(r("İstanbul", "i\u{307}stanbuls"), "İstanbuls");
        assert_eq!(r("İstanbuls", "i\u{307}stanbul"), "İstanbul");
        assert_eq!(r("İSTANBUL", "i\u{307}stanbuls"), "İSTANBULs");
        // The expansion itself is rewritten: nothing of it is copied.
        assert_eq!(r("İx", "j"), "J");
        // Final sigma lower-cases to "ς" in the key, "σ" per character.
        assert_eq!(r("ΟΔΟΣ", "οδος"), "ΟΔΟΣ");
        assert_eq!(r("ΟΔΟΣ", "οδοι"), "ΟΔΟΙ");
        assert_eq!(r("ǅx", "ǆxs"), "ǅxs");
        assert_eq!(r("STRAẞE", "straßes"), "STRAẞEs");
    }

    #[test]
    fn lower_key() {
        assert_eq!(Lower::new("AbC").as_str(), "abc");
        assert_eq!(Lower::new("").as_str(), "");
        let long = "Ab".repeat(40);
        assert_eq!(Lower::new(&long).as_str(), "ab".repeat(40));
        assert_eq!(Lower::new("CAFÉ").as_str(), "café");
    }
}
