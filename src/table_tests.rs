//! Tests over the whole generated tables and the policy lists. They live
//! inside the crate because the tables are `pub(crate)`.

extern crate std;

use alloc::borrow::Cow;
use alloc::vec::Vec;
use std::string::String;

use crate::lookup::contains;
use crate::{data, pluralize, policy, singularize};

fn nouns() -> impl Iterator<Item = &'static str> {
    data::NOUNS.lines()
}

fn is_invariant(word: &str) -> bool {
    policy::INVARIANT.binary_search(&word).is_ok()
}

fn is_noun_lemma(word: &str) -> bool {
    contains(data::NOUNS, word)
}

fn is_strictly_sorted<T: Ord>(items: &[T]) -> bool {
    items.windows(2).all(|w| w[0] < w[1])
}

#[test]
fn singularize_never_invents_a_non_word() {
    let bad: Vec<String> = nouns()
        .filter_map(|l| {
            let s = singularize(l);
            (s != l && !is_noun_lemma(&s)).then(|| std::format!("{l} -> {s}"))
        })
        .collect();
    assert!(
        bad.is_empty(),
        "singularize left the noun list:\n{}",
        bad.join("\n")
    );
}

// Lemmas whose plural does not singularize back to them, by design. The test
// below fails on any lemma that is not listed here, and on any listed lemma
// that now round-trips, so the lists stay exact.

/// The plural is itself a noun that singularize leaves alone: plural-only and
/// `-ics` nouns ("pants", "ethics"), and pairs where WordNet lists the plural as
/// a lemma ("corp", "datum" to an invariant "data").
const PLURAL_IS_OWN_SINGULAR: &[&str] = &[
    "acoustic",
    "aesthetic",
    "auspex",
    "auspice",
    "bellow",
    "candelabrum",
    "ceramic",
    "chasse",
    "cheval-de-frise",
    "cinerarium",
    "corp",
    "crossroad",
    "datum",
    "dermatoglyphic",
    "dialectic",
    "djinni",
    "djinny",
    "dynamic",
    "ellipse",
    "ellipsis",
    "ephemeron",
    "esthetic",
    "ethic",
    "eyeglass",
    "graffito",
    "heroic",
    "hypochondrium",
    "hysteric",
    "inning",
    "isometric",
    "jean",
    "leu",
    "mean",
    "mechanic",
    "mew",
    "optic",
    "pant",
    "pharmaceutic",
    "physic",
    "plier",
    "poove",
    "pragmatic",
    "propaedeutic",
    "pubis",
    "pyrotechnic",
    "rich",
    "shamble",
    "shear",
    "shittah",
    "short",
    "signora",
    "specie",
    "stamen",
    "static",
    "sunglass",
    "therapeutic",
    "thrip",
    "trivium",
    "trouser",
];
/// The plural reduces to a different singular of the same word: a variant
/// spelling ("axe" and "ax", "caddy" and "caddie", "annexe" and "annex") or a
/// homograph with another lemma ("basis", "bases", "base"; "leave", "leaves",
/// "leaf"). Pluralizing one spelling and singularizing the result lands on the
/// other by design.
const PLURAL_READS_AS_SIBLING: &[&str] = &[
    "adz",
    "aery",
    "annexe",
    "anomy",
    "asbestos",
    "assagai",
    "aunty",
    "axe",
    "bacchant",
    "basis",
    "battle-ax",
    "beany",
    "bedesman",
    "benny",
    "bize",
    "blintze",
    "bogy",
    "bolshy",
    "booty",
    "breakax",
    "broadax",
    "burnouse",
    "buss",
    "caddy",
    "cartouch",
    "cattie",
    "chilly",
    "cicatrice",
    "cohoe",
    "cooky",
    "cooly",
    "cowry",
    "crosse",
    "darky",
    "deary",
    "dicky",
    "doggy",
    "dogy",
    "eclipsis",
    "entozoan",
    "epizoan",
    "eyry",
    "faery",
    "fesse",
    "floozy",
    "frijole",
    "galactosis",
    "gendarmery",
    "goony",
    "grannie",
    "grotesquery",
    "hanky",
    "hippy",
    "hoagy",
    "honky",
    "junky",
    "kelpy",
    "ketosis",
    "kiddy",
    "knobkerry",
    "largess",
    "leave",
    "lense",
    "lit",
    "loony",
    "marquise",
    "masse",
    "meany",
    "missis",
    "mollie",
    "monas",
    "nooky",
    "orach",
    "organdy",
    "pantie",
    "paralipsis",
    "paraphrasis",
    "pavis",
    "pickax",
    "pinche",
    "pinky",
    "pixy",
    "plectron",
    "plowman",
    "poleax",
    "prothalamium",
    "punky",
    "quicky",
    "revers",
    "revery",
    "roomy",
    "sharpy",
    "signior",
    "sise",
    "smoothy",
    "softy",
    "stogy",
    "stymy",
    "synapsis",
    "syringe",
    "towny",
    "weirdy",
    "yeshivah",
];

#[test]
fn round_trip_holds_except_for_the_allowlist() {
    assert!(is_strictly_sorted(PLURAL_IS_OWN_SINGULAR));
    assert!(is_strictly_sorted(PLURAL_READS_AS_SIBLING));
    let listed = |l: &str| {
        PLURAL_IS_OWN_SINGULAR.binary_search(&l).is_ok()
            || PLURAL_READS_AS_SIBLING.binary_search(&l).is_ok()
    };
    let (mut new_failures, mut stale) = (Vec::new(), Vec::new());
    for l in nouns() {
        if l.len() < 3 || is_invariant(l) || singularize(l) != l {
            continue;
        }
        let p = pluralize(l);
        let round_trips = singularize(&p) == l;
        match (round_trips, listed(l)) {
            (false, false) => new_failures.push(std::format!("{l} -> {p} -> {}", singularize(&p))),
            (true, true) => stale.push(l),
            _ => {}
        }
    }
    assert!(
        new_failures.is_empty(),
        "new round-trip failures:\n{}",
        new_failures.join("\n")
    );
    assert!(
        stale.is_empty(),
        "allowlisted but round-trips now: {stale:?}"
    );
}

#[test]
fn pluralize_never_yields_an_excluded_form() {
    let bad: Vec<String> = nouns()
        .filter_map(|l| {
            let p = pluralize(l);
            policy::EXCLUDED_FORMS
                .binary_search(&(l, &*p))
                .is_ok()
                .then(|| std::format!("{l} -> {p}"))
        })
        .collect();
    assert!(
        bad.is_empty(),
        "excluded forms produced:\n{}",
        bad.join("\n")
    );
}

#[test]
fn unchanged_words_are_borrowed() {
    for l in nouns() {
        let s = singularize(l);
        assert_eq!(matches!(s, Cow::Borrowed(_)), s == l, "singularize({l})");
        let p = pluralize(l);
        assert_eq!(matches!(p, Cow::Borrowed(_)), p == l, "pluralize({l})");
    }
    for w in policy::INVARIANT {
        assert!(matches!(pluralize(w), Cow::Borrowed(_)), "{w}");
        assert!(matches!(singularize(w), Cow::Borrowed(_)), "{w}");
    }
    for w in ["glasses", "men", "IDs", "Kubernetes", "", "s", "two words"] {
        assert!(matches!(pluralize(w), Cow::Borrowed(_)), "{w}");
    }
}

fn well_formed_lines(table: &str) {
    assert!(table.ends_with('\n'));
    let lines: Vec<&str> = table.lines().collect();
    assert!(lines.iter().all(|l| !l.is_empty()), "blank line");
    assert!(
        lines
            .iter()
            .all(|l| **l == *l.to_lowercase() && l.trim() == *l),
        "not lower case or padded"
    );
    assert!(is_strictly_sorted(&lines), "unsorted or duplicate");
}

#[test]
fn word_tables_are_well_formed() {
    well_formed_lines(data::NOUNS);
    well_formed_lines(data::NAMES);
    let overlap: Vec<&str> = data::NAMES.lines().filter(|n| is_noun_lemma(n)).collect();
    assert!(overlap.is_empty(), "in both NOUNS and NAMES: {overlap:?}");
}

#[test]
fn form_tables_are_sorted() {
    assert!(is_strictly_sorted(data::FORM_TO_LEMMA));
    // LEMMA_TO_PLURAL is a permutation of FORM_TO_LEMMA's indices, ordered by
    // (lemma, form).
    let index = data::LEMMA_TO_PLURAL;
    assert_eq!(index.len(), data::FORM_TO_LEMMA.len());
    let mut seen = std::vec![false; index.len()];
    for &i in index {
        let slot = &mut seen[usize::from(i)];
        assert!(!*slot, "index {i} repeated");
        *slot = true;
    }
    let by_lemma: Vec<(&str, &str)> = index
        .iter()
        .map(|&i| {
            let (form, lemma) = data::FORM_TO_LEMMA[usize::from(i)];
            (lemma, form)
        })
        .collect();
    assert!(is_strictly_sorted(&by_lemma));
    // rows_by_lemma finds exactly the rows a linear scan finds.
    for &(lemma, _) in by_lemma.iter().step_by(7) {
        let found: Vec<(&str, &str)> = crate::lookup::rows_by_lemma(lemma).collect();
        let scan: Vec<(&str, &str)> = data::FORM_TO_LEMMA
            .iter()
            .copied()
            .filter(|&(_, l)| l == lemma)
            .collect();
        assert_eq!(found, scan, "{lemma}");
    }
    assert_eq!(crate::lookup::rows_by_lemma("zzzz-none").count(), 0);
}

#[test]
fn policy_lists_are_sorted_without_duplicate_keys() {
    assert!(is_strictly_sorted(policy::INVARIANT));
    assert!(is_strictly_sorted(policy::MAN_REGULAR));
    assert!(is_strictly_sorted(policy::CH_REGULAR));
    assert!(is_strictly_sorted(policy::EXCLUDED_FORMS));
    assert!(is_strictly_sorted(policy::SINGULAR_CHOICE));
    assert!(is_strictly_sorted(policy::PLURAL_CHOICE));
    for table in [policy::SINGULAR_CHOICE, policy::PLURAL_CHOICE] {
        let keys: Vec<&str> = table.iter().map(|r| r.0).collect();
        assert!(is_strictly_sorted(&keys), "duplicate key");
    }
    // IRREGULAR is scanned, so order is free, but no key may repeat.
    for (name, side) in [("singular", 0), ("plural", 1)] {
        let mut keys: Vec<&str> = policy::IRREGULAR
            .iter()
            .map(|r| if side == 0 { r.0 } else { r.1 })
            .collect();
        keys.sort_unstable();
        assert!(is_strictly_sorted(&keys), "duplicate IRREGULAR {name}");
    }
}

#[test]
fn contains_agrees_with_a_linear_scan() {
    let words: Vec<&str> = nouns().collect();
    let scan = |w: &str| words.contains(&w);
    for w in words.iter().step_by(97) {
        assert!(contains(data::NOUNS, w) && scan(w), "{w}");
        for extra in ["s", "x", "-"] {
            let longer = std::format!("{w}{extra}");
            assert_eq!(contains(data::NOUNS, &longer), scan(&longer), "{longer}");
        }
        let shorter = &w[..w.len() - 1];
        assert_eq!(contains(data::NOUNS, shorter), scan(shorter), "{shorter}");
    }
    for w in ["", "a", "zzzzzz", "0", "~", "\u{e9}"] {
        assert_eq!(contains(data::NOUNS, w), scan(w), "{w:?}");
    }
}

#[test]
fn others_table_is_well_formed() {
    let words: Vec<&str> = data::OTHERS.lines().collect();
    assert!(!words.is_empty() && data::OTHERS.ends_with('\n'));
    assert!(is_strictly_sorted(&words), "OTHERS is sorted and unique");
    for w in &words {
        assert!(w.ends_with('s'), "{w} does not end in s");
        assert_eq!(*w, w.to_lowercase(), "{w} is not lower case");
        assert!(!contains(data::NOUNS, w), "{w} is also a noun");
        assert!(!contains(data::NAMES, w), "{w} is also a name");
        // Step 5 precedes the checked rules: an OTHERS word is unchanged and
        // not a noun, or it is the plural of the noun it reduces to.
        let sing = singularize(w);
        if sing == *w {
            assert!(!crate::is_noun(w), "{w} reads as a noun");
        } else {
            assert!(crate::is_noun(w), "{w} reduces but is not a noun");
            assert_eq!(pluralize(&sing), *w, "{w} -> {sing} is not its plural");
        }
        assert_eq!(pluralize(w), *w, "{w} changed by pluralize");
    }
}
