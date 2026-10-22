//! Lemmatization through the public API only: candidate lists and their order,
//! the overrides, and robustness.

use moult::PartOfSpeech::{self, Adjective, Adverb, Noun, Verb};
use moult::{Inflector, OverrideError};
use std::borrow::Cow;

const ALL: [PartOfSpeech; 4] = [Noun, Verb, Adjective, Adverb];

fn list(word: &str) -> Vec<(String, PartOfSpeech)> {
    moult::lemmas(word)
        .map(|l| (l.text.to_string(), l.pos))
        .collect()
}

fn expect(word: &str, want: &[(&str, PartOfSpeech)]) {
    let want: Vec<(String, PartOfSpeech)> = want.iter().map(|&(t, p)| (t.to_string(), p)).collect();
    assert_eq!(list(word), want, "{word}");
}

#[test]
fn candidate_lists_and_their_order() {
    expect("saw", &[("saw", Noun), ("see", Verb), ("saw", Verb)]);
    expect(
        "better",
        &[
            ("better", Noun),
            ("better", Verb),
            ("good", Adjective),
            ("well", Adjective),
            ("better", Adjective),
            ("well", Adverb),
            ("better", Adverb),
        ],
    );
    expect(
        "found",
        &[
            ("found", Noun),
            ("find", Verb),
            ("found", Verb),
            ("found", Adjective),
        ],
    );
    expect(
        "wages",
        &[
            ("wage", Noun),
            ("wages", Noun),
            ("wage", Verb),
            ("wag", Verb),
        ],
    );
    // The first reading that is a rule result comes in rule order, not shortest first.
    expect("hating", &[("hate", Verb), ("hat", Verb)]);
    // The curated choice comes before the listed form.
    expect(
        "lay",
        &[
            ("lay", Noun),
            ("lay", Verb),
            ("lie", Verb),
            ("lay", Adjective),
        ],
    );
}

#[test]
fn lemmas_for_filters_one_part_of_speech() {
    let got: Vec<_> = moult::lemmas_for("saw", Verb).map(|l| l.text).collect();
    assert_eq!(got, ["see", "saw"]);
    assert!(moult::lemmas_for("saw", Adjective).is_empty());
    assert_eq!(
        moult::lemmas_for("better", Adverb).first().map(|l| l.text),
        Some("well")
    );
}

#[test]
fn unknown_empty_and_multi_word_input_has_no_candidates() {
    for w in [
        "qwzzx", "qwzzxed", "", "make it", " make", "make ", "a\tb", "\n",
    ] {
        assert!(moult::lemmas(w).is_empty(), "{w:?}");
        for pos in ALL {
            assert!(moult::lemmas_for(w, pos).is_empty(), "{w:?} {pos:?}");
            assert_eq!(moult::lemmatize(w, pos), w);
            assert!(matches!(moult::lemmatize(w, pos), Cow::Borrowed(_)));
        }
    }
}

#[test]
fn a_lemma_lists_itself_whatever_its_length() {
    expect("a", &[("a", Noun)]);
    assert_eq!(moult::lemmatize("a", Noun), "a");
}

#[test]
fn capitalised_only_lemmas_are_known_but_no_rule_target() {
    expect("Americanize", &[("americanize", Verb)]);
    expect("African", &[("african", Noun), ("african", Adjective)]);
    assert_eq!(moult::lemmatize("Americanize", Verb), "Americanize");
    // The rule would give "americanize", which a rule never reaches.
    assert!(moult::lemmas("americanized").is_empty());
    assert_eq!(moult::lemmatize("americanized", Verb), "americanized");
}

#[test]
fn the_noun_reading_is_singularize() {
    for w in ["leaves", "mice", "Wolves", "glasses", "data", "zzz", "Axes"] {
        assert_eq!(moult::lemmatize(w, Noun), moult::singularize(w), "{w}");
    }
}

#[test]
fn borrowed_exactly_when_unchanged() {
    for w in [
        "made", "make", "Made", "MADE", "running", "modest", "qwzzx", "better",
    ] {
        for pos in ALL {
            let got = moult::lemmatize(w, pos);
            assert_eq!(matches!(got, Cow::Borrowed(_)), got == w, "{w} {pos:?}");
        }
    }
}

#[test]
fn len_stays_consistent_while_iterating() {
    let mut it = moult::lemmas("better");
    let total = it.len();
    assert_eq!(total, 7);
    assert_eq!(it.size_hint(), (total, Some(total)));
    let mut seen = 0;
    while let Some(first) = it.first() {
        assert_eq!(it.len(), total - seen);
        assert_eq!(it.next(), Some(first));
        seen += 1;
        assert_eq!(it.size_hint(), (total - seen, Some(total - seen)));
    }
    assert_eq!(seen, total);
    assert!(it.is_empty());
    assert_eq!(it.next(), None);
}

fn with(form: &str, pos: PartOfSpeech, lemma: &str) -> Inflector {
    Inflector::builder()
        .lemma(form, pos, lemma)
        .build()
        .unwrap()
}

#[test]
fn an_override_per_part_of_speech() {
    let v = with("zorped", Verb, "zorp");
    assert_eq!(v.lemmatize("zorped", Verb), "zorp");
    assert_eq!(v.lemmatize("Zorped", Verb), "Zorp");
    assert_eq!(
        v.lemmas_for("zorped", Verb).first().map(|l| l.text),
        Some("zorp")
    );
    assert_eq!(moult::lemmatize("zorped", Verb), "zorped");

    let a = with("zorpier", Adjective, "zorpy");
    assert_eq!(a.lemmatize("zorpier", Adjective), "zorpy");
    assert_eq!(a.lemmatize("zorpier", Verb), "zorpier");

    let adv = with("zorpily", Adverb, "zorpy");
    assert_eq!(adv.lemmatize("zorpily", Adverb), "zorpy");

    let n = with("zorpen", Noun, "zorp");
    assert_eq!(n.lemmatize("zorpen", Noun), "zorp");
    assert_eq!(n.singularize("zorpen"), "zorp");
}

#[test]
fn an_override_beats_the_built_in_choice() {
    let inf = with("saw", Verb, "saw");
    assert_eq!(inf.lemmatize("saw", Verb), "saw");
    assert_eq!(moult::lemmatize("saw", Verb), "see");
    // The override is for the verb only; other parts of speech are untouched.
    assert_eq!(inf.lemmatize("better", Adjective), "good");
    // The built-in readings still follow the override.
    let got: Vec<_> = inf.lemmas_for("saw", Verb).map(|l| l.text).collect();
    assert_eq!(got, ["saw", "see"]);
}

#[test]
fn the_same_form_and_part_of_speech_twice_is_rejected() {
    for pos in ALL {
        let err = Inflector::builder()
            .lemma("geese", pos, "goose")
            .lemma("GEESE", pos, "gander")
            .build()
            .expect_err("duplicate");
        assert_eq!(
            err,
            OverrideError::DuplicateLemma("geese".into()),
            "{pos:?}"
        );
    }
    // The same form for two parts of speech is fine.
    assert!(Inflector::builder()
        .lemma("zorp", Verb, "zorp")
        .lemma("zorp", Adjective, "zorp")
        .build()
        .is_ok());
}

#[test]
fn a_noun_lemma_override_acts_as_the_pair() {
    let agree = Inflector::builder()
        .pair("zorp", "zorpen")
        .lemma("zorpen", Noun, "zorp")
        .build();
    assert!(agree.is_ok());
    let clash = Inflector::builder()
        .pair("zorp", "zorpen")
        .lemma("zorpen", Noun, "zorpa")
        .build();
    assert!(clash.is_err());
}

#[test]
fn a_noun_lemma_override_sets_the_plural() {
    assert_eq!(moult::pluralize("leaf"), "leaves");
    let inf = with("leafs", Noun, "leaf");
    assert_eq!(inf.pluralize("leaf"), "leafs");
    assert_eq!(inf.singularize("leafs"), "leaf");
    assert_eq!(inf.lemmatize("leafs", Noun), "leaf");
}

#[test]
fn free_functions_match_the_default_inflector() {
    let new = Inflector::new();
    let default = Inflector::default();
    for w in [
        "made",
        "Went",
        "BETTER",
        "saw",
        "hating",
        "leaves",
        "qwzzx",
        "a",
        "",
        "make it",
        "Americanize",
        "stopped",
        "lay",
    ] {
        for pos in ALL {
            for inf in [&new, &default] {
                assert_eq!(inf.lemmatize(w, pos), moult::lemmatize(w, pos), "{w}");
                assert!(inf.lemmas_for(w, pos).eq(moult::lemmas_for(w, pos)), "{w}");
            }
        }
        assert!(new.lemmas(w).eq(moult::lemmas(w)), "{w}");
        assert!(default.lemmas(w).eq(moult::lemmas(w)), "{w}");
    }
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
fn random_strings_never_panic_and_candidates_are_lower_case() {
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-' \u{e9}\u{df}\u{130}\u{1F600}"
        .chars()
        .collect();
    let tails = [
        "s", "es", "ies", "ied", "ing", "ed", "er", "est", "ier", "iest", "ying", "ses", "men",
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
        let all = moult::lemmas(&w);
        assert!(all.len() <= 16, "{w:?}");
        for pos in ALL {
            let some = moult::lemmas_for(&w, pos);
            assert!(some.len() <= 16, "{w:?}");
            let _ = moult::lemmatize(&w, pos);
        }
        for l in moult::lemmas(&w) {
            assert!(
                l.text.chars().all(|c| !c.is_uppercase()),
                "{w:?} -> {:?}",
                l.text
            );
        }
    }
}

#[test]
fn a_non_noun_ending_in_s_has_no_noun_reading_unless_it_is_a_plural() {
    // "alas" is no noun and not the plural of "ala"; singularize keeps it.
    assert!(!moult::is_noun("alas"));
    assert_eq!(moult::singularize("alas"), "alas");
    assert!(moult::lemmas_for("alas", Noun).next().is_none());
    assert_eq!(moult::lemmatize("alas", Noun), "alas");
    assert!(!list("alas").iter().any(|&(_, p)| p == Noun));
    // "bats" is listed as an adverb-or-other word too, but is the plural of
    // the noun "bat": the reading stays.
    for (w, noun) in [("bats", "bat"), ("needs", "need"), ("divers", "diver")] {
        assert_eq!(moult::singularize(w), noun, "{w}");
        let got: Vec<_> = moult::lemmas_for(w, Noun).map(|l| l.text).collect();
        assert_eq!(got.first(), Some(&noun), "{w}");
    }
}
