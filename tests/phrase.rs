//! `singularize_phrase` and `pluralize_phrase`: the last word is inflected,
//! the rest of the text is kept byte for byte.

use std::borrow::Cow;

use moult::{pluralize, pluralize_phrase, singularize, singularize_phrase, Inflector};

#[test]
fn inflects_the_last_word() {
    assert_eq!(singularize_phrase("order lines"), "order line");
    assert_eq!(singularize_phrase("Order Lines"), "Order Line");
    assert_eq!(pluralize_phrase("dry dock"), "dry docks");
    assert_eq!(pluralize_phrase("Dry Dock"), "Dry Docks");
}

#[test]
fn keeps_surrounding_and_internal_whitespace() {
    assert_eq!(singularize_phrase("  order lines\n"), "  order line\n");
    assert_eq!(
        pluralize_phrase("\tdry \t dock \r\n"),
        "\tdry \t docks \r\n"
    );
    assert_eq!(singularize_phrase("order   lines"), "order   line");
    assert_eq!(singularize_phrase("order\u{a0}lines"), "order\u{a0}line");
    assert_eq!(
        pluralize_phrase("dry\u{a0}dock\u{2003}"),
        "dry\u{a0}docks\u{2003}"
    );
    assert_eq!(singularize_phrase("order\nlines"), "order\nline");
}

#[test]
fn a_single_word_equals_the_word_function() {
    for word in [
        "lines",
        "mice",
        "gas",
        "IDs",
        "s",
        "41",
        "blorps",
        "Kubernetes",
    ] {
        assert_eq!(singularize_phrase(word), singularize(word), "{word}");
        assert_eq!(pluralize_phrase(word), pluralize(word), "{word}");
    }
}

#[test]
fn empty_and_whitespace_only_are_unchanged() {
    for input in ["", " ", "  \t\n", "\u{a0}"] {
        assert!(
            matches!(singularize_phrase(input), Cow::Borrowed(_)),
            "{input:?}"
        );
        assert!(
            matches!(pluralize_phrase(input), Cow::Borrowed(_)),
            "{input:?}"
        );
        assert_eq!(singularize_phrase(input), input);
        assert_eq!(pluralize_phrase(input), input);
    }
}

#[test]
fn borrowed_when_unchanged_owned_when_changed() {
    assert!(matches!(singularize_phrase("order line"), Cow::Borrowed(_)));
    assert!(matches!(pluralize_phrase("dry docks"), Cow::Borrowed(_)));
    assert!(matches!(singularize_phrase("order lines"), Cow::Owned(_)));
    assert!(matches!(pluralize_phrase("dry dock"), Cow::Owned(_)));
}

#[test]
fn the_word_functions_still_ignore_whitespace_input() {
    assert_eq!(singularize("order lines"), "order lines");
    assert_eq!(pluralize("dry dock"), "dry dock");
    assert!(!moult::is_noun("order lines"));
}

#[test]
fn an_override_applies_to_the_last_word() {
    let inflector = Inflector::builder()
        .pair("sku", "skus")
        .invariant("personnel")
        .build()
        .unwrap();
    assert_eq!(inflector.pluralize_phrase("blue SKU"), "blue SKUs");
    assert_eq!(inflector.singularize_phrase("blue skus"), "blue sku");
    assert_eq!(inflector.pluralize_phrase("new personnel"), "new personnel");
    assert!(matches!(
        inflector.pluralize_phrase("new personnel"),
        Cow::Borrowed(_)
    ));
}

#[test]
fn guess_unknown_applies_to_the_last_word() {
    let guessing = Inflector::builder().guess_unknown(true).build().unwrap();
    // Guard: the guess really reduces the word on its own.
    assert_eq!(guessing.singularize("microservices"), "microservice");
    assert_eq!(
        guessing.singularize_phrase("transit microservices"),
        "transit microservice"
    );
    assert_eq!(
        singularize_phrase("transit microservices"),
        "transit microservices"
    );
}

#[test]
fn case_is_kept() {
    assert_eq!(singularize_phrase("ORDER LINES"), "ORDER LINE");
    assert_eq!(singularize_phrase("open IDs"), "open ID");
    assert_eq!(pluralize_phrase("Dry DOCK"), "Dry DOCKs");
}

#[test]
fn head_initial_compounds_inflect_the_wrong_word() {
    // Documented limitation: only the last word changes, so the head of a
    // head-initial compound is not inflected. The caller must handle those.
    assert_eq!(pluralize_phrase("attorney general"), "attorney generals");
    assert_ne!(pluralize_phrase("attorney general"), "attorneys general");
}

#[test]
fn a_last_word_ending_in_punctuation_is_unchanged() {
    for phrase in ["dry dock,", "dry dock.", "C++"] {
        assert!(matches!(pluralize_phrase(phrase), Cow::Borrowed(p) if p == phrase));
    }
    assert!(matches!(
        singularize_phrase("dry docks,"),
        Cow::Borrowed("dry docks,")
    ));
}
