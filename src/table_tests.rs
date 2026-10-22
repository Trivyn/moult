//! Tests over the whole generated tables and the policy lists. They live
//! inside the crate because the tables are `pub(crate)`.

extern crate std;

use alloc::borrow::Cow;
use alloc::vec::Vec;
use std::string::String;

use crate::lookup::contains;
use crate::{data, pluralize, policy, singularize, PartOfSpeech};

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
    assert!(is_strictly_sorted(policy::S_REGULAR));
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

/// Whether WordNet, IRREGULAR or PLURAL_CHOICE lists a plural for `lemma`.
fn has_listed_plural(lemma: &str) -> bool {
    policy::IRREGULAR.iter().any(|&(s, _)| s == lemma)
        || policy::PLURAL_CHOICE
            .binary_search_by(|&(k, _)| k.cmp(lemma))
            .is_ok()
        || crate::lookup::rows_by_lemma(lemma)
            .any(|(form, l)| policy::EXCLUDED_FORMS.binary_search(&(l, form)).is_err())
}

#[test]
fn pluralize_leaves_plural_shaped_nouns_alone() {
    let (mut kept, mut s_regular) = (0, 0);
    let mut vowel_changed = 0;
    for l in nouns() {
        if singularize(l) != l || is_invariant(l) {
            continue;
        }
        let plural = pluralize(l);
        if !crate::rules::has_plural_shape(l) {
            // The vowel + s singulars ("bus", "status", "iris") keep their -es.
            let vowel_s = l.ends_with('s') && l[..l.len() - 1].ends_with(['a', 'i', 'o', 'u', 'y']);
            if vowel_s && plural != l {
                vowel_changed += 1;
            }
            continue;
        }
        if policy::S_REGULAR.contains(&l) {
            s_regular += 1;
            assert_eq!(plural, crate::rules::regular_plural(l), "{l}");
        } else if !has_listed_plural(l) {
            kept += 1;
            assert_eq!(plural, l, "{l} is plural-shaped with no plural");
        }
    }
    assert_eq!(s_regular, policy::S_REGULAR.len());
    // The same figure `cargo xtask report` prints for the plural-shaped
    // lemmas pluralize leaves unchanged.
    assert_eq!(kept, 418, "plural-only nouns");
    // Guards the vowel + s singulars: the count is the 0.1.0 measurement.
    assert!(
        vowel_changed >= 1369,
        "only {vowel_changed} vowel + s changed"
    );
    for (word, want) in [("bus", "buses"), ("status", "statuses"), ("iris", "irises")] {
        assert_eq!(pluralize(word), want);
    }
}

type LemmaTables = [(
    PartOfSpeech,
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
); 3];

/// Per part of speech: the lower-case lemmas, the capitalised-only lemmas and
/// the listed forms.
fn lemma_pos() -> LemmaTables {
    [
        (
            PartOfSpeech::Verb,
            data::VERBS,
            data::VERB_NAMES,
            data::VERB_FORMS,
        ),
        (
            PartOfSpeech::Adjective,
            data::ADJECTIVES,
            data::ADJECTIVE_NAMES,
            data::ADJECTIVE_FORMS,
        ),
        (
            PartOfSpeech::Adverb,
            data::ADVERBS,
            data::ADVERB_NAMES,
            data::ADVERB_FORMS,
        ),
    ]
}

/// Whether `text` is a dictionary lemma of `pos`: an entry of the part of
/// speech's list or of its capitalised-only list; for nouns, of NOUNS or NAMES.
fn is_lemma_of(text: &str, pos: PartOfSpeech) -> bool {
    match pos {
        PartOfSpeech::Verb => contains(data::VERBS, text) || contains(data::VERB_NAMES, text),
        PartOfSpeech::Adjective => {
            contains(data::ADJECTIVES, text) || contains(data::ADJECTIVE_NAMES, text)
        }
        PartOfSpeech::Adverb => contains(data::ADVERBS, text) || contains(data::ADVERB_NAMES, text),
        _ => contains(data::NOUNS, text) || contains(data::NAMES, text),
    }
}

/// Every word the lemma tests probe for `pos`: its lemmas and listed forms.
fn lemma_inputs(pos: PartOfSpeech) -> Vec<&'static str> {
    let mut out: Vec<&str> = Vec::new();
    for (p, words, names, forms) in lemma_pos() {
        if p == pos {
            out.extend(words.lines());
            out.extend(names.lines());
            out.extend(forms.iter().map(|&(form, _)| form));
        }
    }
    if pos == PartOfSpeech::Noun {
        out.extend(data::NOUNS.lines());
        out.extend(data::NAMES.lines());
        out.extend(data::FORM_TO_LEMMA.iter().map(|&(form, _)| form));
        out.extend(policy::INVARIANT);
        out.extend(policy::SINGULAR_CHOICE.iter().map(|&(k, _)| k));
    }
    out
}

const ALL_POS: [PartOfSpeech; 4] = [
    PartOfSpeech::Noun,
    PartOfSpeech::Verb,
    PartOfSpeech::Adjective,
    PartOfSpeech::Adverb,
];

#[test]
fn lemma_candidates_are_lemmas_of_their_part_of_speech() {
    let mut bad: Vec<String> = Vec::new();
    let mut most = 0;
    for pos in ALL_POS {
        for w in lemma_inputs(pos) {
            let all = crate::lemmas(w);
            most = most.max(all.len());
            for l in crate::lemmas_for(w, pos) {
                if l.pos != pos || !is_lemma_of(l.text, pos) {
                    bad.push(std::format!("{w}: {} is no {pos:?} lemma", l.text));
                }
            }
        }
    }
    assert!(bad.is_empty(), "bad candidates:\n{}", bad.join("\n"));
    assert!(
        most <= crate::pos::LEMMA_CAPACITY - 4,
        "{most} table-derived candidates"
    );
}

#[test]
fn every_lemma_lists_itself_and_lemmatizes_to_itself_when_first() {
    let mut bad: Vec<String> = Vec::new();
    for (pos, words, names, _) in lemma_pos() {
        for w in words.lines().chain(names.lines()) {
            let found = crate::lemmas_for(w, pos);
            if !found.clone().any(|l| l.text == w) {
                bad.push(std::format!("{w} ({pos:?}) does not list itself"));
            } else if found.first().map(|l| l.text) == Some(w) && crate::lemmatize(w, pos) != w {
                bad.push(std::format!("{w} ({pos:?}) lemmatizes away from itself"));
            }
        }
    }
    assert!(bad.is_empty(), "self lemmas:\n{}", bad.join("\n"));
}

#[test]
fn first_noun_candidate_is_singularizes_checked_answer() {
    let mut bad: Vec<String> = Vec::new();
    for w in lemma_inputs(PartOfSpeech::Noun) {
        let want = singularize(w);
        let found = crate::lemmas_for(w, PartOfSpeech::Noun);
        // When singularize's answer is a dictionary lemma it comes first;
        // otherwise (a policy-listed word WordNet lacks, a typo form the
        // policy excludes) it is no candidate at all.
        if is_lemma_of(&want, PartOfSpeech::Noun) {
            let got = found.first().map(|l| l.text);
            if got != Some(&*want) {
                bad.push(std::format!("{w}: {got:?} but singularize gives {want}"));
            }
        } else if found.clone().any(|l| l.text == &*want) {
            bad.push(std::format!("{w}: lists non-lemma {want}"));
        }
        if crate::lemmatize(w, PartOfSpeech::Noun) != want {
            bad.push(std::format!("{w}: lemmatize differs from singularize"));
        }
    }
    assert!(bad.is_empty(), "noun candidates:\n{}", bad.join("\n"));
}

#[test]
fn a_noun_candidate_is_a_dictionary_lemma_not_a_policy_word() {
    use PartOfSpeech::Noun;
    for w in ["ellipses", "chasses", "pooves"] {
        assert!(!is_lemma_of(w, Noun), "{w} is now a WordNet noun");
        assert!(crate::lemmas_for(w, Noun).all(|l| l.text != w), "{w}");
        assert_eq!(crate::lemmatize(w, Noun), singularize(w));
    }
}

#[test]
fn short_and_letterless_lemmas_list_themselves() {
    use PartOfSpeech::*;
    let own = |w: &str, pos| {
        crate::lemmas_for(w, pos)
            .map(|l| l.text)
            .collect::<Vec<_>>()
    };
    assert!(is_lemma_of("a", Noun) && is_lemma_of("10", Adjective));
    assert_eq!(own("a", Noun), ["a"]);
    assert_eq!(own("10", Adjective), ["10"]);
    assert_eq!(crate::lemmatize("10", Adjective), "10");
    assert_eq!(singularize("a"), "a");
    // Not a lemma of that part of speech: nothing; and not one word: nothing.
    assert!(own("10", Verb).is_empty() && own("s", Verb).is_empty());
    assert!(crate::lemmas("").is_empty() && crate::lemmas(" a").is_empty());
}

#[test]
fn capitalised_only_lemmas_are_known_but_never_a_rule_target() {
    use PartOfSpeech::*;
    let first = |w: &str, pos| crate::lemmas_for(w, pos).first();
    let lemma = |text, pos| Some(crate::Lemma { text, pos });
    assert_eq!(first("Americanize", Verb), lemma("americanize", Verb));
    assert_eq!(first("african", Adjective), lemma("african", Adjective));
    assert!(matches!(
        crate::lemmatize("Americanize", Verb),
        Cow::Borrowed(_)
    ));
    assert_eq!(crate::lemmatize("Americanize", Verb), "Americanize");
    assert_eq!(crate::lemmatize("AFRICAN", Adjective), "AFRICAN");
    // The word is known, so it is not a lemma of a lower-case rule stem: the
    // plural "africans" has no adjective reading through "african".
    for (names, pos) in [
        (data::VERB_NAMES, Verb),
        (data::ADJECTIVE_NAMES, Adjective),
        (data::ADVERB_NAMES, Adverb),
    ] {
        for w in names.lines() {
            for suffix in ["s", "es", "ed", "ing", "er", "est"] {
                let longer = std::format!("{w}{suffix}");
                assert!(
                    crate::lemmas_for(&longer, pos).all(|l| l.text != w),
                    "{longer} reaches {w}"
                );
            }
        }
    }
}

#[test]
fn lemma_tables_are_well_formed() {
    for (pos, words, names, forms) in lemma_pos() {
        well_formed_lines(words);
        well_formed_lines(names);
        for w in names.lines() {
            assert!(
                !contains(words, w),
                "{w} is both a lemma and capitalised-only"
            );
        }
        assert!(is_strictly_sorted(forms), "{pos:?} forms unsorted");
        for &(form, lemma) in forms {
            assert!(
                contains(words, lemma) || contains(names, lemma),
                "{form} -> {lemma} is no {pos:?}"
            );
            assert!(
                form == form.to_lowercase() && form.trim() == form && !form.is_empty(),
                "{form:?}"
            );
        }
    }
    let keys: Vec<(&str, PartOfSpeech)> = policy::LEMMA_CHOICE.iter().map(|r| (r.0, r.1)).collect();
    assert!(is_strictly_sorted(&keys), "LEMMA_CHOICE unsorted");
    for &(form, pos, lemma) in policy::LEMMA_CHOICE {
        assert!(
            pos != PartOfSpeech::Noun && is_lemma_of(lemma, pos),
            "{form}"
        );
    }
    assert!(is_strictly_sorted(policy::LEMMA_EXCLUDED));
    for &(form, pos, lemma) in policy::LEMMA_EXCLUDED {
        assert!(pos != PartOfSpeech::Noun, "{form}");
        let table = lemma_pos().iter().find(|r| r.0 == pos).map(|r| r.3);
        let listed = table.is_some_and(|t| t.contains(&(form, lemma)));
        assert!(!listed, "{form} -> {lemma} is excluded but listed");
    }
}

#[test]
fn others_words_offer_only_nouns_they_are_the_plural_of() {
    use PartOfSpeech::Noun;
    let mut with_reading = 0;
    for w in data::OTHERS.lines() {
        for l in crate::lemmas_for(w, Noun) {
            with_reading += 1;
            // The same gate as singularize: the word is not a noun, so a
            // noun reading must be one whose plural the word is.
            assert_eq!(
                crate::resolve::plural(l.text).0,
                w,
                "{w}: noun candidate {} is not a noun it is the plural of",
                l.text
            );
            assert_eq!(singularize(w), l.text, "{w}");
        }
    }
    assert!(with_reading > 0, "no OTHERS word has a noun reading");
}
