//! The opt-in fallback: `Inflector::builder().guess_unknown(true)`. The
//! default mode, which never guesses, is tested in tests/cases.rs.

use moult::Inflector;

fn guessing() -> Inflector {
    Inflector::builder().guess_unknown(true).build().unwrap()
}

#[test]
fn guesses_regular_plurals_of_unknown_words() {
    let g = guessing();
    for (input, want) in [
        ("microservices", "microservice"),
        ("namespaces", "namespace"),
        ("webhooks", "webhook"),
        // Resolved by dictionary-checked rules, with or without the guess.
        ("endpoints", "endpoint"),
        ("workflows", "workflow"),
        ("widgets", "widget"),
    ] {
        assert_eq!(g.singularize(input), want, "{input}");
    }
    // Case is kept.
    assert_eq!(guessing().singularize("Microservices"), "Microservices");
}

#[test]
fn still_refuses_what_the_default_refuses() {
    let g = guessing();
    for w in [
        "backwards",
        "afterwards",
        "themselves",
        "superspecies",
        "subseries",
        "facies",
        "biomechanics",
        "tholos",
        "xyrichthys",
        "agaricales",
        "moses",
        "ulysses",
        "kubernetes",
        "Kubernetes",
        "laches",
        "abies",
        "colluvies",
        "hirsuties",
        "natchitoches",
        "lichenes",
        "albanenses",
        "elaps",
        "custodes",
        "perhaps",
        "sometimes",
        "alas",
    ] {
        assert_eq!(g.singularize(w), w, "{w}");
    }
}

/// Known residual of the opt-in fallback, not a goal: a compound whose last
/// part is a noun passes the known-head check although the compound is not a
/// plural. The default mode leaves both words unchanged (tests/cases.rs).
#[test]
fn known_residual_non_word_compounds() {
    let g = guessing();
    assert_eq!(g.singularize("afterhours"), "afterhour");
    assert_eq!(g.singularize("belowstairs"), "belowstair");
}

#[test]
fn acronym_plurals_need_no_guess() {
    for inflector in [Inflector::new(), guessing()] {
        assert_eq!(inflector.singularize("IDs"), "ID");
        assert_eq!(inflector.singularize("OSes"), "OS");
    }
}

#[test]
fn default_builder_does_not_guess() {
    let plain = Inflector::builder().build().unwrap();
    assert_eq!(plain.singularize("microservices"), "microservices");
    assert_eq!(
        Inflector::default().singularize("microservices"),
        "microservices"
    );
    let off = Inflector::builder()
        .guess_unknown(true)
        .guess_unknown(false)
        .build()
        .unwrap();
    assert_eq!(off.singularize("microservices"), "microservices");
}

#[test]
fn is_noun_ignores_the_flag() {
    let g = guessing();
    for w in [
        "microservices",
        "waves",
        "gas",
        "mice",
        "blorps",
        "perhaps",
        "IDs",
    ] {
        assert_eq!(g.is_noun(w), moult::is_noun(w), "{w}");
    }
}

/// Pluralize must not depend on the flag, including for unknown words.
#[test]
fn pluralize_is_identical_in_both_modes() {
    let g = guessing();
    let plain = Inflector::new();
    let mut words: Vec<String> = Vec::new();
    // Every PLURAL input of the acceptance table.
    let cases = include_str!("cases.rs");
    let plural = cases.split("const PLURAL").nth(1).unwrap();
    let plural = plural.split("fn check").next().unwrap();
    for line in plural.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("(\"") {
            words.push(rest.split('"').next().unwrap().to_string());
        }
    }
    assert!(words.len() > 50);
    // Unknown words, plural-looking and not, and compounds.
    for w in [
        "microservices",
        "microservice",
        "namespaces",
        "webhooks",
        "afterhours",
        "belowstairs",
        "blorps",
        "blorp",
        "zorbit",
        "zorbits",
        "kubernetes",
        "moses",
        "ulysses",
        "laches",
        "elaps",
        "facies",
        "tholos",
        "backwards",
        "Foobars",
        "FOOBARS",
        "IDs",
        "SKUs",
        "quuxes",
        "frobnicate",
        "frobnicates",
        "xyz",
        "wibbleies",
        "wibbley",
    ] {
        words.push(w.to_string());
    }
    // A few hundred dictionary words, spread across the noun list.
    let nouns = include_str!("../src/nouns.txt");
    words.extend(nouns.lines().step_by(150).map(str::to_string));
    // Unknown words made from them.
    let extra: Vec<String> = words
        .iter()
        .flat_map(|w| [format!("{w}zq"), format!("{w}zqs"), format!("un{w}s")])
        .collect();
    words.extend(extra);
    assert!(words.len() > 300);
    for w in &words {
        assert_eq!(g.pluralize(w), plain.pluralize(w), "pluralize({w})");
    }
}
