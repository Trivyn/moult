//! Case handling and odd input, through the public API only.

use std::borrow::Cow;

fn sing(w: &str) -> String {
    moult::singularize(w).into_owned()
}
fn plur(w: &str) -> String {
    moult::pluralize(w).into_owned()
}

#[test]
fn case_patterns() {
    assert_eq!(sing("mice"), "mouse");
    assert_eq!(sing("Mice"), "Mouse");
    assert_eq!(sing("MICE"), "MOUSE");
    assert_eq!(plur("mouse"), "mice");
    assert_eq!(plur("Mouse"), "Mice");
    assert_eq!(plur("MOUSE"), "MICE");
    assert_eq!(plur("Box"), "Boxes");
    assert_eq!(plur("BOX"), "BOXes");
    assert_eq!(sing("BOXES"), "BOX");
}

#[test]
fn acronym_plurals() {
    for (plural, single) in [
        ("IDs", "ID"),
        ("ISBNs", "ISBN"),
        ("URLs", "URL"),
        ("OSes", "OS"),
        ("SKUs", "SKU"),
    ] {
        assert_eq!(sing(plural), single);
        assert_eq!(plur(single), plural);
        assert_eq!(plur(plural), plural);
    }
}

#[test]
fn mixed_case_keeps_its_pattern() {
    assert_eq!(plur("iPhone"), "iPhones");
    // The unknown-word fallback reads lower-case ASCII only.
    assert_eq!(sing("iPhones"), "iPhones");
    assert_eq!(plur("eBay"), "eBays");
}

#[test]
fn non_ascii_input_keeps_its_case_and_does_not_panic() {
    assert_eq!(plur("Café"), "Cafés");
    // Not a WordNet plural, and the fallback reads ASCII only.
    assert_eq!(sing("Cafés"), "Cafés");
    assert_eq!(plur("café"), "cafés");
    for w in [
        "CAFÉ",
        "Straße",
        "İstanbul",
        "ÀÉÎÕÜ",
        "日本語",
        "😀s",
        "naïveté",
    ] {
        let _ = (
            moult::singularize(w),
            moult::pluralize(w),
            moult::is_noun(w),
        );
    }
}

#[test]
fn very_long_words_take_the_allocating_path() {
    for n in [64, 65, 200, 5000] {
        let w = "a".repeat(n);
        assert!(!moult::is_noun(&w));
        let _ = (moult::singularize(&w), moult::pluralize(&w));
        let wx = format!("{}s", "x".repeat(n));
        let _ = (moult::singularize(&wx), moult::pluralize(&wx));
        let wu = "É".repeat(n);
        let _ = (moult::singularize(&wu), moult::pluralize(&wu));
    }
    // A long word whose tail is a plural still inflects without panicking.
    let w = format!("{}box", "x".repeat(100));
    assert_eq!(plur(&w), format!("{w}es"));
}

#[test]
fn empty_whitespace_and_single_characters_are_unchanged() {
    for w in [
        "",
        " ",
        "two words",
        "a b",
        "\t",
        "cat\n",
        " cats",
        "é",
        "s",
        "a",
        "I",
        "X",
    ] {
        assert!(matches!(moult::singularize(w), Cow::Borrowed(_)), "{w:?}");
        assert!(matches!(moult::pluralize(w), Cow::Borrowed(_)), "{w:?}");
    }
}

#[test]
fn digits() {
    assert_eq!(plur("3D"), "3Ds");
    assert_eq!(sing("3Ds"), "3D");
    assert_eq!(sing("MP3s"), "MP3");
    assert_eq!(plur("MP3"), "MP3s");
    assert_eq!(sing("123"), "123");
    assert_eq!(plur("123"), "123");
}

#[test]
fn hyphenated_words_are_one_word() {
    // WordNet lists "mother-in-law" with the form "mothers-in-law".
    assert_eq!(sing("mothers-in-law"), "mother-in-law");
    assert_eq!(sing("Mothers-in-Law"), "Mother-in-Law");
    assert_eq!(plur("mother-in-law"), "mothers-in-law");
}

struct Xorshift(u64);
impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn random_strings_never_panic() {
    let guessing = moult::Inflector::builder()
        .guess_unknown(true)
        .build()
        .unwrap();
    let plain = moult::Inflector::new();
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-' \u{e9}\u{df}\u{130}\u{1F600}"
        .chars()
        .collect();
    // Suffixes that exercise the rules.
    let tails = [
        "s", "es", "ies", "ses", "men", "is", "ves", "oes", "ss", "ys",
    ];
    let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
    for i in 0..20_000 {
        let len = (rng.next() % 12) as usize;
        let mut w: String = (0..len)
            .map(|_| alphabet[(rng.next() % alphabet.len() as u64) as usize])
            .collect();
        if i % 3 == 0 {
            w.push_str(tails[(rng.next() % tails.len() as u64) as usize]);
        }
        for inflector in [&plain, &guessing] {
            let (s, p) = (inflector.singularize(&w), inflector.pluralize(&w));
            let _ = inflector.is_noun(&w);
            // Valid UTF-8 is guaranteed by the type; the output must also be sane.
            assert!(std::str::from_utf8(s.as_bytes()).is_ok());
            assert!(std::str::from_utf8(p.as_bytes()).is_ok());
            if !w.chars().any(char::is_alphabetic) {
                assert_eq!(s, w, "{w:?}");
                assert_eq!(p, w, "{w:?}");
            }
        }
        assert_eq!(plain.pluralize(&w), guessing.pluralize(&w), "{w:?}");
        assert_eq!(plain.is_noun(&w), guessing.is_noun(&w), "{w:?}");
    }
}

#[test]
fn cow_is_borrowed_exactly_when_unchanged() {
    for (plural, single) in [("IDs", "ID"), ("SKUs", "SKU"), ("OSes", "OS")] {
        let got = moult::singularize(plural);
        assert_eq!(got, single);
        assert!(matches!(got, Cow::Owned(_)), "{plural} changed, so Owned");
    }
    for w in ["ID", "gas", "laches", "İstanbul", "data"] {
        for got in [moult::singularize(w), moult::pluralize(w)] {
            if got == w {
                assert!(
                    matches!(got, Cow::Borrowed(_)),
                    "{w} unchanged, so Borrowed"
                );
            } else {
                assert!(matches!(got, Cow::Owned(_)), "{w} changed, so Owned");
            }
        }
    }
}

#[test]
fn length_changing_lower_casing_keeps_case() {
    // "İ" lower-cases to two characters.
    let same = moult::singularize("İstanbul");
    assert!(matches!(same, Cow::Borrowed("İstanbul")));
    assert_eq!(plur("İstanbul"), "İstanbuls");
    // A titlecase digraph, sharp s (small and capital) and a final sigma.
    for w in [
        "ǅ", "ǅudo", "Straße", "STRAẞE", "straße", "ΟΔΟΣ", "Οδος", "οδος",
    ] {
        let (s, p) = (moult::singularize(w), moult::pluralize(w));
        assert!(s.starts_with(&w[..w.char_indices().nth(1).map_or(w.len(), |(i, _)| i)]));
        assert!(p.len() >= w.len() || p == w, "{w}");
    }
    assert_eq!(plur("ΟΔΟΣ"), "ΟΔΟΣs");
    assert_eq!(plur("STRAẞE"), "STRAẞEs");
}
