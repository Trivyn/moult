//! The acceptance cases: words that hand-written suffix rules get wrong in
//! one direction or the other. Words with two right answers ("leaves": leaf
//! or leave; "axes": axis, ax or axe) are here only because the crate states
//! a policy for them (src/policy.rs): the common modern reading wins, and a
//! word with no clear reading is left unchanged.

use std::borrow::Cow;

const SINGULAR: &[(&str, &str)] = &[
    // Regular plurals.
    ("territories", "territory"),
    ("boxes", "box"),
    ("glasses", "glass"),
    ("quizzes", "quiz"),
    ("metrics", "metric"),
    ("statuses", "status"),
    ("buses", "bus"),
    ("heroes", "hero"),
    ("potatoes", "potato"),
    ("toes", "toe"),
    ("wolves", "wolf"),
    ("knives", "knife"),
    // "-ies" after "ie" stems, and "-ves" that is not "-f"/"-fe".
    ("cookies", "cookie"),
    ("zombies", "zombie"),
    ("calories", "calorie"),
    ("waves", "wave"),
    ("caves", "cave"),
    ("drives", "drive"),
    ("curves", "curve"),
    // Endings a rule mistakes for an irregular plural.
    ("police", "police"),
    ("slices", "slice"),
    ("quotas", "quota"),
    ("quota", "quota"),
    ("deltas", "delta"),
    // Irregular forms.
    ("mice", "mouse"),
    ("people", "person"),
    ("criteria", "criterion"),
    ("indices", "index"),
    ("matrices", "matrix"),
    ("crises", "crisis"),
    ("analyses", "analysis"),
    // Already singular, though they end in "s".
    ("gas", "gas"),
    ("lens", "lens"),
    ("bus", "bus"),
    ("analysis", "analysis"),
    ("status", "status"),
    // Invariant and mass nouns.
    ("data", "data"),
    ("news", "news"),
    ("series", "series"),
    ("species", "species"),
    ("sheep", "sheep"),
    ("logistics", "logistics"),
    ("physics", "physics"),
    // Case is kept.
    ("Territories", "Territory"),
    ("ISBN", "ISBN"),
    ("ISBNs", "ISBN"),
    ("IDs", "ID"),
    // Words with two readings, settled by policy.
    ("leaves", "leaf"),
    ("axes", "axis"),
    ("corpses", "corpse"),
    ("senses", "sense"),
    ("busses", "bus"),
    ("Kansas", "Kansas"),
    // Compounds of man and Greek nouns in -sis.
    ("policemen", "policeman"),
    ("women", "woman"),
    ("mitoses", "mitosis"),
    // Case survives past the changed part of the word.
    ("Mothers-in-Law", "Mother-in-Law"),
    // The default mode never guesses: a word no dictionary-checked step
    // resolves is unchanged. The refused words below are also refused by the
    // opt-in fallback (tests/guess.rs).
    ("backwards", "backwards"),
    ("afterwards", "afterwards"),
    ("themselves", "themselves"),
    ("superspecies", "superspecies"),
    ("subseries", "subseries"),
    ("facies", "facies"),
    ("biomechanics", "biomechanics"),
    ("tholos", "tholos"),
    ("xyrichthys", "xyrichthys"),
    ("agaricales", "agaricales"),
    ("moses", "moses"),
    ("ulysses", "ulysses"),
    ("kubernetes", "kubernetes"),
    ("Kubernetes", "Kubernetes"),
    ("laches", "laches"),
    ("abies", "abies"),
    ("colluvies", "colluvies"),
    ("hirsuties", "hirsuties"),
    ("natchitoches", "natchitoches"),
    ("lichenes", "lichenes"),
    ("albanenses", "albanenses"),
    // One character is never inflected.
    ("s", "s"),
    ("a", "a"),
    // Words WordNet lists only as adverbs, adjectives or verbs are known:
    // the unknown-word fallback must not strip their `s`.
    ("perhaps", "perhaps"),
    ("sometimes", "sometimes"),
    ("besides", "besides"),
    ("indoors", "indoors"),
    ("afterwards", "afterwards"),
    ("midships", "midships"),
    ("always", "always"),
    ("overseas", "overseas"),
    // A -chs plural of a /k/ noun reduces by the checked -s rule.
    ("stomachs", "stomach"),
    ("monarchs", "monarch"),
    ("epochs", "epoch"),
    // Plurals WordNet does not list stay unchanged without the opt-in guess;
    // tests/guess.rs shows what `guess_unknown(true)` does with them.
    ("microservices", "microservices"),
    ("namespaces", "namespaces"),
    ("webhooks", "webhooks"),
    // Compounds whose last part is a noun: the opt-in guess strips them to a
    // non-word, the default leaves them alone.
    ("afterhours", "afterhours"),
    ("belowstairs", "belowstairs"),
    // Plurals that dictionary-checked rules reduce.
    ("endpoints", "endpoint"),
    ("workflows", "workflow"),
    ("widgets", "widget"),
    // "elap" is no word, so even the opt-in guess leaves these alone.
    ("elaps", "elaps"),
    ("custodes", "custodes"),
    // WordNet lists "alas" only as an interjection: no rule reduces it.
    ("alas", "alas"),
    // WordNet lists these as other parts of speech, yet they are noun plurals.
    ("bats", "bat"),
    ("nuts", "nut"),
    ("tops", "top"),
];

const PLURAL: &[(&str, &str)] = &[
    // Case of an irregular plural follows the word; a rule suffix stays lower case.
    ("CHILD", "CHILDREN"),
    ("OX", "OXEN"),
    ("Child", "Children"),
    ("PERSON", "PEOPLE"),
    ("MOUSE", "MICE"),
    ("CRITERION", "CRITERIA"),
    ("BOX", "BOXes"),
    ("ISBN", "ISBNs"),
    ("ID", "IDs"),
    ("alas", "alas"),
    ("territory", "territories"),
    ("box", "boxes"),
    ("bus", "buses"),
    ("status", "statuses"),
    ("quiz", "quizzes"),
    ("hero", "heroes"),
    ("potato", "potatoes"),
    ("photo", "photos"),
    ("wolf", "wolves"),
    ("roof", "roofs"),
    ("chief", "chiefs"),
    ("person", "people"),
    ("mouse", "mice"),
    ("criterion", "criteria"),
    ("analysis", "analyses"),
    ("sheep", "sheep"),
    ("data", "data"),
    ("ISBN", "ISBNs"),
    ("Territory", "Territories"),
    ("policeman", "policemen"),
    ("woman", "women"),
    ("mitosis", "mitoses"),
    // A ch pronounced /k/ takes a plain s; the controls below keep es.
    ("stomach", "stomachs"),
    ("monarch", "monarchs"),
    ("epoch", "epochs"),
    ("loch", "lochs"),
    ("patriarch", "patriarchs"),
    ("eunuch", "eunuchs"),
    ("triptych", "triptychs"),
    ("tech", "techs"),
    ("church", "churches"),
    ("march", "marches"),
    ("larch", "larches"),
    ("couch", "couches"),
    ("beach", "beaches"),
    ("speech", "speeches"),
    // Already plural, so unchanged.
    ("glasses", "glasses"),
    ("men", "men"),
    ("ID", "IDs"),
    ("BOX", "BOXes"),
    ("MOUSE", "MICE"),
    ("Person", "People"),
    ("quiz", "quizzes"),
    ("gas", "gases"),
    ("genus", "genera"),
    ("brother", "brothers"),
    // Plural-only nouns are unchanged, in any case; the real singulars of that
    // shape take es, and a WordNet form still wins.
    ("thanks", "thanks"),
    ("Thanks", "Thanks"),
    ("clothes", "clothes"),
    ("earnings", "earnings"),
    ("geophysics", "geophysics"),
    ("1990s", "1990s"),
    ("measles", "measles"),
    ("avionics", "avionics"),
    ("lens", "lenses"),
    ("summons", "summonses"),
    ("yes", "yeses"),
    ("glans", "glandes"),
    ("cyclops", "cyclopes"),
    // WordNet lists another noun's plural against these lemmas.
    ("diva", "divas"),
    ("sol", "sols"),
    ("Mother-in-Law", "Mothers-in-Law"),
    // Input that is not a word is returned unchanged.
    ("41", "41"),
    ("two words", "two words"),
    // Unknown words ending in s are taken as plural already.
    ("kubernetes", "kubernetes"),
    ("Kubernetes", "Kubernetes"),
    ("moses", "moses"),
    ("facies", "facies"),
    // Unknown words ending in s are taken as plural, with or without the guess.
    ("elaps", "elaps"),
    ("custodes", "custodes"),
    // WordNet lists these only as another part of speech: no rule reduces them.
    ("alas", "alas"),
    // One character is never inflected.
    ("s", "s"),
    ("a", "a"),
    ("I", "I"),
];

fn check(cases: &[(&str, &str)], f: fn(&str) -> Cow<'_, str>) {
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(|&(input, want)| {
            let got = f(input);
            (&*got != want).then(|| format!("{input} -> {got} (want {want})"))
        })
        .collect();
    assert!(wrong.is_empty(), "wrong:\n{}", wrong.join("\n"));
}

#[test]
fn singularize_cases() {
    check(SINGULAR, moult::singularize);
}

#[test]
fn pluralize_cases() {
    check(PLURAL, moult::pluralize);
}
