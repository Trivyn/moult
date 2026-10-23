//! The caller-supplied override layer, through the public API only.

use moult::{Inflector, OverrideError, PartOfSpeech};

fn build(pairs: &[(&str, &str)], invariant: &[&str]) -> Result<Inflector, OverrideError> {
    let mut b = Inflector::builder();
    for (s, p) in pairs {
        b = b.pair(s, p);
    }
    for w in invariant {
        b = b.invariant(w);
    }
    b.build()
}

#[test]
fn pairs_work_in_both_directions_and_keep_case() {
    let inf = build(&[("SKU", "SKUs"), ("ETA", "ETAs")], &[]).unwrap();
    assert_eq!(inf.singularize("SKUs"), "SKU");
    assert_eq!(inf.singularize("Skus"), "Sku");
    assert_eq!(inf.singularize("skus"), "sku");
    assert_eq!(inf.pluralize("SKU"), "SKUs");
    assert_eq!(inf.pluralize("sku"), "skus");
    assert_eq!(inf.singularize("ETAs"), "ETA");
    assert_eq!(inf.pluralize("ETA"), "ETAs");
    assert_eq!(inf.pluralize("Eta"), "Etas");
}

#[test]
fn invariant_words_are_unchanged_both_ways() {
    let inf = build(&[], &["personnel"]).unwrap();
    for w in ["personnel", "Personnel", "PERSONNEL"] {
        assert_eq!(inf.singularize(w), w);
        assert_eq!(inf.pluralize(w), w);
    }
}

#[test]
fn overrides_beat_the_built_in_data() {
    let inf = build(&[("datum", "data")], &["glasses"]).unwrap();
    assert_eq!(inf.singularize("data"), "datum");
    assert_eq!(inf.pluralize("datum"), "data");
    assert_eq!(inf.pluralize("glasses"), "glasses");
    assert_eq!(inf.singularize("glasses"), "glasses");
    // Without the overrides the built-ins answer.
    assert_eq!(moult::singularize("data"), "data");
    assert_eq!(moult::singularize("glasses"), "glass");
}

#[test]
fn override_singulars_count_as_known() {
    let inf = build(&[("sku", "skus"), ("blorp", "blorpii")], &["zzyzx"]).unwrap();
    assert_eq!(inf.singularize("sku"), "sku");
    assert_eq!(inf.singularize("blorp"), "blorp");
    assert_eq!(inf.pluralize("blorpii"), "blorpii");
    assert!(inf.is_noun("sku") && inf.is_noun("SKUs") && inf.is_noun("blorpii"));
    assert!(inf.is_noun("zzyzx"));
    assert!(!moult::is_noun("blorpii"));
    assert!(!moult::is_noun("zzyzx"));
}

#[test]
fn every_error_variant_has_a_minimal_input_and_a_message() {
    let cases: Vec<(Result<Inflector, OverrideError>, OverrideError)> = vec![
        (
            build(&[("", "x")], &[]),
            OverrideError::InvalidWord("".into()),
        ),
        (
            build(&[], &["a b"]),
            OverrideError::InvalidWord("a b".into()),
        ),
        (
            build(&[("A", "a")], &[]),
            OverrideError::IdenticalPair("a".into()),
        ),
        (
            build(&[("a", "b"), ("a", "c")], &[]),
            OverrideError::DuplicateSingular("a".into()),
        ),
        (
            build(&[("a", "c"), ("b", "c")], &[]),
            OverrideError::DuplicatePlural("c".into()),
        ),
        (
            build(&[("a", "b"), ("b", "c")], &[]),
            OverrideError::Conflict("b".into()),
        ),
        (
            build(&[("a", "b")], &["a"]),
            OverrideError::Conflict("a".into()),
        ),
        (
            Inflector::builder()
                .lemma("x", PartOfSpeech::Verb, "y")
                .lemma("X", PartOfSpeech::Verb, "z")
                .build(),
            OverrideError::DuplicateLemma("x".into()),
        ),
        (
            Inflector::builder()
                .third_person_singular("can", "cans")
                .third_person_singular("CAN", "can")
                .build(),
            OverrideError::DuplicateThirdPerson("can".into()),
        ),
    ];
    for (got, want) in cases {
        let err = got.expect_err("build should fail");
        assert_eq!(err, want);
        let msg = err.to_string();
        assert!(!msg.is_empty(), "{want:?}");
    }
    assert!(build(&[], &["x", "x"]).is_ok());
}

#[test]
fn inflector_is_send_and_sync() {
    const fn assert_send_sync<T: Send + Sync>() {}
    const _: () = assert_send_sync::<Inflector>();
    const _: () = assert_send_sync::<OverrideError>();
}

#[test]
fn default_matches_the_free_functions() {
    let sample = [
        "waves",
        "mice",
        "People",
        "ISBNs",
        "glasses",
        "analysis",
        "Territory",
        "BOX",
        "data",
        "blorps",
        "Kansas",
        "",
        "s",
        "Café",
        "mothers-in-law",
        "two words",
    ];
    for inf in [Inflector::new(), Inflector::default()] {
        for w in sample {
            assert_eq!(inf.singularize(w), moult::singularize(w), "{w:?}");
            assert_eq!(inf.pluralize(w), moult::pluralize(w), "{w:?}");
            assert_eq!(inf.is_noun(w), moult::is_noun(w), "{w:?}");
        }
    }
}

#[test]
fn all_caps_override_plurals_follow_the_documented_case_rule() {
    // A plural that only adds "s" or "es" is an appended suffix.
    let inf = build(&[("sku", "skus"), ("bus", "buses")], &[]).unwrap();
    assert_eq!(inf.pluralize("SKU"), "SKUs");
    assert_eq!(inf.pluralize("BUS"), "BUSes");
    // Any other supplied plural follows the word's case.
    let inf = build(&[("index", "indices"), ("cow", "kine")], &[]).unwrap();
    assert_eq!(inf.pluralize("INDEX"), "INDICES");
    assert_eq!(inf.pluralize("COW"), "KINE");
    assert_eq!(inf.pluralize("Cow"), "Kine");
}

#[test]
fn borrowed_if_and_only_if_unchanged() {
    use std::borrow::Cow;
    // Final sigma: "\u{3bf}\u{3c2}" is the plural of "\u{3bf}\u{3c3}", yet both restore to the
    // same upper-case input, so nothing changes and nothing is allocated.
    let inf = build(&[("\u{3bf}\u{3c3}", "\u{3bf}\u{3c2}")], &[]).unwrap();
    let word = "\u{39f}\u{3a3}";
    assert!(matches!(inf.singularize(word), Cow::Borrowed(w) if w == word));
    for word in [
        "\u{39f}\u{3a3}",
        "\u{3bf}\u{3c3}",
        "\u{3bf}\u{3c2}",
        "\u{39f}\u{3a3}s",
    ] {
        for got in [inf.singularize(word), inf.pluralize(word)] {
            assert_eq!(
                got.as_ref() == word,
                matches!(got, Cow::Borrowed(_)),
                "{word}"
            );
        }
    }
}

#[test]
fn third_person_overrides_keep_case_and_register_the_verb_lemma() {
    use std::borrow::Cow;
    let inf = Inflector::builder()
        .third_person_singular("can", "cans")
        .build()
        .unwrap();
    for (input, want) in [("can", "cans"), ("Can", "Cans"), ("cans", "cans")] {
        let got = inf.third_person_singular(input);
        assert_eq!(got, want, "{input}");
        assert_eq!(matches!(got, Cow::Borrowed(_)), input == want, "{input}");
    }
    assert_eq!(inf.lemmatize("cans", PartOfSpeech::Verb), "can");
}

#[test]
fn third_person_overrides_reject_empty_or_whitespace_words() {
    for word in ["", " ", "two words", "can\n", "\u{a0}"] {
        for (verb, form) in [(word, "cans"), ("can", word)] {
            let err = Inflector::builder()
                .third_person_singular(verb, form)
                .build()
                .expect_err("invalid word");
            assert_eq!(
                err,
                OverrideError::InvalidWord(word.into()),
                "{verb:?} -> {form:?}"
            );
        }
    }
}

#[test]
fn third_person_and_explicit_verb_lemmas_must_agree() {
    use PartOfSpeech::Verb;
    let err = Inflector::builder()
        .third_person_singular("can", "cans")
        .lemma("cans", Verb, "x")
        .build()
        .expect_err("conflicting lemma");
    assert_eq!(err, OverrideError::Conflict("cans".into()));
    let inf = Inflector::builder()
        .third_person_singular("can", "cans")
        .lemma("cans", Verb, "can")
        .build()
        .unwrap();
    assert_eq!(inf.third_person_singular("can"), "cans");
    assert_eq!(inf.lemmatize("cans", Verb), "can");
}

#[test]
fn third_person_override_forms_cannot_be_other_override_verbs() {
    for form in ["b", "B"] {
        for verb in ["b", "B"] {
            for pairs in [[("a", form), (verb, "c")], [(verb, "c"), ("a", form)]] {
                let mut builder = Inflector::builder();
                for (verb, form) in pairs {
                    builder = builder.third_person_singular(verb, form);
                }
                let err = builder.build().expect_err("conflicting form and verb");
                assert_eq!(err, OverrideError::Conflict("b".into()), "{pairs:?}");
            }
        }
    }
}

#[test]
fn third_person_overrides_cannot_share_a_form() {
    // Each form registers its verb as the verb lemma, so two verbs cannot
    // claim one form.
    for (first, second) in [("foo", "bar"), ("bar", "foo")] {
        let err = Inflector::builder()
            .third_person_singular(first, "zaps")
            .third_person_singular(second, "zaps")
            .build()
            .expect_err("shared form");
        assert_eq!(err, OverrideError::Conflict("zaps".into()));
    }
}

#[test]
fn third_person_override_can_keep_a_known_verb_unchanged() {
    use std::borrow::Cow;
    for (verb, form) in [("live", "live"), ("LIVE", "live"), ("live", "LIVE")] {
        let inf = Inflector::builder()
            .third_person_singular(verb, form)
            .build()
            .unwrap();
        for input in ["live", "Live", "LIVE"] {
            assert!(matches!(
                inf.third_person_singular(input),
                Cow::Borrowed(word) if word == input
            ));
        }
    }
}

#[test]
fn third_person_override_needs_no_guess_for_an_unknown_verb() {
    let inf = Inflector::builder()
        .third_person_singular("upsert", "upserts")
        .build()
        .unwrap();
    assert_eq!(inf.third_person_singular("upsert"), "upserts");
}
